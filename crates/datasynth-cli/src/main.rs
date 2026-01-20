//! CLI for synthetic accounting data generation.

use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use anyhow::Result;
use clap::{Parser, Subcommand};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use datasynth_config::{presets, GeneratorConfig};
use datasynth_core::memory_guard::{MemoryGuard, MemoryGuardConfig};
use datasynth_core::models::{CoAComplexity, IndustrySector};
use datasynth_runtime::{EnhancedOrchestrator, PhaseConfig};

#[cfg(unix)]
use signal_hook::consts::SIGUSR1;

#[derive(Parser)]
#[command(name = "synth-data")]
#[command(about = "Synthetic Enterprise Accounting Data Generator")]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    /// Enable verbose output
    #[arg(short, long, global = true)]
    verbose: bool,
}

#[derive(Subcommand)]
enum Commands {
    /// Generate synthetic accounting data
    Generate {
        /// Path to configuration file
        #[arg(short, long)]
        config: Option<PathBuf>,

        /// Output directory
        #[arg(short, long, default_value = "./output")]
        output: PathBuf,

        /// Use demo preset (small dataset for testing)
        #[arg(long)]
        demo: bool,

        /// Random seed for reproducibility
        #[arg(short, long)]
        seed: Option<u64>,

        /// Enable banking KYC/AML data generation
        #[arg(long)]
        banking: bool,

        /// Enable audit data generation
        #[arg(long)]
        audit: bool,

        /// Memory limit in MB (default: 1024 MB)
        #[arg(long, default_value = "1024")]
        memory_limit: usize,

        /// Maximum CPU threads to use (default: half of available cores, min 1)
        #[arg(long)]
        max_threads: Option<usize>,
    },

    /// Validate a configuration file
    Validate {
        /// Path to configuration file
        #[arg(short, long)]
        config: PathBuf,
    },

    /// Generate a sample configuration file
    Init {
        /// Output path
        #[arg(short, long, default_value = "datasynth_config.yaml")]
        output: PathBuf,

        /// Industry preset
        #[arg(short, long, default_value = "manufacturing")]
        industry: String,

        /// CoA complexity (small, medium, large)
        #[arg(short, long, default_value = "medium")]
        complexity: String,
    },

