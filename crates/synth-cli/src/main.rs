//! CLI for synthetic accounting data generation.

use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use anyhow::Result;
use clap::{Parser, Subcommand};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use synth_config::{presets, GeneratorConfig};
use synth_core::models::{CoAComplexity, IndustrySector};
use synth_runtime::GenerationOrchestrator;

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
        #[arg(short, long, default_value = "synth_config.yaml")]
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
        } => {
            let mut generator_config = if demo {
                tracing::info!("Using demo preset");
                presets::demo_preset()
            } else if let Some(config_path) = config {
                let content = std::fs::read_to_string(&config_path)?;
                serde_yaml::from_str(&content)?
            } else {
                tracing::info!("No config specified, using demo preset");
                presets::demo_preset()
            };

            if let Some(s) = seed {
                generator_config.global.seed = Some(s);
            }

            generator_config.output.output_directory = output.clone();

            tracing::info!("Starting generation...");
            tracing::info!("Industry: {:?}", generator_config.global.industry);
            tracing::info!("Period: {} months", generator_config.global.period_months);
            tracing::info!("Companies: {}", generator_config.companies.len());

            // Set up pause flag for signal handling
            let pause_flag = Arc::new(AtomicBool::new(false));

            // Register signal handler on Unix systems
            #[cfg(unix)]
            {
                let pause_flag_clone = Arc::clone(&pause_flag);
                // Use register to set flag to true, then spawn a thread to toggle
                let signal_flag = Arc::new(AtomicBool::new(false));
                let signal_flag_clone = Arc::clone(&signal_flag);

                if signal_hook::flag::register(SIGUSR1, signal_flag_clone).is_ok() {
                    let pid = std::process::id();
                    tracing::info!("Pause/resume: send SIGUSR1 to toggle (kill -USR1 {})", pid);

                    // Spawn a thread to monitor the signal and toggle pause state
                    std::thread::spawn(move || {
                        loop {
                            if signal_flag.swap(false, Ordering::Relaxed) {
                                // Signal received - toggle pause state
                                let was_paused = pause_flag_clone.load(Ordering::Relaxed);
                                pause_flag_clone.store(!was_paused, Ordering::Relaxed);
                                if was_paused {
                                    eprintln!("\n>>> RESUMED");
                                } else {
                                    eprintln!("\n>>> PAUSED - send SIGUSR1 again to resume");
                                }
                            }
                            std::thread::sleep(std::time::Duration::from_millis(50));
                        }
                    });
                } else {
                    tracing::warn!("Failed to register SIGUSR1 handler");
                }
            }

            let mut orchestrator =
                GenerationOrchestrator::new(generator_config)?.with_pause_flag(pause_flag);
            let result = orchestrator.generate()?;

            tracing::info!("Generation complete!");
            tracing::info!("Total entries: {}", result.statistics.total_entries);
            tracing::info!("Total line items: {}", result.statistics.total_line_items);
            tracing::info!("Accounts in CoA: {}", result.statistics.accounts_count);

            // Create output directory
            std::fs::create_dir_all(&output)?;

            // Write sample output (up to 10000 entries for evaluation purposes)
            let sample_path = output.join("sample_entries.json");
            let sample_entries: Vec<_> = result.journal_entries.iter().take(10000).collect();
            let json = serde_json::to_string_pretty(&sample_entries)?;
            std::fs::write(&sample_path, json)?;
            tracing::info!("Sample entries written to: {} ({} entries)", sample_path.display(), sample_entries.len());

            Ok(())
        }

        Commands::Validate { config } => {
            let content = std::fs::read_to_string(&config)?;
            let generator_config: GeneratorConfig = serde_yaml::from_str(&content)?;
            synth_config::validate_config(&generator_config)?;
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
                synth_config::TransactionVolume::HundredK,
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
            Ok(())
        }
    }
}
