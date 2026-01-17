//! Enhanced generation orchestrator with full feature integration.
//!
//! This orchestrator coordinates all generation phases:
//! 1. Chart of Accounts generation
//! 2. Master data generation (vendors, customers, materials, assets, employees)
//! 3. Document flow generation (P2P, O2C)
//! 4. Journal entry generation
//! 5. Anomaly injection

use std::collections::HashMap;
use std::sync::Arc;

/// Get current process memory usage in MB.
///
/// Returns `None` if memory tracking is not available on the current platform.
#[cfg(target_os = "linux")]
fn get_memory_usage_mb() -> Option<usize> {
    use std::fs;
    // Read /proc/self/statm - format: size resident shared text lib data dt
    // We want 'resident' (2nd field) which is memory pages, multiply by page size
    if let Ok(content) = fs::read_to_string("/proc/self/statm") {
        let parts: Vec<&str> = content.split_whitespace().collect();
        if parts.len() >= 2 {
            if let Ok(pages) = parts[1].parse::<usize>() {
                // Page size is typically 4KB on Linux
                let page_size_kb = 4;
                return Some((pages * page_size_kb) / 1024);
            }
        }
    }
    None
}

#[cfg(not(target_os = "linux"))]
fn get_memory_usage_mb() -> Option<usize> {
    // Memory tracking not available on this platform
    None
}

use chrono::{Datelike, NaiveDate};
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use tracing::{debug, info, warn};

use synth_config::schema::GeneratorConfig;
use synth_core::error::{SynthError, SynthResult};
use synth_core::models::subledger::ap::APInvoice;
use synth_core::models::subledger::ar::ARInvoice;
use synth_core::models::*;
use synth_generators::{
    // Anomaly injection
    AnomalyInjector,
    AnomalyInjectorConfig,
    AssetGenerator,
    BalanceTrackerConfig,
    // Core generators
    ChartOfAccountsGenerator,
    CustomerGenerator,
    DataQualityConfig,
    // Data quality
    DataQualityInjector,
    DataQualityStats,
    // Document flow JE generator
    DocumentFlowJeGenerator,
    // Subledger linker
    DocumentFlowLinker,
    EmployeeGenerator,
    JournalEntryGenerator,
    MaterialGenerator,
    O2CDocumentChain,
    O2CGenerator,
    P2PDocumentChain,
    // Document flow generators
    P2PGenerator,
    // Balance validation
    RunningBalanceTracker,
    ValidationError,
    // Master data generators
    VendorGenerator,
};

/// Configuration for which generation phases to run.
#[derive(Debug, Clone)]
pub struct PhaseConfig {
    /// Generate master data (vendors, customers, materials, assets, employees).
    pub generate_master_data: bool,
    /// Generate document flows (P2P, O2C).
    pub generate_document_flows: bool,
    /// Generate journal entries.
    pub generate_journal_entries: bool,
    /// Inject anomalies.
    pub inject_anomalies: bool,
    /// Inject data quality variations (typos, missing values, format variations).
    pub inject_data_quality: bool,
    /// Validate balance sheet equation after generation.
    pub validate_balances: bool,
    /// Show progress bars.
    pub show_progress: bool,
    /// Number of vendors to generate per company.
    pub vendors_per_company: usize,
    /// Number of customers to generate per company.
    pub customers_per_company: usize,
    /// Number of materials to generate per company.
    pub materials_per_company: usize,
    /// Number of assets to generate per company.
    pub assets_per_company: usize,
    /// Number of employees to generate per company.
    pub employees_per_company: usize,
    /// Number of P2P chains to generate.
    pub p2p_chains: usize,
    /// Number of O2C chains to generate.
    pub o2c_chains: usize,
}

impl Default for PhaseConfig {
    fn default() -> Self {
        Self {
            generate_master_data: true,
            generate_document_flows: true,
            generate_journal_entries: true,
            inject_anomalies: false,
            inject_data_quality: false, // Off by default (to preserve clean test data)
            validate_balances: true,
            show_progress: true,
            vendors_per_company: 50,
            customers_per_company: 100,
            materials_per_company: 200,
            assets_per_company: 50,
            employees_per_company: 100,
            p2p_chains: 100,
            o2c_chains: 100,
        }
    }
}

/// Master data snapshot containing all generated entities.
#[derive(Debug, Clone, Default)]
pub struct MasterDataSnapshot {
    /// Generated vendors.
    pub vendors: Vec<Vendor>,
    /// Generated customers.
    pub customers: Vec<Customer>,
    /// Generated materials.
    pub materials: Vec<Material>,
    /// Generated fixed assets.
    pub assets: Vec<FixedAsset>,
    /// Generated employees.
    pub employees: Vec<Employee>,
}

/// Document flow snapshot containing all generated document chains.
#[derive(Debug, Clone, Default)]
pub struct DocumentFlowSnapshot {
    /// P2P document chains.
    pub p2p_chains: Vec<P2PDocumentChain>,
    /// O2C document chains.
    pub o2c_chains: Vec<O2CDocumentChain>,
    /// All purchase orders (flattened).
    pub purchase_orders: Vec<documents::PurchaseOrder>,
    /// All goods receipts (flattened).
    pub goods_receipts: Vec<documents::GoodsReceipt>,
    /// All vendor invoices (flattened).
    pub vendor_invoices: Vec<documents::VendorInvoice>,
    /// All sales orders (flattened).
    pub sales_orders: Vec<documents::SalesOrder>,
    /// All deliveries (flattened).
    pub deliveries: Vec<documents::Delivery>,
    /// All customer invoices (flattened).
    pub customer_invoices: Vec<documents::CustomerInvoice>,
    /// All payments (flattened).
    pub payments: Vec<documents::Payment>,
}

/// Subledger snapshot containing generated subledger records.
#[derive(Debug, Clone, Default)]
pub struct SubledgerSnapshot {
    /// AP invoices linked from document flow vendor invoices.
    pub ap_invoices: Vec<APInvoice>,
    /// AR invoices linked from document flow customer invoices.
    pub ar_invoices: Vec<ARInvoice>,
}

/// Anomaly labels generated during injection.
#[derive(Debug, Clone, Default)]
pub struct AnomalyLabels {
    /// All anomaly labels.
    pub labels: Vec<LabeledAnomaly>,
    /// Summary statistics.
    pub summary: Option<AnomalySummary>,
    /// Count by anomaly type.
    pub by_type: HashMap<String, usize>,
}