    /// Show information about available presets
    Info,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    // Setup logging
    let filter = if cli.verbose { "debug" } else { "info" };
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| filter.into()),
        )
        .init();

    match cli.command {
        Commands::Generate {
            config,
            output,
            demo,
            seed,
            banking,
            audit,
            memory_limit,
            max_threads,
        } => {
            // ========================================
            // CPU SAFEGUARD: Limit thread pool size
            // ========================================
            let available_cpus = num_cpus::get();
            let effective_threads = max_threads.unwrap_or_else(|| {
                // Default: use half of available cores, minimum 1, maximum 4
                (available_cpus / 2).max(1).min(4)
            });

            // Configure rayon thread pool with limited threads
            rayon::ThreadPoolBuilder::new()
                .num_threads(effective_threads)
                .build_global()
                .ok(); // Ignore error if already initialized

            tracing::info!(
                "CPU safeguard: using {} threads (of {} available)",
                effective_threads,
                available_cpus
            );

            // ========================================
            // MEMORY SAFEGUARD: Set conservative limits
            // ========================================
            let effective_memory_limit = if memory_limit > 0 {
                memory_limit.min(get_safe_memory_limit()) // Cap at safe limit
            } else {
                1024 // Default 1GB
            };

            let memory_config =
                MemoryGuardConfig::with_limit_mb(effective_memory_limit).aggressive();
            let memory_guard = Arc::new(MemoryGuard::new(memory_config));

            tracing::info!(
                "Memory safeguard: {} MB limit ({} MB soft limit)",
                effective_memory_limit,
                (effective_memory_limit * 80) / 100
            );

            // Check initial memory status
            let initial_memory = memory_guard.current_usage_mb();
            tracing::info!("Initial memory usage: {} MB", initial_memory);

            // ========================================
            // LOAD CONFIGURATION
            // ========================================
            let mut generator_config = if demo {
                tracing::info!("Using demo preset (conservative settings)");
                create_safe_demo_preset()
            } else if let Some(config_path) = config {
                let content = std::fs::read_to_string(&config_path)?;
                let mut cfg: GeneratorConfig = serde_yaml::from_str(&content)?;
                // Apply safety limits to loaded config
                apply_safety_limits(&mut cfg);
                cfg
            } else {
                tracing::info!("No config specified, using safe demo preset");
                create_safe_demo_preset()
            };

            if let Some(s) = seed {
                generator_config.global.seed = Some(s);
            }

            // Enable banking if flag is set (with conservative defaults)
            if banking {
                generator_config.banking.enabled = true;
                // Apply conservative banking limits
                generator_config.banking.population.retail_customers = generator_config
                    .banking
                    .population
                    .retail_customers
                    .min(100);
                generator_config.banking.population.business_customers = generator_config
                    .banking
                    .population
                    .business_customers
                    .min(20);
                generator_config.banking.population.trusts =
                    generator_config.banking.population.trusts.min(5);
                tracing::info!("Banking KYC/AML generation enabled (conservative mode)");
            }

            generator_config.output.output_directory = output.clone();
            generator_config.global.parallel = false; // Disable parallel for safety
            generator_config.global.worker_threads = effective_threads;
            generator_config.global.memory_limit_mb = effective_memory_limit;

            tracing::info!("Starting generation...");
            tracing::info!("Industry: {:?}", generator_config.global.industry);
            tracing::info!("Period: {} months", generator_config.global.period_months);
            tracing::info!("Companies: {}", generator_config.companies.len());

            // ========================================
            // SIGNAL HANDLING (Unix only)
            // ========================================
            let pause_flag = Arc::new(AtomicBool::new(false));

            #[cfg(unix)]
            {
                let pause_flag_clone = Arc::clone(&pause_flag);
                let signal_flag = Arc::new(AtomicBool::new(false));
                let signal_flag_clone = Arc::clone(&signal_flag);

                if signal_hook::flag::register(SIGUSR1, signal_flag_clone).is_ok() {
                    let pid = std::process::id();
                    tracing::info!("Pause/resume: send SIGUSR1 to toggle (kill -USR1 {})", pid);

                    std::thread::spawn(move || loop {
                        if signal_flag.swap(false, Ordering::Relaxed) {
                            let was_paused = pause_flag_clone.load(Ordering::Relaxed);
                            pause_flag_clone.store(!was_paused, Ordering::Relaxed);
                            if was_paused {
                                eprintln!("\n>>> RESUMED");
                            } else {
                                eprintln!("\n>>> PAUSED - send SIGUSR1 again to resume");
                            }
                        }
                        std::thread::sleep(std::time::Duration::from_millis(100));
                    });
                }
            }

            // ========================================
            // PRE-GENERATION MEMORY CHECK
            // ========================================
            if let Err(e) = memory_guard.check_now() {
                tracing::error!("Memory limit already exceeded before generation: {}", e);
                return Err(anyhow::anyhow!("Insufficient memory to start generation"));
            }

            // ========================================
            // GENERATE DATA
            // ========================================
            let phase_config = PhaseConfig {
                generate_banking: banking,
                generate_audit: audit,
                show_progress: true,
                // Use conservative defaults for document generation
                p2p_chains: 50,
                o2c_chains: 50,
                vendors_per_company: 20,
                customers_per_company: 30,
                materials_per_company: 50,
                assets_per_company: 20,
                employees_per_company: 30,
                ..PhaseConfig::default()
            };

            let mut orchestrator = EnhancedOrchestrator::new(generator_config, phase_config)?;
            let result = orchestrator.generate()?;

            // ========================================
            // REPORT RESULTS
            // ========================================
            tracing::info!("Generation complete!");
            tracing::info!("Total entries: {}", result.statistics.total_entries);
            tracing::info!("Total line items: {}", result.statistics.total_line_items);
            tracing::info!("Accounts in CoA: {}", result.statistics.accounts_count);

            // Memory usage reporting
            let stats = memory_guard.stats();
            let peak_mb = stats.peak_resident_bytes / (1024 * 1024);
            let current_mb = stats.resident_bytes / (1024 * 1024);
            tracing::info!(
                "Memory usage: current {} MB, peak {} MB",
                current_mb,
                peak_mb
            );
            if stats.soft_limit_warnings > 0 {
                tracing::warn!(
                    "Memory soft limit was exceeded {} times during generation",
                    stats.soft_limit_warnings
                );
            }

            // Banking statistics
            if result.statistics.banking_customer_count > 0 {
                tracing::info!(
                    "Banking: {} customers, {} accounts, {} transactions ({} suspicious)",
                    result.statistics.banking_customer_count,
                    result.statistics.banking_account_count,
                    result.statistics.banking_transaction_count,
                    result.statistics.banking_suspicious_count
                );
            }

            // Audit statistics
            if result.statistics.audit_engagement_count > 0 {
                tracing::info!(
                    "Audit: {} engagements, {} workpapers, {} findings",
                    result.statistics.audit_engagement_count,
                    result.statistics.audit_workpaper_count,
                    result.statistics.audit_finding_count
                );
            }

            // ========================================
            // WRITE OUTPUT (with memory checks)
            // ========================================
            std::fs::create_dir_all(&output)?;

            // Check memory before writing
            if memory_guard.check_now().is_err() {
                tracing::warn!("Memory limit reached, writing minimal output");
            }

            // Write sample output (limited to 1000 entries for safety)
            let sample_path = output.join("sample_entries.json");
            let sample_entries: Vec<_> = result.journal_entries.iter().take(1000).collect();
            let json = serde_json::to_string_pretty(&sample_entries)?;
            std::fs::write(&sample_path, json)?;
            tracing::info!(
                "Sample entries written to: {} ({} entries)",
                sample_path.display(),
                sample_entries.len()
            );

            // Write banking output if generated
            if banking && !result.banking.customers.is_empty() {
                let banking_dir = output.join("banking");
                std::fs::create_dir_all(&banking_dir)?;

                // Write banking customers
                let customers_path = banking_dir.join("banking_customers.json");
                let json = serde_json::to_string_pretty(&result.banking.customers)?;
                std::fs::write(&customers_path, json)?;

                // Write banking accounts
                let accounts_path = banking_dir.join("banking_accounts.json");
                let json = serde_json::to_string_pretty(&result.banking.accounts)?;
                std::fs::write(&accounts_path, json)?;

                // Write banking transactions (limited for safety)
                let transactions_path = banking_dir.join("banking_transactions.json");
                let limited_txns: Vec<_> = result.banking.transactions.iter().take(10000).collect();
                let json = serde_json::to_string_pretty(&limited_txns)?;
                std::fs::write(&transactions_path, json)?;

                tracing::info!(
                    "Banking data written to: {} ({} customers, {} accounts, {} transactions)",
                    banking_dir.display(),
                    result.banking.customers.len(),
                    result.banking.accounts.len(),
                    limited_txns.len()
                );
            }

            Ok(())
        }

        Commands::Validate { config } => {
            let content = std::fs::read_to_string(&config)?;
            let generator_config: GeneratorConfig = serde_yaml::from_str(&content)?;
            datasynth_config::validate_config(&generator_config)?;
            tracing::info!("Configuration is valid!");
            Ok(())
        }

        Commands::Init {
            output,
            industry,
            complexity,
        } => {
            let industry_sector = match industry.to_lowercase().as_str() {
                "manufacturing" => IndustrySector::Manufacturing,
                "retail" => IndustrySector::Retail,
                "financial" | "financial_services" => IndustrySector::FinancialServices,
                "healthcare" => IndustrySector::Healthcare,
                "technology" | "tech" => IndustrySector::Technology,
                _ => IndustrySector::Manufacturing,
            };

            let coa_complexity = match complexity.to_lowercase().as_str() {
                "small" => CoAComplexity::Small,
                "medium" => CoAComplexity::Medium,
                "large" => CoAComplexity::Large,
                _ => CoAComplexity::Medium,
            };

            let config = presets::create_preset(
                industry_sector,
                2,
                12,
                coa_complexity,
                datasynth_config::TransactionVolume::TenK, // Conservative default
            );

            let yaml = serde_yaml::to_string(&config)?;
            std::fs::write(&output, yaml)?;
            tracing::info!("Configuration written to: {}", output.display());
            Ok(())
        }

        Commands::Info => {
            println!("Available Industry Presets:");
            println!("  - manufacturing: Manufacturing industry");
            println!("  - retail: Retail industry");
            println!("  - financial_services: Financial services");
            println!("  - healthcare: Healthcare industry");
            println!("  - technology: Technology industry");
            println!();
            println!("Chart of Accounts Complexity:");
            println!("  - small: ~100 accounts");
            println!("  - medium: ~400 accounts");
            println!("  - large: ~2500 accounts");
            println!();
            println!("Transaction Volumes:");
            println!("  - ten_k: 10,000 transactions/year");
            println!("  - hundred_k: 100,000 transactions/year");
            println!("  - one_m: 1,000,000 transactions/year");
            println!("  - ten_m: 10,000,000 transactions/year");
            println!("  - hundred_m: 100,000,000 transactions/year");
            println!();
            println!("Resource Safeguards:");
            println!("  --memory-limit <MB>  : Set memory limit (default: 1024 MB)");
            println!("  --max-threads <N>    : Limit CPU threads (default: half of cores, max 4)");
            Ok(())
        }
    }
}

