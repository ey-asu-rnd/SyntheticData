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

use chrono::{Datelike, NaiveDate};
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};

use synth_config::schema::GeneratorConfig;
use synth_core::error::{SynthError, SynthResult};
use synth_core::models::*;
use synth_generators::{
    // Core generators
    ChartOfAccountsGenerator, JournalEntryGenerator,
    // Master data generators
    VendorGenerator, CustomerGenerator, MaterialGenerator, AssetGenerator, EmployeeGenerator,
    // Document flow generators
    P2PGenerator, P2PDocumentChain, O2CGenerator, O2CDocumentChain,
    // Anomaly injection
    AnomalyInjector, AnomalyInjectorConfig,
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

/// Complete result of enhanced generation run.
#[derive(Debug)]
pub struct EnhancedGenerationResult {
    /// Generated chart of accounts.
    pub chart_of_accounts: ChartOfAccounts,
    /// Master data snapshot.
    pub master_data: MasterDataSnapshot,
    /// Document flow snapshot.
    pub document_flows: DocumentFlowSnapshot,
    /// Generated journal entries.
    pub journal_entries: Vec<JournalEntry>,
    /// Anomaly labels (if injection enabled).
    pub anomaly_labels: AnomalyLabels,
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
    /// Anomaly counts.
    pub anomalies_injected: usize,
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

    /// Run the complete generation workflow.
    pub fn generate(&mut self) -> SynthResult<EnhancedGenerationResult> {
        let mut stats = EnhancedGenerationStatistics::default();
        stats.companies_count = self.config.companies.len();
        stats.period_months = self.config.global.period_months;

        // Phase 1: Generate Chart of Accounts
        let coa = self.generate_coa()?;
        stats.accounts_count = coa.account_count();

        // Phase 2: Generate Master Data
        if self.phase_config.generate_master_data {
            self.generate_master_data()?;
            stats.vendor_count = self.master_data.vendors.len();
            stats.customer_count = self.master_data.customers.len();
            stats.material_count = self.master_data.materials.len();
            stats.asset_count = self.master_data.assets.len();
            stats.employee_count = self.master_data.employees.len();
        }

        // Phase 3: Generate Document Flows
        let mut document_flows = DocumentFlowSnapshot::default();
        if self.phase_config.generate_document_flows && !self.master_data.vendors.is_empty() {
            self.generate_document_flows(&mut document_flows)?;
            stats.p2p_chain_count = document_flows.p2p_chains.len();
            stats.o2c_chain_count = document_flows.o2c_chains.len();
        }

        // Phase 4: Generate Journal Entries
        let mut entries = Vec::new();
        if self.phase_config.generate_journal_entries {
            entries = self.generate_journal_entries(&coa)?;
            stats.total_entries = entries.len() as u64;
            stats.total_line_items = entries.iter().map(|e| e.line_count() as u64).sum();
        }

        // Phase 5: Inject Anomalies
        let mut anomaly_labels = AnomalyLabels::default();
        if self.phase_config.inject_anomalies && !entries.is_empty() {
            let result = self.inject_anomalies(&mut entries)?;
            stats.anomalies_injected = result.labels.len();
            anomaly_labels = result;
        }

        Ok(EnhancedGenerationResult {
            chart_of_accounts: (*coa).clone(),
            master_data: self.master_data.clone(),
            document_flows,
            journal_entries: entries,
            anomaly_labels,
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
            if let Some(pb) = &pb { pb.inc(1); }

            // Generate customers
            let mut customer_gen = CustomerGenerator::new(company_seed + 100);
            let customer_pool = customer_gen.generate_customer_pool(
                self.phase_config.customers_per_company,
                &company.code,
                start_date,
            );
            self.master_data.customers.extend(customer_pool.customers);
            if let Some(pb) = &pb { pb.inc(1); }

            // Generate materials
            let mut material_gen = MaterialGenerator::new(company_seed + 200);
            let material_pool = material_gen.generate_material_pool(
                self.phase_config.materials_per_company,
                &company.code,
                start_date,
            );
            self.master_data.materials.extend(material_pool.materials);
            if let Some(pb) = &pb { pb.inc(1); }

            // Generate fixed assets
            let mut asset_gen = AssetGenerator::new(company_seed + 300);
            let asset_pool = asset_gen.generate_asset_pool(
                self.phase_config.assets_per_company,
                &company.code,
                (start_date, end_date),
            );
            self.master_data.assets.extend(asset_pool.assets);
            if let Some(pb) = &pb { pb.inc(1); }

            // Generate employees
            let mut employee_gen = EmployeeGenerator::new(company_seed + 400);
            let employee_pool = employee_gen.generate_company_pool(
                &company.code,
                (start_date, end_date),
            );
            self.master_data.employees.extend(employee_pool.employees);
            if let Some(pb) = &pb { pb.inc(1); }
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
        let p2p_count = self.phase_config.p2p_chains.min(self.master_data.vendors.len() * 2);
        let pb = self.create_progress_bar(p2p_count as u64, "Generating P2P Document Flows");

        let mut p2p_gen = P2PGenerator::new(self.seed + 1000);

        for i in 0..p2p_count {
            let vendor = &self.master_data.vendors[i % self.master_data.vendors.len()];
            let materials: Vec<&Material> = self.master_data.materials
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
            let created_by = self.master_data.employees
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

            if let Some(pb) = &pb { pb.inc(1); }
        }

        if let Some(pb) = pb {
            pb.finish_with_message("P2P document flows complete");
        }

        // Generate O2C chains
        let o2c_count = self.phase_config.o2c_chains.min(self.master_data.customers.len() * 2);
        let pb = self.create_progress_bar(o2c_count as u64, "Generating O2C Document Flows");

        let mut o2c_gen = O2CGenerator::new(self.seed + 2000);

        for i in 0..o2c_count {
            let customer = &self.master_data.customers[i % self.master_data.customers.len()];
            let materials: Vec<&Material> = self.master_data.materials
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
            let created_by = self.master_data.employees
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

            if let Some(pb) = &pb { pb.inc(1); }
        }

        if let Some(pb) = pb {
            pb.finish_with_message("O2C document flows complete");
        }

        Ok(())
    }

    /// Generate journal entries.
    fn generate_journal_entries(&mut self, coa: &Arc<ChartOfAccounts>) -> SynthResult<Vec<JournalEntry>> {
        let total = self.calculate_total_transactions();
        let pb = self.create_progress_bar(total, "Generating Journal Entries");

        let start_date = NaiveDate::parse_from_str(&self.config.global.start_date, "%Y-%m-%d")
            .map_err(|e| SynthError::config(format!("Invalid start_date: {}", e)))?;
        let end_date = start_date + chrono::Months::new(self.config.global.period_months);

        let company_codes: Vec<String> = self.config.companies.iter().map(|c| c.code.clone()).collect();

        let mut generator = JournalEntryGenerator::new_with_params(
            self.config.transactions.clone(),
            Arc::clone(coa),
            company_codes,
            start_date,
            end_date,
            self.seed,
        );

        let mut entries = Vec::with_capacity(total as usize);

        for _ in 0..total {
            let entry = generator.generate();
            entries.push(entry);
            if let Some(pb) = &pb { pb.inc(1); }
        }

        if let Some(pb) = pb {
            pb.finish_with_message("Journal entries complete");
        }

        Ok(entries)
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
            *by_type.entry(format!("{:?}", label.anomaly_type)).or_insert(0) += 1;
        }

        Ok(AnomalyLabels {
            labels: result.labels,
            summary: Some(result.summary),
            by_type,
        })
    }

    /// Calculate total transactions to generate.
    fn calculate_total_transactions(&self) -> u64 {
        let months = self.config.global.period_months as f64;
        self.config.companies.iter().map(|c| {
            let annual = c.annual_transaction_volume.count() as f64;
            let weighted = annual * c.volume_weight;
            (weighted * months / 12.0) as u64
        }).sum()
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
                .unwrap()
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
        assert_eq!(result.statistics.vendor_count, result.master_data.vendors.len());
        assert_eq!(result.statistics.customer_count, result.master_data.customers.len());
        assert_eq!(result.statistics.material_count, result.master_data.materials.len());
        assert_eq!(result.statistics.total_entries as usize, result.journal_entries.len());
    }

    #[test]
    fn test_phase_config_defaults() {
        let config = PhaseConfig::default();
        assert!(config.generate_master_data);
        assert!(config.generate_document_flows);
        assert!(config.generate_journal_entries);
        assert!(!config.inject_anomalies);
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
        assert!(orchestrator.phase_config.show_progress == false);
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
            generate_master_data: false, // Skip master data
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