/// Balance validation results from running balance tracker.
#[derive(Debug, Clone, Default)]
pub struct BalanceValidationResult {
    /// Whether validation was performed.
    pub validated: bool,
    /// Whether balance sheet equation is satisfied.
    pub is_balanced: bool,
    /// Number of entries processed.
    pub entries_processed: u64,
    /// Total debits across all entries.
    pub total_debits: rust_decimal::Decimal,
    /// Total credits across all entries.
    pub total_credits: rust_decimal::Decimal,
    /// Number of accounts tracked.
    pub accounts_tracked: usize,
    /// Number of companies tracked.
    pub companies_tracked: usize,
    /// Validation errors encountered.
    pub validation_errors: Vec<ValidationError>,
    /// Whether any unbalanced entries were found.
    pub has_unbalanced_entries: bool,
}

/// Complete result of enhanced generation run.
#[derive(Debug)]
pub struct EnhancedGenerationResult {
    /// Generated chart of accounts.
    pub chart_of_accounts: ChartOfAccounts,
    /// Master data snapshot.
    pub master_data: MasterDataSnapshot,
    /// Document flow snapshot.
    pub document_flows: DocumentFlowSnapshot,
    /// Subledger snapshot (linked from document flows).
    pub subledger: SubledgerSnapshot,
    /// Generated journal entries.
    pub journal_entries: Vec<JournalEntry>,
    /// Anomaly labels (if injection enabled).
    pub anomaly_labels: AnomalyLabels,
    /// Balance validation results (if validation enabled).
    pub balance_validation: BalanceValidationResult,
    /// Data quality statistics (if injection enabled).
    pub data_quality_stats: DataQualityStats,
    /// Generation statistics.
    pub statistics: EnhancedGenerationStatistics,
}

/// Enhanced statistics about a generation run.
#[derive(Debug, Clone, Default)]
pub struct EnhancedGenerationStatistics {
    /// Total journal entries generated.
    pub total_entries: u64,
    /// Total line items generated.
    pub total_line_items: u64,
    /// Number of accounts in CoA.
    pub accounts_count: usize,
    /// Number of companies.
    pub companies_count: usize,
    /// Period in months.
    pub period_months: u32,
    /// Master data counts.
    pub vendor_count: usize,
    pub customer_count: usize,
    pub material_count: usize,
    pub asset_count: usize,
    pub employee_count: usize,
    /// Document flow counts.
    pub p2p_chain_count: usize,
    pub o2c_chain_count: usize,
    /// Subledger counts.
    pub ap_invoice_count: usize,
    pub ar_invoice_count: usize,
    /// Anomaly counts.
    pub anomalies_injected: usize,
    /// Data quality issue counts.
    pub data_quality_issues: usize,
}

/// Enhanced orchestrator with full feature integration.
pub struct EnhancedOrchestrator {
    config: GeneratorConfig,
    phase_config: PhaseConfig,
    coa: Option<Arc<ChartOfAccounts>>,
    master_data: MasterDataSnapshot,
    seed: u64,
    multi_progress: Option<MultiProgress>,
}

impl EnhancedOrchestrator {
    /// Create a new enhanced orchestrator.
    pub fn new(config: GeneratorConfig, phase_config: PhaseConfig) -> SynthResult<Self> {
        synth_config::validate_config(&config)?;

        let seed = config.global.seed.unwrap_or_else(rand::random);

        Ok(Self {
            config,
            phase_config,
            coa: None,
            master_data: MasterDataSnapshot::default(),
            seed,
            multi_progress: None,
        })
    }

    /// Create with default phase config.
    pub fn with_defaults(config: GeneratorConfig) -> SynthResult<Self> {
        Self::new(config, PhaseConfig::default())
    }

    /// Enable/disable progress bars.
    pub fn with_progress(mut self, show: bool) -> Self {
        self.phase_config.show_progress = show;
        if show {
            self.multi_progress = Some(MultiProgress::new());
        }
        self
    }

    /// Check if memory limit is exceeded.
    ///
    /// Returns an error if the configured memory limit is exceeded.
    /// Returns Ok(()) if:
    /// - Memory limit is 0 (disabled)
    /// - Memory tracking is not available on this platform
    /// - Current memory usage is within limits
    fn check_memory_limit(&self) -> SynthResult<()> {
        let limit_mb = self.config.global.memory_limit_mb;
        if limit_mb == 0 {
            return Ok(()); // Memory limit disabled
        }

        if let Some(current_mb) = get_memory_usage_mb() {
            if current_mb > limit_mb {
                return Err(SynthError::resource(format!(
                    "Memory limit exceeded: using {} MB, limit is {} MB. \
                     Reduce transaction volume or increase memory_limit_mb in config.",
                    current_mb, limit_mb
                )));
            }
        }
        Ok(())
    }