/// Create a safe demo preset with conservative resource usage.
fn create_safe_demo_preset() -> GeneratorConfig {
    use datasynth_config::schema::*;

    GeneratorConfig {
        global: GlobalConfig {
            industry: IndustrySector::Manufacturing,
            start_date: "2024-01-01".to_string(),
            period_months: 1, // Just 1 month for demo
            seed: Some(42),
            parallel: false,
            group_currency: "USD".to_string(),
            worker_threads: 2,
            memory_limit_mb: 512,
        },
        companies: vec![CompanyConfig {
            code: "DEMO".to_string(),
            name: "Demo Company".to_string(),
            currency: "USD".to_string(),
            country: "US".to_string(),
            annual_transaction_volume: TransactionVolume::TenK, // Small volume
            volume_weight: 1.0,
            fiscal_year_variant: "K4".to_string(),
        }],
        chart_of_accounts: ChartOfAccountsConfig {
            complexity: CoAComplexity::Small,
            industry_specific: false,
            custom_accounts: None,
            min_hierarchy_depth: 2,
            max_hierarchy_depth: 3,
        },
        transactions: TransactionConfig::default(),
        output: OutputConfig::default(),
        fraud: FraudConfig {
            enabled: false,
            ..Default::default()
        },
        internal_controls: InternalControlsConfig::default(),
        business_processes: BusinessProcessConfig::default(),
        user_personas: UserPersonaConfig::default(),
        templates: TemplateConfig::default(),
        approval: ApprovalConfig::default(),
        departments: DepartmentConfig::default(),
        master_data: MasterDataConfig::default(),
        document_flows: DocumentFlowConfig::default(),
        intercompany: IntercompanyConfig::default(),
        balance: BalanceConfig::default(),
        ocpm: OcpmConfig::default(),
        audit: AuditGenerationConfig::default(),
        banking: datasynth_banking::BankingConfig::small(), // Use small banking config
        data_quality: DataQualitySchemaConfig::default(),
    }
}