    /// Run the complete generation workflow.
    #[allow(clippy::field_reassign_with_default)]
    pub fn generate(&mut self) -> SynthResult<EnhancedGenerationResult> {
        info!("Starting enhanced generation workflow");
        info!(
            "Config: industry={:?}, period_months={}, companies={}",
            self.config.global.industry,
            self.config.global.period_months,
            self.config.companies.len()
        );

        // Initial memory check before starting
        self.check_memory_limit()?;

        let mut stats = EnhancedGenerationStatistics::default();
        stats.companies_count = self.config.companies.len();
        stats.period_months = self.config.global.period_months;

        // Phase 1: Generate Chart of Accounts
        info!("Phase 1: Generating Chart of Accounts");
        let coa = self.generate_coa()?;
        stats.accounts_count = coa.account_count();
        info!(
            "Chart of Accounts generated: {} accounts",
            stats.accounts_count
        );

        // Check memory after CoA generation
        self.check_memory_limit()?;

        // Phase 2: Generate Master Data
        if self.phase_config.generate_master_data {
            info!("Phase 2: Generating Master Data");
            self.generate_master_data()?;
            stats.vendor_count = self.master_data.vendors.len();
            stats.customer_count = self.master_data.customers.len();
            stats.material_count = self.master_data.materials.len();
            stats.asset_count = self.master_data.assets.len();
            stats.employee_count = self.master_data.employees.len();
            info!(
                "Master data generated: {} vendors, {} customers, {} materials, {} assets, {} employees",
                stats.vendor_count, stats.customer_count, stats.material_count,
                stats.asset_count, stats.employee_count
            );

            // Check memory after master data generation
            self.check_memory_limit()?;
        } else {
            debug!("Phase 2: Skipped (master data generation disabled)");
        }

        // Phase 3: Generate Document Flows
        let mut document_flows = DocumentFlowSnapshot::default();
        let mut subledger = SubledgerSnapshot::default();
        if self.phase_config.generate_document_flows && !self.master_data.vendors.is_empty() {
            info!("Phase 3: Generating Document Flows");
            self.generate_document_flows(&mut document_flows)?;
            stats.p2p_chain_count = document_flows.p2p_chains.len();
            stats.o2c_chain_count = document_flows.o2c_chains.len();
            info!(
                "Document flows generated: {} P2P chains, {} O2C chains",
                stats.p2p_chain_count, stats.o2c_chain_count
            );

            // Phase 3b: Link document flows to subledgers (for data coherence)
            debug!("Phase 3b: Linking document flows to subledgers");
            subledger = self.link_document_flows_to_subledgers(&document_flows)?;
            stats.ap_invoice_count = subledger.ap_invoices.len();
            stats.ar_invoice_count = subledger.ar_invoices.len();
            debug!(
                "Subledgers linked: {} AP invoices, {} AR invoices",
                stats.ap_invoice_count, stats.ar_invoice_count
            );

            // Check memory after document flow generation
            self.check_memory_limit()?;
        } else {
            debug!("Phase 3: Skipped (document flow generation disabled or no master data)");
        }

        // Phase 4: Generate Journal Entries
        let mut entries = Vec::new();

        // Phase 4a: Generate JEs from document flows (for data coherence)
        if self.phase_config.generate_document_flows && !document_flows.p2p_chains.is_empty() {
            debug!("Phase 4a: Generating JEs from document flows");
            let flow_entries = self.generate_jes_from_document_flows(&document_flows)?;
            debug!("Generated {} JEs from document flows", flow_entries.len());
            entries.extend(flow_entries);
        }

        // Phase 4b: Generate standalone journal entries
        if self.phase_config.generate_journal_entries {
            info!("Phase 4: Generating Journal Entries");
            let je_entries = self.generate_journal_entries(&coa)?;
            info!("Generated {} standalone journal entries", je_entries.len());
            entries.extend(je_entries);
        } else {
            debug!("Phase 4: Skipped (journal entry generation disabled)");
        }

        if !entries.is_empty() {
            stats.total_entries = entries.len() as u64;
            stats.total_line_items = entries.iter().map(|e| e.line_count() as u64).sum();
            info!(
                "Total entries: {}, total line items: {}",
                stats.total_entries, stats.total_line_items
            );
        }

        // Phase 5: Inject Anomalies
        let mut anomaly_labels = AnomalyLabels::default();
        if self.phase_config.inject_anomalies && !entries.is_empty() {
            info!("Phase 5: Injecting Anomalies");
            let result = self.inject_anomalies(&mut entries)?;
            stats.anomalies_injected = result.labels.len();
            anomaly_labels = result;
            info!("Injected {} anomalies", stats.anomalies_injected);
        } else {
            debug!("Phase 5: Skipped (anomaly injection disabled or no entries)");
        }

        // Phase 6: Validate Balances
        let mut balance_validation = BalanceValidationResult::default();
        if self.phase_config.validate_balances && !entries.is_empty() {
            debug!("Phase 6: Validating Balances");
            balance_validation = self.validate_journal_entries(&entries)?;
            if balance_validation.is_balanced {
                debug!("Balance validation passed");
            } else {
                warn!(
                    "Balance validation found {} errors",
                    balance_validation.validation_errors.len()
                );
            }
        }

        // Phase 7: Inject Data Quality Variations
        let mut data_quality_stats = DataQualityStats::default();
        if self.phase_config.inject_data_quality && !entries.is_empty() {
            info!("Phase 7: Injecting Data Quality Variations");
            data_quality_stats = self.inject_data_quality(&mut entries)?;
            stats.data_quality_issues = data_quality_stats.records_with_issues;
            info!("Injected {} data quality issues", stats.data_quality_issues);
        } else {
            debug!("Phase 7: Skipped (data quality injection disabled or no entries)");
        }

        info!("Generation workflow complete");

        Ok(EnhancedGenerationResult {
            chart_of_accounts: (*coa).clone(),
            master_data: self.master_data.clone(),
            document_flows,
            subledger,
            journal_entries: entries,
            anomaly_labels,
            balance_validation,
            data_quality_stats,
            statistics: stats,
        })
    }

    /// Generate the chart of accounts.
    fn generate_coa(&mut self) -> SynthResult<Arc<ChartOfAccounts>> {
        let pb = self.create_progress_bar(1, "Generating Chart of Accounts");

        let mut gen = ChartOfAccountsGenerator::new(
            self.config.chart_of_accounts.complexity,
            self.config.global.industry,
            self.seed,
        );

        let coa = Arc::new(gen.generate());
        self.coa = Some(Arc::clone(&coa));

        if let Some(pb) = pb {
            pb.finish_with_message("Chart of Accounts complete");
        }

        Ok(coa)
    }

    /// Generate master data entities.
    fn generate_master_data(&mut self) -> SynthResult<()> {
        let start_date = NaiveDate::parse_from_str(&self.config.global.start_date, "%Y-%m-%d")
            .map_err(|e| SynthError::config(format!("Invalid start_date: {}", e)))?;
        let end_date = start_date + chrono::Months::new(self.config.global.period_months);

        let total = self.config.companies.len() as u64 * 5; // 5 entity types
        let pb = self.create_progress_bar(total, "Generating Master Data");

        for (i, company) in self.config.companies.iter().enumerate() {
            let company_seed = self.seed.wrapping_add(i as u64 * 1000);

            // Generate vendors
            let mut vendor_gen = VendorGenerator::new(company_seed);
            let vendor_pool = vendor_gen.generate_vendor_pool(
                self.phase_config.vendors_per_company,
                &company.code,
                start_date,
            );
            self.master_data.vendors.extend(vendor_pool.vendors);
            if let Some(pb) = &pb {
                pb.inc(1);
            }

            // Generate customers
            let mut customer_gen = CustomerGenerator::new(company_seed + 100);
            let customer_pool = customer_gen.generate_customer_pool(
                self.phase_config.customers_per_company,
                &company.code,
                start_date,
            );
            self.master_data.customers.extend(customer_pool.customers);
            if let Some(pb) = &pb {
                pb.inc(1);
            }

            // Generate materials
            let mut material_gen = MaterialGenerator::new(company_seed + 200);
            let material_pool = material_gen.generate_material_pool(
                self.phase_config.materials_per_company,
                &company.code,
                start_date,
            );
            self.master_data.materials.extend(material_pool.materials);
            if let Some(pb) = &pb {
                pb.inc(1);
            }

            // Generate fixed assets
            let mut asset_gen = AssetGenerator::new(company_seed + 300);
            let asset_pool = asset_gen.generate_asset_pool(
                self.phase_config.assets_per_company,
                &company.code,
                (start_date, end_date),
            );
            self.master_data.assets.extend(asset_pool.assets);
            if let Some(pb) = &pb {
                pb.inc(1);
            }

            // Generate employees
            let mut employee_gen = EmployeeGenerator::new(company_seed + 400);
            let employee_pool =
                employee_gen.generate_company_pool(&company.code, (start_date, end_date));
            self.master_data.employees.extend(employee_pool.employees);
            if let Some(pb) = &pb {
                pb.inc(1);
            }
        }

        if let Some(pb) = pb {
            pb.finish_with_message("Master data generation complete");
        }

        Ok(())
    }

    /// Generate document flows (P2P and O2C).
    fn generate_document_flows(&mut self, flows: &mut DocumentFlowSnapshot) -> SynthResult<()> {
        let start_date = NaiveDate::parse_from_str(&self.config.global.start_date, "%Y-%m-%d")
            .map_err(|e| SynthError::config(format!("Invalid start_date: {}", e)))?;

        // Generate P2P chains
        let p2p_count = self
            .phase_config
            .p2p_chains
            .min(self.master_data.vendors.len() * 2);
        let pb = self.create_progress_bar(p2p_count as u64, "Generating P2P Document Flows");

        let mut p2p_gen = P2PGenerator::new(self.seed + 1000);

        for i in 0..p2p_count {
            let vendor = &self.master_data.vendors[i % self.master_data.vendors.len()];
            let materials: Vec<&Material> = self
                .master_data
                .materials
                .iter()
                .skip(i % self.master_data.materials.len().max(1))
                .take(2.min(self.master_data.materials.len()))
                .collect();

            if materials.is_empty() {
                continue;
            }

            let company = &self.config.companies[i % self.config.companies.len()];
            let po_date = start_date + chrono::Duration::days((i * 3) as i64 % 365);
            let fiscal_period = po_date.month() as u8;
            let created_by = self
                .master_data
                .employees
                .first()
                .map(|e| e.user_id.as_str())
                .unwrap_or("SYSTEM");

            let chain = p2p_gen.generate_chain(
                &company.code,
                vendor,
                &materials,
                po_date,
                start_date.year() as u16,
                fiscal_period,
                created_by,
            );

            // Flatten documents
            flows.purchase_orders.push(chain.purchase_order.clone());
            flows.goods_receipts.extend(chain.goods_receipts.clone());
            if let Some(vi) = &chain.vendor_invoice {
                flows.vendor_invoices.push(vi.clone());
            }
            if let Some(payment) = &chain.payment {
                flows.payments.push(payment.clone());
            }
            flows.p2p_chains.push(chain);

            if let Some(pb) = &pb {
                pb.inc(1);
            }
        }

        if let Some(pb) = pb {
            pb.finish_with_message("P2P document flows complete");
        }

        // Generate O2C chains
        let o2c_count = self
            .phase_config
            .o2c_chains
            .min(self.master_data.customers.len() * 2);
        let pb = self.create_progress_bar(o2c_count as u64, "Generating O2C Document Flows");

        let mut o2c_gen = O2CGenerator::new(self.seed + 2000);

        for i in 0..o2c_count {
            let customer = &self.master_data.customers[i % self.master_data.customers.len()];
            let materials: Vec<&Material> = self
                .master_data
                .materials
                .iter()
                .skip(i % self.master_data.materials.len().max(1))
                .take(2.min(self.master_data.materials.len()))
                .collect();

            if materials.is_empty() {
                continue;
            }

            let company = &self.config.companies[i % self.config.companies.len()];
            let so_date = start_date + chrono::Duration::days((i * 2) as i64 % 365);
            let fiscal_period = so_date.month() as u8;
            let created_by = self
                .master_data
                .employees
                .first()
                .map(|e| e.user_id.as_str())
                .unwrap_or("SYSTEM");

            let chain = o2c_gen.generate_chain(
                &company.code,
                customer,
                &materials,
                so_date,
                start_date.year() as u16,
                fiscal_period,
                created_by,
            );

            // Flatten documents
            flows.sales_orders.push(chain.sales_order.clone());
            flows.deliveries.extend(chain.deliveries.clone());
            if let Some(ci) = &chain.customer_invoice {
                flows.customer_invoices.push(ci.clone());
            }
            if let Some(receipt) = &chain.customer_receipt {
                flows.payments.push(receipt.clone());
            }
            flows.o2c_chains.push(chain);

            if let Some(pb) = &pb {
                pb.inc(1);
            }
        }

        if let Some(pb) = pb {
            pb.finish_with_message("O2C document flows complete");
        }

        Ok(())
    }

    /// Generate journal entries.
    fn generate_journal_entries(
        &mut self,
        coa: &Arc<ChartOfAccounts>,
    ) -> SynthResult<Vec<JournalEntry>> {
        let total = self.calculate_total_transactions();
        let pb = self.create_progress_bar(total, "Generating Journal Entries");

        let start_date = NaiveDate::parse_from_str(&self.config.global.start_date, "%Y-%m-%d")
            .map_err(|e| SynthError::config(format!("Invalid start_date: {}", e)))?;
        let end_date = start_date + chrono::Months::new(self.config.global.period_months);

        let company_codes: Vec<String> = self
            .config
            .companies
            .iter()
            .map(|c| c.code.clone())
            .collect();

        let generator = JournalEntryGenerator::new_with_params(
            self.config.transactions.clone(),
            Arc::clone(coa),
            company_codes,
            start_date,
            end_date,
            self.seed,
        );

        // Connect generated master data to ensure JEs reference real entities
        // Enable persona-based error injection for realistic human behavior
        let mut generator = generator
            .with_master_data(
                &self.master_data.vendors,
                &self.master_data.customers,
                &self.master_data.materials,
            )
            .with_persona_errors(true);

        let mut entries = Vec::with_capacity(total as usize);

        // Check memory limit at start
        self.check_memory_limit()?;

        // Check every 1000 entries to avoid overhead
        const MEMORY_CHECK_INTERVAL: u64 = 1000;

        for i in 0..total {
            let entry = generator.generate();
            entries.push(entry);
            if let Some(pb) = &pb {
                pb.inc(1);
            }

            // Periodic memory limit check
            if (i + 1) % MEMORY_CHECK_INTERVAL == 0 {
                self.check_memory_limit()?;
            }
        }

        if let Some(pb) = pb {
            pb.finish_with_message("Journal entries complete");
        }

        Ok(entries)
    }