/// Apply safety limits to a loaded configuration.
fn apply_safety_limits(config: &mut GeneratorConfig) {
    // Limit period to 12 months max
    config.global.period_months = config.global.period_months.min(12);

    // Limit transaction volume
    for company in &mut config.companies {
        company.annual_transaction_volume = match company.annual_transaction_volume {
            datasynth_config::TransactionVolume::OneM
            | datasynth_config::TransactionVolume::TenM
            | datasynth_config::TransactionVolume::HundredM => {
                datasynth_config::TransactionVolume::HundredK
            }
            other => other,
        };
    }

    // Limit banking population
    if config.banking.enabled {
        config.banking.population.retail_customers =
            config.banking.population.retail_customers.min(500);
        config.banking.population.business_customers =
            config.banking.population.business_customers.min(100);
        config.banking.population.trusts = config.banking.population.trusts.min(20);
    }

    // Force conservative settings
    config.global.parallel = false;
    config.global.worker_threads = config.global.worker_threads.min(4);
}

/// Get safe memory limit based on available system memory.
/// Returns a conservative limit that won't overwhelm the system.
fn get_safe_memory_limit() -> usize {
    #[cfg(target_os = "linux")]
    {
        if let Ok(content) = std::fs::read_to_string("/proc/meminfo") {
            for line in content.lines() {
                if line.starts_with("MemAvailable:") {
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    if parts.len() >= 2 {
                        if let Ok(kb) = parts[1].parse::<usize>() {
                            let mb = kb / 1024;
                            // Use 50% of available memory, capped at 4GB
                            return (mb / 2).min(4096);
                        }
                    }
                    break;
                }
            }
        }
    }

    // Default to 1GB if detection fails
    1024
}