    /// Generate journal entries from document flows.
    ///
    /// This creates proper GL entries for each document in the P2P and O2C flows,
    /// ensuring that document activity is reflected in the general ledger.
    fn generate_jes_from_document_flows(
        &mut self,
        flows: &DocumentFlowSnapshot,
    ) -> SynthResult<Vec<JournalEntry>> {
        let total_chains = flows.p2p_chains.len() + flows.o2c_chains.len();
        let pb = self.create_progress_bar(total_chains as u64, "Generating Document Flow JEs");

        let mut generator = DocumentFlowJeGenerator::new();
        let mut entries = Vec::new();

        // Generate JEs from P2P chains
        for chain in &flows.p2p_chains {
            let chain_entries = generator.generate_from_p2p_chain(chain);
            entries.extend(chain_entries);
            if let Some(pb) = &pb {
                pb.inc(1);
            }
        }

        // Generate JEs from O2C chains
        for chain in &flows.o2c_chains {
            let chain_entries = generator.generate_from_o2c_chain(chain);
            entries.extend(chain_entries);
            if let Some(pb) = &pb {
                pb.inc(1);
            }
        }

        if let Some(pb) = pb {
            pb.finish_with_message(format!(
                "Generated {} JEs from document flows",
                entries.len()
            ));
        }

        Ok(entries)
    }

    /// Link document flows to subledger records.
    ///
    /// Creates AP invoices from vendor invoices and AR invoices from customer invoices,
    /// ensuring subledger data is coherent with document flow data.
    fn link_document_flows_to_subledgers(
        &mut self,
        flows: &DocumentFlowSnapshot,
    ) -> SynthResult<SubledgerSnapshot> {
        let total = flows.vendor_invoices.len() + flows.customer_invoices.len();
        let pb = self.create_progress_bar(total as u64, "Linking Subledgers");

        let mut linker = DocumentFlowLinker::new();

        // Convert vendor invoices to AP invoices
        let ap_invoices = linker.batch_create_ap_invoices(&flows.vendor_invoices);
        if let Some(pb) = &pb {
            pb.inc(flows.vendor_invoices.len() as u64);
        }

        // Convert customer invoices to AR invoices
        let ar_invoices = linker.batch_create_ar_invoices(&flows.customer_invoices);
        if let Some(pb) = &pb {
            pb.inc(flows.customer_invoices.len() as u64);
        }

        if let Some(pb) = pb {
            pb.finish_with_message(format!(
                "Linked {} AP and {} AR invoices",
                ap_invoices.len(),
                ar_invoices.len()
            ));
        }

        Ok(SubledgerSnapshot {
            ap_invoices,
            ar_invoices,
        })
    }

    /// Inject anomalies into journal entries.
    fn inject_anomalies(&mut self, entries: &mut [JournalEntry]) -> SynthResult<AnomalyLabels> {
        let pb = self.create_progress_bar(entries.len() as u64, "Injecting Anomalies");

        let anomaly_config = AnomalyInjectorConfig {
            rates: AnomalyRateConfig {
                total_rate: 0.02,
                ..Default::default()
            },
            seed: self.seed + 5000,
            ..Default::default()
        };

        let mut injector = AnomalyInjector::new(anomaly_config);
        let result = injector.process_entries(entries);

        if let Some(pb) = &pb {
            pb.inc(entries.len() as u64);
            pb.finish_with_message("Anomaly injection complete");
        }

        let mut by_type = HashMap::new();
        for label in &result.labels {
            *by_type
                .entry(format!("{:?}", label.anomaly_type))
                .or_insert(0) += 1;
        }

        Ok(AnomalyLabels {
            labels: result.labels,
            summary: Some(result.summary),
            by_type,
        })
    }

    /// Validate journal entries using running balance tracker.
    ///
    /// Applies all entries to the balance tracker and validates:
    /// - Each entry is internally balanced (debits = credits)
    /// - Balance sheet equation holds (Assets = Liabilities + Equity + Net Income)
    ///
    /// Note: Entries with human errors (marked with [HUMAN_ERROR:*] tags) are
    /// excluded from balance validation as they may be intentionally unbalanced.
    fn validate_journal_entries(
        &mut self,
        entries: &[JournalEntry],
    ) -> SynthResult<BalanceValidationResult> {
        // Filter out entries with human errors as they may be intentionally unbalanced
        let clean_entries: Vec<&JournalEntry> = entries
            .iter()
            .filter(|e| {
                e.header
                    .header_text
                    .as_ref()
                    .map(|t| !t.contains("[HUMAN_ERROR:"))
                    .unwrap_or(true)
            })
            .collect();

        let pb = self.create_progress_bar(clean_entries.len() as u64, "Validating Balances");

        // Configure tracker to not fail on errors (collect them instead)
        let config = BalanceTrackerConfig {
            validate_on_each_entry: false,   // We'll validate at the end
            track_history: false,            // Skip history for performance
            fail_on_validation_error: false, // Collect errors, don't fail
            ..Default::default()
        };

        let mut tracker = RunningBalanceTracker::new(config);

        // Apply clean entries (without human errors)
        let clean_refs: Vec<JournalEntry> = clean_entries.into_iter().cloned().collect();
        let errors = tracker.apply_entries(&clean_refs);

        if let Some(pb) = &pb {
            pb.inc(entries.len() as u64);
        }

        // Check if any entries were unbalanced
        let has_unbalanced = errors
            .iter()
            .any(|e| e.error_type == synth_generators::ValidationErrorType::UnbalancedEntry);

        // Validate balance sheet for each company
        let mut all_errors = errors;
        let company_codes: Vec<String> = self
            .config
            .companies
            .iter()
            .map(|c| c.code.clone())
            .collect();

        let end_date = NaiveDate::parse_from_str(&self.config.global.start_date, "%Y-%m-%d")
            .map(|d| d + chrono::Months::new(self.config.global.period_months))
            .unwrap_or_else(|_| chrono::Local::now().date_naive());

        for company_code in &company_codes {
            if let Err(e) = tracker.validate_balance_sheet(company_code, end_date, None) {
                all_errors.push(e);
            }
        }

        // Get statistics after all mutable operations are done
        let stats = tracker.get_statistics();

        // Determine if balanced overall
        let is_balanced = all_errors.is_empty();

        if let Some(pb) = pb {
            let msg = if is_balanced {
                "Balance validation passed"
            } else {
                "Balance validation completed with errors"
            };
            pb.finish_with_message(msg);
        }

        Ok(BalanceValidationResult {
            validated: true,
            is_balanced,
            entries_processed: stats.entries_processed,
            total_debits: stats.total_debits,
            total_credits: stats.total_credits,
            accounts_tracked: stats.accounts_tracked,
            companies_tracked: stats.companies_tracked,
            validation_errors: all_errors,
            has_unbalanced_entries: has_unbalanced,
        })
    }

    /// Inject data quality variations into journal entries.
    ///
    /// Applies typos, missing values, and format variations to make
    /// the synthetic data more realistic for testing data cleaning pipelines.
    fn inject_data_quality(
        &mut self,
        entries: &mut [JournalEntry],
    ) -> SynthResult<DataQualityStats> {
        let pb = self.create_progress_bar(entries.len() as u64, "Injecting Data Quality Issues");

        // Use minimal configuration by default for realistic but not overwhelming issues
        let config = DataQualityConfig::minimal();
        let mut injector = DataQualityInjector::new(config);

        // Build context for missing value decisions
        let context = HashMap::new();

        for entry in entries.iter_mut() {
            // Process header_text field (common target for typos)
            if let Some(text) = &entry.header.header_text {
                let processed = injector.process_text_field(
                    "header_text",
                    text,
                    &entry.header.document_id.to_string(),
                    &context,
                );
                match processed {
                    Some(new_text) if new_text != *text => {
                        entry.header.header_text = Some(new_text);
                    }
                    None => {
                        entry.header.header_text = None; // Missing value
                    }
                    _ => {}
                }
            }

            // Process reference field
            if let Some(ref_text) = &entry.header.reference {
                let processed = injector.process_text_field(
                    "reference",
                    ref_text,
                    &entry.header.document_id.to_string(),
                    &context,
                );
                match processed {
                    Some(new_text) if new_text != *ref_text => {
                        entry.header.reference = Some(new_text);
                    }
                    None => {
                        entry.header.reference = None;
                    }
                    _ => {}
                }
            }

            // Process user_persona field (potential for typos in user IDs)
            let user_persona = entry.header.user_persona.clone();
            if let Some(processed) = injector.process_text_field(
                "user_persona",
                &user_persona,
                &entry.header.document_id.to_string(),
                &context,
            ) {
                if processed != user_persona {
                    entry.header.user_persona = processed;
                }
            }

            // Process line items
            for line in &mut entry.lines {
                // Process line description if present
                if let Some(ref text) = line.line_text {
                    let processed = injector.process_text_field(
                        "line_text",
                        text,
                        &entry.header.document_id.to_string(),
                        &context,
                    );
                    match processed {
                        Some(new_text) if new_text != *text => {
                            line.line_text = Some(new_text);
                        }
                        None => {
                            line.line_text = None;
                        }
                        _ => {}
                    }
                }

                // Process cost_center if present
                if let Some(cc) = &line.cost_center {
                    let processed = injector.process_text_field(
                        "cost_center",
                        cc,
                        &entry.header.document_id.to_string(),
                        &context,
                    );
                    match processed {
                        Some(new_cc) if new_cc != *cc => {
                            line.cost_center = Some(new_cc);
                        }
                        None => {
                            line.cost_center = None;
                        }
                        _ => {}
                    }
                }
            }

            if let Some(pb) = &pb {
                pb.inc(1);
            }
        }

        if let Some(pb) = pb {
            pb.finish_with_message("Data quality injection complete");
        }

        Ok(injector.stats().clone())
    }

    /// Calculate total transactions to generate.
    fn calculate_total_transactions(&self) -> u64 {
        let months = self.config.global.period_months as f64;
        self.config
            .companies
            .iter()
            .map(|c| {
                let annual = c.annual_transaction_volume.count() as f64;
                let weighted = annual * c.volume_weight;
                (weighted * months / 12.0) as u64
            })
            .sum()
    }

    /// Create a progress bar if progress display is enabled.
    fn create_progress_bar(&self, total: u64, message: &str) -> Option<ProgressBar> {
        if !self.phase_config.show_progress {
            return None;
        }

        let pb = if let Some(mp) = &self.multi_progress {
            mp.add(ProgressBar::new(total))
        } else {
            ProgressBar::new(total)
        };

        pb.set_style(
            ProgressStyle::default_bar()
                .template(&format!(
                    "{{spinner:.green}} {} [{{elapsed_precise}}] [{{bar:40.cyan/blue}}] {{pos}}/{{len}} ({{per_sec}})",
                    message
                ))
                .expect("Progress bar template should be valid - uses only standard indicatif placeholders")
                .progress_chars("#>-"),
        );

        Some(pb)
    }

    /// Get the generated chart of accounts.
    pub fn get_coa(&self) -> Option<Arc<ChartOfAccounts>> {
        self.coa.clone()
    }

    /// Get the generated master data.
    pub fn get_master_data(&self) -> &MasterDataSnapshot {
        &self.master_data
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use synth_config::schema::*;

    fn create_test_config() -> GeneratorConfig {
        GeneratorConfig {
            global: GlobalConfig {
                industry: IndustrySector::Manufacturing,
                start_date: "2024-01-01".to_string(),
                period_months: 1,
                seed: Some(42),
                parallel: false,
                group_currency: "USD".to_string(),
                worker_threads: 0,
                memory_limit_mb: 0,
            },
            companies: vec![CompanyConfig {
                code: "1000".to_string(),
                name: "Test Company".to_string(),
                currency: "USD".to_string(),
                country: "US".to_string(),
                annual_transaction_volume: TransactionVolume::TenK,
                volume_weight: 1.0,
                fiscal_year_variant: "K4".to_string(),
            }],
            chart_of_accounts: ChartOfAccountsConfig {
                complexity: CoAComplexity::Small,
                industry_specific: true,
                custom_accounts: None,
                min_hierarchy_depth: 2,
                max_hierarchy_depth: 4,
            },
            transactions: TransactionConfig::default(),
            output: OutputConfig::default(),
            fraud: FraudConfig::default(),
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
        }
    }

    #[test]
    fn test_enhanced_orchestrator_creation() {
        let config = create_test_config();
        let orchestrator = EnhancedOrchestrator::with_defaults(config);
        assert!(orchestrator.is_ok());
    }

    #[test]
    fn test_minimal_generation() {
        let config = create_test_config();
        let phase_config = PhaseConfig {
            generate_master_data: false,
            generate_document_flows: false,
            generate_journal_entries: true,
            inject_anomalies: false,
            show_progress: false,
            ..Default::default()
        };

        let mut orchestrator = EnhancedOrchestrator::new(config, phase_config).unwrap();
        let result = orchestrator.generate();

        assert!(result.is_ok());
        let result = result.unwrap();
        assert!(!result.journal_entries.is_empty());
    }

    #[test]
    fn test_master_data_generation() {
        let config = create_test_config();
        let phase_config = PhaseConfig {
            generate_master_data: true,
            generate_document_flows: false,
            generate_journal_entries: false,
            inject_anomalies: false,
            show_progress: false,
            vendors_per_company: 5,
            customers_per_company: 5,
            materials_per_company: 10,
            assets_per_company: 5,
            employees_per_company: 10,
            ..Default::default()
        };

        let mut orchestrator = EnhancedOrchestrator::new(config, phase_config).unwrap();
        let result = orchestrator.generate().unwrap();

        assert!(!result.master_data.vendors.is_empty());
        assert!(!result.master_data.customers.is_empty());
        assert!(!result.master_data.materials.is_empty());
    }

    #[test]
    fn test_document_flow_generation() {
        let config = create_test_config();
        let phase_config = PhaseConfig {
            generate_master_data: true,
            generate_document_flows: true,
            generate_journal_entries: false,
            inject_anomalies: false,
            inject_data_quality: false,
            validate_balances: false,
            show_progress: false,
            vendors_per_company: 5,
            customers_per_company: 5,
            materials_per_company: 10,
            assets_per_company: 5,
            employees_per_company: 10,
            p2p_chains: 5,
            o2c_chains: 5,
        };

        let mut orchestrator = EnhancedOrchestrator::new(config, phase_config).unwrap();
        let result = orchestrator.generate().unwrap();

        // Should have generated P2P and O2C chains
        assert!(!result.document_flows.p2p_chains.is_empty());
        assert!(!result.document_flows.o2c_chains.is_empty());

        // Flattened documents should be populated
        assert!(!result.document_flows.purchase_orders.is_empty());
        assert!(!result.document_flows.sales_orders.is_empty());
    }

    #[test]
    fn test_anomaly_injection() {
        let config = create_test_config();
        let phase_config = PhaseConfig {
            generate_master_data: false,
            generate_document_flows: false,
            generate_journal_entries: true,
            inject_anomalies: true,
            show_progress: false,
            ..Default::default()
        };

        let mut orchestrator = EnhancedOrchestrator::new(config, phase_config).unwrap();
        let result = orchestrator.generate().unwrap();

        // Should have journal entries
        assert!(!result.journal_entries.is_empty());

        // With ~833 entries and 2% rate, expect some anomalies
        // Note: This is probabilistic, so we just verify the structure exists
        assert!(result.anomaly_labels.summary.is_some());
    }

    #[test]
    fn test_full_generation_pipeline() {
        let config = create_test_config();
        let phase_config = PhaseConfig {
            generate_master_data: true,
            generate_document_flows: true,
            generate_journal_entries: true,
            inject_anomalies: false,
            inject_data_quality: false,
            validate_balances: true,
            show_progress: false,
            vendors_per_company: 3,
            customers_per_company: 3,
            materials_per_company: 5,
            assets_per_company: 3,
            employees_per_company: 5,
            p2p_chains: 3,
            o2c_chains: 3,
        };

        let mut orchestrator = EnhancedOrchestrator::new(config, phase_config).unwrap();
        let result = orchestrator.generate().unwrap();

        // All phases should have results
        assert!(!result.master_data.vendors.is_empty());
        assert!(!result.master_data.customers.is_empty());
        assert!(!result.document_flows.p2p_chains.is_empty());
        assert!(!result.document_flows.o2c_chains.is_empty());
        assert!(!result.journal_entries.is_empty());
        assert!(result.statistics.accounts_count > 0);

        // Subledger linking should have run
        assert!(!result.subledger.ap_invoices.is_empty());
        assert!(!result.subledger.ar_invoices.is_empty());

        // Balance validation should have run
        assert!(result.balance_validation.validated);
        assert!(result.balance_validation.entries_processed > 0);
    }

    #[test]
    fn test_subledger_linking() {
        let config = create_test_config();
        let phase_config = PhaseConfig {
            generate_master_data: true,
            generate_document_flows: true,
            generate_journal_entries: false,
            inject_anomalies: false,
            inject_data_quality: false,
            validate_balances: false,
            show_progress: false,
            vendors_per_company: 5,
            customers_per_company: 5,
            materials_per_company: 10,
            assets_per_company: 3,
            employees_per_company: 5,
            p2p_chains: 5,
            o2c_chains: 5,
        };

        let mut orchestrator = EnhancedOrchestrator::new(config, phase_config).unwrap();
        let result = orchestrator.generate().unwrap();

        // Should have document flows
        assert!(!result.document_flows.vendor_invoices.is_empty());
        assert!(!result.document_flows.customer_invoices.is_empty());

        // Subledger should be linked from document flows
        assert!(!result.subledger.ap_invoices.is_empty());
        assert!(!result.subledger.ar_invoices.is_empty());

        // AP invoices count should match vendor invoices count
        assert_eq!(
            result.subledger.ap_invoices.len(),
            result.document_flows.vendor_invoices.len()
        );

        // AR invoices count should match customer invoices count
        assert_eq!(
            result.subledger.ar_invoices.len(),
            result.document_flows.customer_invoices.len()
        );

        // Statistics should reflect subledger counts
        assert_eq!(
            result.statistics.ap_invoice_count,
            result.subledger.ap_invoices.len()
        );
        assert_eq!(
            result.statistics.ar_invoice_count,
            result.subledger.ar_invoices.len()
        );
    }

    #[test]
    fn test_balance_validation() {
        let config = create_test_config();
        let phase_config = PhaseConfig {
            generate_master_data: false,
            generate_document_flows: false,
            generate_journal_entries: true,
            inject_anomalies: false,
            validate_balances: true,
            show_progress: false,
            ..Default::default()
        };

        let mut orchestrator = EnhancedOrchestrator::new(config, phase_config).unwrap();
        let result = orchestrator.generate().unwrap();

        // Balance validation should run
        assert!(result.balance_validation.validated);
        assert!(result.balance_validation.entries_processed > 0);

        // Generated JEs should be balanced (no unbalanced entries)
        assert!(!result.balance_validation.has_unbalanced_entries);

        // Total debits should equal total credits
        assert_eq!(
            result.balance_validation.total_debits,
            result.balance_validation.total_credits
        );
    }

    #[test]
    fn test_statistics_accuracy() {
        let config = create_test_config();
        let phase_config = PhaseConfig {
            generate_master_data: true,
            generate_document_flows: false,
            generate_journal_entries: true,
            inject_anomalies: false,
            show_progress: false,
            vendors_per_company: 10,
            customers_per_company: 20,
            materials_per_company: 15,
            assets_per_company: 5,
            employees_per_company: 8,
            ..Default::default()
        };

        let mut orchestrator = EnhancedOrchestrator::new(config, phase_config).unwrap();
        let result = orchestrator.generate().unwrap();

        // Statistics should match actual data
        assert_eq!(
            result.statistics.vendor_count,
            result.master_data.vendors.len()
        );
        assert_eq!(
            result.statistics.customer_count,
            result.master_data.customers.len()
        );
        assert_eq!(
            result.statistics.material_count,
            result.master_data.materials.len()
        );
        assert_eq!(
            result.statistics.total_entries as usize,
            result.journal_entries.len()
        );
    }

    #[test]
    fn test_phase_config_defaults() {
        let config = PhaseConfig::default();
        assert!(config.generate_master_data);
        assert!(config.generate_document_flows);
        assert!(config.generate_journal_entries);
        assert!(!config.inject_anomalies);
        assert!(config.validate_balances);
        assert!(config.show_progress);
        assert!(config.vendors_per_company > 0);
        assert!(config.customers_per_company > 0);
    }

    #[test]
    fn test_get_coa_before_generation() {
        let config = create_test_config();
        let orchestrator = EnhancedOrchestrator::with_defaults(config).unwrap();

        // Before generation, CoA should be None
        assert!(orchestrator.get_coa().is_none());
    }

    #[test]
    fn test_get_coa_after_generation() {
        let config = create_test_config();
        let phase_config = PhaseConfig {
            generate_master_data: false,
            generate_document_flows: false,
            generate_journal_entries: true,
            inject_anomalies: false,
            show_progress: false,
            ..Default::default()
        };

        let mut orchestrator = EnhancedOrchestrator::new(config, phase_config).unwrap();
        let _ = orchestrator.generate().unwrap();

        // After generation, CoA should be available
        assert!(orchestrator.get_coa().is_some());
    }

    #[test]
    fn test_get_master_data() {
        let config = create_test_config();
        let phase_config = PhaseConfig {
            generate_master_data: true,
            generate_document_flows: false,
            generate_journal_entries: false,
            inject_anomalies: false,
            show_progress: false,
            vendors_per_company: 5,
            customers_per_company: 5,
            materials_per_company: 5,
            assets_per_company: 5,
            employees_per_company: 5,
            ..Default::default()
        };

        let mut orchestrator = EnhancedOrchestrator::new(config, phase_config).unwrap();
        let _ = orchestrator.generate().unwrap();

        let master_data = orchestrator.get_master_data();
        assert!(!master_data.vendors.is_empty());
    }

    #[test]
    fn test_with_progress_builder() {
        let config = create_test_config();
        let orchestrator = EnhancedOrchestrator::with_defaults(config)
            .unwrap()
            .with_progress(false);

        // Should still work without progress
        assert!(!orchestrator.phase_config.show_progress);
    }

    #[test]
    fn test_multi_company_generation() {
        let mut config = create_test_config();
        config.companies.push(CompanyConfig {
            code: "2000".to_string(),
            name: "Subsidiary".to_string(),
            currency: "EUR".to_string(),
            country: "DE".to_string(),
            annual_transaction_volume: TransactionVolume::TenK,
            volume_weight: 0.5,
            fiscal_year_variant: "K4".to_string(),
        });

        let phase_config = PhaseConfig {
            generate_master_data: true,
            generate_document_flows: false,
            generate_journal_entries: true,
            inject_anomalies: false,
            show_progress: false,
            vendors_per_company: 5,
            customers_per_company: 5,
            materials_per_company: 5,
            assets_per_company: 5,
            employees_per_company: 5,
            ..Default::default()
        };

        let mut orchestrator = EnhancedOrchestrator::new(config, phase_config).unwrap();
        let result = orchestrator.generate().unwrap();

        // Should have master data for both companies
        assert!(result.statistics.vendor_count >= 10); // 5 per company
        assert!(result.statistics.customer_count >= 10);
        assert!(result.statistics.companies_count == 2);
    }

    #[test]
    fn test_empty_master_data_skips_document_flows() {
        let config = create_test_config();
        let phase_config = PhaseConfig {
            generate_master_data: false,   // Skip master data
            generate_document_flows: true, // Try to generate flows
            generate_journal_entries: false,
            inject_anomalies: false,
            show_progress: false,
            ..Default::default()
        };

        let mut orchestrator = EnhancedOrchestrator::new(config, phase_config).unwrap();
        let result = orchestrator.generate().unwrap();

        // Without master data, document flows should be empty
        assert!(result.document_flows.p2p_chains.is_empty());
        assert!(result.document_flows.o2c_chains.is_empty());
    }

    #[test]
    fn test_journal_entry_line_item_count() {
        let config = create_test_config();
        let phase_config = PhaseConfig {
            generate_master_data: false,
            generate_document_flows: false,
            generate_journal_entries: true,
            inject_anomalies: false,
            show_progress: false,
            ..Default::default()
        };

        let mut orchestrator = EnhancedOrchestrator::new(config, phase_config).unwrap();
        let result = orchestrator.generate().unwrap();

        // Total line items should match sum of all entry line counts
        let calculated_line_items: u64 = result
            .journal_entries
            .iter()
            .map(|e| e.line_count() as u64)
            .sum();
        assert_eq!(result.statistics.total_line_items, calculated_line_items);
    }
}
