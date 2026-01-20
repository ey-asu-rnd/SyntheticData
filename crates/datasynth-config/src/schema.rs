//! Configuration schema for synthetic data generation.

use datasynth_core::distributions::{
    AmountDistributionConfig, DebitCreditDistributionConfig, EvenOddDistributionConfig,
    LineItemDistributionConfig, SeasonalityConfig,
};
use datasynth_core::models::{CoAComplexity, IndustrySector};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Root configuration for the synthetic data generator.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneratorConfig {
    /// Global settings
    pub global: GlobalConfig,
    /// Company configuration
    pub companies: Vec<CompanyConfig>,
    /// Chart of Accounts configuration
    pub chart_of_accounts: ChartOfAccountsConfig,
    /// Transaction generation settings
    #[serde(default)]
    pub transactions: TransactionConfig,
    /// Output configuration
    pub output: OutputConfig,
    /// Fraud simulation settings
    #[serde(default)]
    pub fraud: FraudConfig,
    /// Data quality variation settings
    #[serde(default)]
    pub data_quality: DataQualitySchemaConfig,
    /// Internal Controls System settings
    #[serde(default)]
    pub internal_controls: InternalControlsConfig,
    /// Business process mix
    #[serde(default)]
    pub business_processes: BusinessProcessConfig,
    /// User persona distribution
    #[serde(default)]
    pub user_personas: UserPersonaConfig,
    /// Template configuration for realistic data
    #[serde(default)]
    pub templates: TemplateConfig,
    /// Approval workflow configuration
    #[serde(default)]
    pub approval: ApprovalConfig,
    /// Department structure configuration
    #[serde(default)]
    pub departments: DepartmentConfig,
    /// Master data generation settings
    #[serde(default)]
    pub master_data: MasterDataConfig,
    /// Document flow generation settings
    #[serde(default)]
    pub document_flows: DocumentFlowConfig,
    /// Intercompany transaction settings
    #[serde(default)]
    pub intercompany: IntercompanyConfig,
    /// Balance and trial balance settings
    #[serde(default)]
    pub balance: BalanceConfig,
    /// OCPM (Object-Centric Process Mining) settings
    #[serde(default)]
    pub ocpm: OcpmConfig,
    /// Audit engagement and workpaper generation settings
    #[serde(default)]
    pub audit: AuditGenerationConfig,
    /// Banking KYC/AML transaction generation settings
    #[serde(default)]
    pub banking: datasynth_banking::BankingConfig,
}

/// Global configuration settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlobalConfig {
    /// Random seed for reproducibility
    pub seed: Option<u64>,
    /// Industry sector
    pub industry: IndustrySector,
    /// Simulation start date (YYYY-MM-DD)
    pub start_date: String,
    /// Simulation period in months
    pub period_months: u32,
    /// Base currency for group reporting
    #[serde(default = "default_currency")]
    pub group_currency: String,
    /// Enable parallel generation
    #[serde(default = "default_true")]
    pub parallel: bool,
    /// Number of worker threads (0 = auto-detect)
    #[serde(default)]
    pub worker_threads: usize,
    /// Memory limit in MB (0 = unlimited)
    #[serde(default)]
    pub memory_limit_mb: usize,
}

fn default_currency() -> String {
    "USD".to_string()
}
fn default_true() -> bool {
    true
}

/// Company code configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompanyConfig {
    /// Company code identifier
    pub code: String,
    /// Company name
    pub name: String,
    /// Local currency (ISO 4217)
    pub currency: String,
    /// Country code (ISO 3166-1 alpha-2)
    pub country: String,
    /// Fiscal year variant
    #[serde(default = "default_fiscal_variant")]
    pub fiscal_year_variant: String,
    /// Transaction volume per year
    pub annual_transaction_volume: TransactionVolume,
    /// Company-specific transaction weight
    #[serde(default = "default_weight")]
    pub volume_weight: f64,
}

fn default_fiscal_variant() -> String {
    "K4".to_string()
}
fn default_weight() -> f64 {
    1.0
}

/// Transaction volume presets.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TransactionVolume {
    /// 10,000 transactions per year
    TenK,
    /// 100,000 transactions per year
    HundredK,
    /// 1,000,000 transactions per year
    OneM,
    /// 10,000,000 transactions per year
    TenM,
    /// 100,000,000 transactions per year
    HundredM,
    /// Custom count
    Custom(u64),
}

impl TransactionVolume {
    /// Get the transaction count.
    pub fn count(&self) -> u64 {
        match self {
            Self::TenK => 10_000,
            Self::HundredK => 100_000,
            Self::OneM => 1_000_000,
            Self::TenM => 10_000_000,
            Self::HundredM => 100_000_000,
            Self::Custom(n) => *n,
        }
    }
}

/// Chart of Accounts configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChartOfAccountsConfig {
    /// CoA complexity level
    pub complexity: CoAComplexity,
    /// Use industry-specific accounts
    #[serde(default = "default_true")]
    pub industry_specific: bool,
    /// Custom account definitions file
    pub custom_accounts: Option<PathBuf>,
    /// Minimum hierarchy depth
    #[serde(default = "default_min_depth")]
    pub min_hierarchy_depth: u8,
    /// Maximum hierarchy depth
    #[serde(default = "default_max_depth")]
    pub max_hierarchy_depth: u8,
}

fn default_min_depth() -> u8 {
    2
}
fn default_max_depth() -> u8 {
    5
}

/// Transaction generation configuration.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TransactionConfig {
    /// Line item distribution
    #[serde(default)]
    pub line_item_distribution: LineItemDistributionConfig,
    /// Debit/credit balance distribution
    #[serde(default)]
    pub debit_credit_distribution: DebitCreditDistributionConfig,
    /// Even/odd line count distribution
    #[serde(default)]
    pub even_odd_distribution: EvenOddDistributionConfig,
    /// Transaction source distribution
    #[serde(default)]
    pub source_distribution: SourceDistribution,
    /// Seasonality configuration
    #[serde(default)]
    pub seasonality: SeasonalityConfig,
    /// Amount distribution
    #[serde(default)]
    pub amounts: AmountDistributionConfig,
    /// Benford's Law compliance configuration
    #[serde(default)]
    pub benford: BenfordConfig,
}

/// Benford's Law compliance configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenfordConfig {
    /// Enable Benford's Law compliance for amount generation
    #[serde(default = "default_true")]
    pub enabled: bool,
    /// Tolerance for deviation from ideal Benford distribution (0.0-1.0)
    #[serde(default = "default_benford_tolerance")]
    pub tolerance: f64,
    /// Transaction sources exempt from Benford's Law (fixed amounts)
    #[serde(default)]
    pub exempt_sources: Vec<BenfordExemption>,
}

fn default_benford_tolerance() -> f64 {
    0.05
}

impl Default for BenfordConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            tolerance: default_benford_tolerance(),
            exempt_sources: vec![BenfordExemption::Recurring, BenfordExemption::Payroll],
        }
    }
}

/// Types of transactions exempt from Benford's Law.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BenfordExemption {
    /// Recurring fixed amounts (rent, subscriptions)
    Recurring,
    /// Payroll (standardized salaries)
    Payroll,
    /// Fixed fees and charges
    FixedFees,
    /// Round number purchases (often legitimate)
    RoundAmounts,
}

/// Distribution of transaction sources.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceDistribution {
    /// Manual entries percentage
    pub manual: f64,
    /// Automated system entries
    pub automated: f64,
    /// Recurring entries
    pub recurring: f64,
    /// Adjustment entries
    pub adjustment: f64,
}

impl Default for SourceDistribution {
    fn default() -> Self {
        Self {
            manual: 0.20,
            automated: 0.70,
            recurring: 0.07,
            adjustment: 0.03,
        }
    }
}

/// Output configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutputConfig {
    /// Output mode
    #[serde(default)]
    pub mode: OutputMode,
    /// Output directory
    pub output_directory: PathBuf,
    /// File formats to generate
    #[serde(default = "default_formats")]
    pub formats: Vec<FileFormat>,
    /// Compression settings
    #[serde(default)]
    pub compression: CompressionConfig,
    /// Batch size for writes
    #[serde(default = "default_batch_size")]
    pub batch_size: usize,
    /// Include ACDOCA format
    #[serde(default = "default_true")]
    pub include_acdoca: bool,
    /// Include BSEG format
    #[serde(default)]
    pub include_bseg: bool,
    /// Partition by fiscal period
    #[serde(default = "default_true")]
    pub partition_by_period: bool,
    /// Partition by company code
    #[serde(default)]
    pub partition_by_company: bool,
}

fn default_formats() -> Vec<FileFormat> {
    vec![FileFormat::Parquet]
}
fn default_batch_size() -> usize {
    100_000
}

impl Default for OutputConfig {
    fn default() -> Self {
        Self {
            mode: OutputMode::FlatFile,
            output_directory: PathBuf::from("./output"),
            formats: default_formats(),
            compression: CompressionConfig::default(),
            batch_size: default_batch_size(),
            include_acdoca: true,
            include_bseg: false,
            partition_by_period: true,
            partition_by_company: false,
        }
    }
}

/// Output mode.
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OutputMode {
    /// Stream records as generated
    Streaming,
    /// Write to flat files
    #[default]
    FlatFile,
    /// Both streaming and flat file
    Both,
}

/// Supported file formats.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FileFormat {
    Csv,
    Parquet,
    Json,
    JsonLines,
}

/// Compression configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompressionConfig {
    /// Enable compression
    #[serde(default = "default_true")]
    pub enabled: bool,
    /// Compression algorithm
    #[serde(default)]
    pub algorithm: CompressionAlgorithm,
    /// Compression level (1-9)
    #[serde(default = "default_compression_level")]
    pub level: u8,
}

fn default_compression_level() -> u8 {
    3
}

impl Default for CompressionConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            algorithm: CompressionAlgorithm::default(),
            level: default_compression_level(),
        }
    }
}

/// Compression algorithms.
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CompressionAlgorithm {
    Gzip,
    #[default]
    Zstd,
    Lz4,
    Snappy,
}

/// Fraud simulation configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FraudConfig {
    /// Enable fraud scenario generation
    #[serde(default)]
    pub enabled: bool,
    /// Overall fraud rate (0.0 to 1.0)
    #[serde(default = "default_fraud_rate")]
    pub fraud_rate: f64,
    /// Fraud type distribution
    #[serde(default)]
    pub fraud_type_distribution: FraudTypeDistribution,
    /// Enable fraud clustering
    #[serde(default)]
    pub clustering_enabled: bool,
    /// Clustering factor
    #[serde(default = "default_clustering_factor")]
    pub clustering_factor: f64,
    /// Approval thresholds for threshold-adjacent fraud pattern
    #[serde(default = "default_approval_thresholds")]
    pub approval_thresholds: Vec<f64>,
}

fn default_approval_thresholds() -> Vec<f64> {
    vec![1000.0, 5000.0, 10000.0, 25000.0, 50000.0, 100000.0]
}

fn default_fraud_rate() -> f64 {
    0.005
}
fn default_clustering_factor() -> f64 {
    3.0
}

impl Default for FraudConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            fraud_rate: default_fraud_rate(),
            fraud_type_distribution: FraudTypeDistribution::default(),
            clustering_enabled: false,
            clustering_factor: default_clustering_factor(),
            approval_thresholds: default_approval_thresholds(),
        }
    }
}

/// Distribution of fraud types.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FraudTypeDistribution {
    pub suspense_account_abuse: f64,
    pub fictitious_transaction: f64,
    pub revenue_manipulation: f64,
    pub expense_capitalization: f64,
    pub split_transaction: f64,
    pub timing_anomaly: f64,
    pub unauthorized_access: f64,
    pub duplicate_payment: f64,
}

impl Default for FraudTypeDistribution {
    fn default() -> Self {
        Self {
            suspense_account_abuse: 0.25,
            fictitious_transaction: 0.15,
            revenue_manipulation: 0.10,
            expense_capitalization: 0.10,
            split_transaction: 0.15,
            timing_anomaly: 0.10,
            unauthorized_access: 0.10,
            duplicate_payment: 0.05,
        }
    }
}

/// Internal Controls System (ICS) configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InternalControlsConfig {
    /// Enable internal controls system
    #[serde(default)]
    pub enabled: bool,
    /// Rate at which controls result in exceptions (0.0 - 1.0)
    #[serde(default = "default_exception_rate")]
    pub exception_rate: f64,
    /// Rate at which SoD violations occur (0.0 - 1.0)
    #[serde(default = "default_sod_violation_rate")]
    pub sod_violation_rate: f64,
    /// Export control master data to separate files
    #[serde(default = "default_true")]
    pub export_control_master_data: bool,
    /// SOX materiality threshold for marking transactions as SOX-relevant
    #[serde(default = "default_sox_materiality_threshold")]
    pub sox_materiality_threshold: f64,
}

fn default_exception_rate() -> f64 {
    0.02
}

fn default_sod_violation_rate() -> f64 {
    0.01
}

fn default_sox_materiality_threshold() -> f64 {
    10000.0
}

impl Default for InternalControlsConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            exception_rate: default_exception_rate(),
            sod_violation_rate: default_sod_violation_rate(),
            export_control_master_data: true,
            sox_materiality_threshold: default_sox_materiality_threshold(),
        }
    }
}

/// Business process configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BusinessProcessConfig {
    /// Order-to-Cash weight
    #[serde(default = "default_o2c")]
    pub o2c_weight: f64,
    /// Procure-to-Pay weight
    #[serde(default = "default_p2p")]
    pub p2p_weight: f64,
    /// Record-to-Report weight
    #[serde(default = "default_r2r")]
    pub r2r_weight: f64,
    /// Hire-to-Retire weight
    #[serde(default = "default_h2r")]
    pub h2r_weight: f64,
    /// Acquire-to-Retire weight
    #[serde(default = "default_a2r")]
    pub a2r_weight: f64,
}

fn default_o2c() -> f64 {
    0.35
}
fn default_p2p() -> f64 {
    0.30
}
fn default_r2r() -> f64 {
    0.20
}
fn default_h2r() -> f64 {
    0.10
}
fn default_a2r() -> f64 {
    0.05
}

impl Default for BusinessProcessConfig {
    fn default() -> Self {
        Self {
            o2c_weight: default_o2c(),
            p2p_weight: default_p2p(),
            r2r_weight: default_r2r(),
            h2r_weight: default_h2r(),
            a2r_weight: default_a2r(),
        }
    }
}

/// User persona configuration.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct UserPersonaConfig {
    /// Distribution of user personas
    #[serde(default)]
    pub persona_distribution: PersonaDistribution,
    /// Users per persona type
    #[serde(default)]
    pub users_per_persona: UsersPerPersona,
}

/// Distribution of user personas for transaction generation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersonaDistribution {
    pub junior_accountant: f64,
    pub senior_accountant: f64,
    pub controller: f64,
    pub manager: f64,
    pub automated_system: f64,
}

impl Default for PersonaDistribution {
    fn default() -> Self {
        Self {
            junior_accountant: 0.15,
            senior_accountant: 0.15,
            controller: 0.05,
            manager: 0.05,
            automated_system: 0.60,
        }
    }
}

/// Number of users per persona type.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsersPerPersona {
    pub junior_accountant: usize,
    pub senior_accountant: usize,
    pub controller: usize,
    pub manager: usize,
    pub automated_system: usize,
}

impl Default for UsersPerPersona {
    fn default() -> Self {
        Self {
            junior_accountant: 10,
            senior_accountant: 5,
            controller: 2,
            manager: 3,
            automated_system: 20,
        }
    }
}

/// Template configuration for realistic data generation.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TemplateConfig {
    /// Name generation settings
    #[serde(default)]
    pub names: NameTemplateConfig,
    /// Description generation settings
    #[serde(default)]
    pub descriptions: DescriptionTemplateConfig,
    /// Reference number settings
    #[serde(default)]
    pub references: ReferenceTemplateConfig,
}

/// Name template configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NameTemplateConfig {
    /// Distribution of name cultures
    #[serde(default)]
    pub culture_distribution: CultureDistribution,
    /// Email domain for generated users
    #[serde(default = "default_email_domain")]
    pub email_domain: String,
    /// Generate realistic display names
    #[serde(default = "default_true")]
    pub generate_realistic_names: bool,
}

fn default_email_domain() -> String {
    "company.com".to_string()
}

impl Default for NameTemplateConfig {
    fn default() -> Self {
        Self {
            culture_distribution: CultureDistribution::default(),
            email_domain: default_email_domain(),
            generate_realistic_names: true,
        }
    }
}

/// Distribution of name cultures for generation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CultureDistribution {
    pub western_us: f64,
    pub hispanic: f64,
    pub german: f64,
    pub french: f64,
    pub chinese: f64,
    pub japanese: f64,
    pub indian: f64,
}

impl Default for CultureDistribution {
    fn default() -> Self {
        Self {
            western_us: 0.40,
            hispanic: 0.20,
            german: 0.10,
            french: 0.05,
            chinese: 0.10,
            japanese: 0.05,
            indian: 0.10,
        }
    }
}

/// Description template configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DescriptionTemplateConfig {
    /// Generate header text for journal entries
    #[serde(default = "default_true")]
    pub generate_header_text: bool,
    /// Generate line text for journal entry lines
    #[serde(default = "default_true")]
    pub generate_line_text: bool,
}

impl Default for DescriptionTemplateConfig {
    fn default() -> Self {
        Self {
            generate_header_text: true,
            generate_line_text: true,
        }
    }
}

/// Reference number template configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReferenceTemplateConfig {
    /// Generate reference numbers
    #[serde(default = "default_true")]
    pub generate_references: bool,
    /// Invoice prefix
    #[serde(default = "default_invoice_prefix")]
    pub invoice_prefix: String,
    /// Purchase order prefix
    #[serde(default = "default_po_prefix")]
    pub po_prefix: String,
    /// Sales order prefix
    #[serde(default = "default_so_prefix")]
    pub so_prefix: String,
}

fn default_invoice_prefix() -> String {
    "INV".to_string()
}
fn default_po_prefix() -> String {
    "PO".to_string()
}
fn default_so_prefix() -> String {
    "SO".to_string()
}

impl Default for ReferenceTemplateConfig {
    fn default() -> Self {
        Self {
            generate_references: true,
            invoice_prefix: default_invoice_prefix(),
            po_prefix: default_po_prefix(),
            so_prefix: default_so_prefix(),
        }
    }
}

/// Approval workflow configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApprovalConfig {
    /// Enable approval workflow generation
    #[serde(default)]
    pub enabled: bool,
    /// Threshold below which transactions are auto-approved
    #[serde(default = "default_auto_approve_threshold")]
    pub auto_approve_threshold: f64,
    /// Rate at which approvals are rejected (0.0 to 1.0)
    #[serde(default = "default_rejection_rate")]
    pub rejection_rate: f64,
    /// Rate at which approvals require revision (0.0 to 1.0)
    #[serde(default = "default_revision_rate")]
    pub revision_rate: f64,
    /// Average delay in hours for approval processing
    #[serde(default = "default_approval_delay_hours")]
    pub average_approval_delay_hours: f64,
    /// Approval chain thresholds
    #[serde(default)]
    pub thresholds: Vec<ApprovalThresholdConfig>,
}

fn default_auto_approve_threshold() -> f64 {
    1000.0
}
fn default_rejection_rate() -> f64 {
    0.02
}
fn default_revision_rate() -> f64 {
    0.05
}
fn default_approval_delay_hours() -> f64 {
    4.0
}

impl Default for ApprovalConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            auto_approve_threshold: default_auto_approve_threshold(),
            rejection_rate: default_rejection_rate(),
            revision_rate: default_revision_rate(),
            average_approval_delay_hours: default_approval_delay_hours(),
            thresholds: vec![
                ApprovalThresholdConfig {
                    amount: 1000.0,
                    level: 1,
                    roles: vec!["senior_accountant".to_string()],
                },
                ApprovalThresholdConfig {
                    amount: 10000.0,
                    level: 2,
                    roles: vec!["senior_accountant".to_string(), "controller".to_string()],
                },
                ApprovalThresholdConfig {
                    amount: 100000.0,
                    level: 3,
                    roles: vec![
                        "senior_accountant".to_string(),
                        "controller".to_string(),
                        "manager".to_string(),
                    ],
                },
                ApprovalThresholdConfig {
                    amount: 500000.0,
                    level: 4,
                    roles: vec![
                        "senior_accountant".to_string(),
                        "controller".to_string(),
                        "manager".to_string(),
                        "executive".to_string(),
                    ],
                },
            ],
        }
    }
}

/// Configuration for a single approval threshold.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApprovalThresholdConfig {
    /// Amount threshold
    pub amount: f64,
    /// Approval level required
    pub level: u8,
    /// Roles that can approve at this level
    pub roles: Vec<String>,
}

/// Department configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DepartmentConfig {
    /// Enable department assignment
    #[serde(default)]
    pub enabled: bool,
    /// Multiplier for department headcounts
    #[serde(default = "default_headcount_multiplier")]
    pub headcount_multiplier: f64,
    /// Custom department definitions (optional)
    #[serde(default)]
    pub custom_departments: Vec<CustomDepartmentConfig>,
}

fn default_headcount_multiplier() -> f64 {
    1.0
}

impl Default for DepartmentConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            headcount_multiplier: default_headcount_multiplier(),
            custom_departments: Vec::new(),
        }
    }
}

/// Custom department definition.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomDepartmentConfig {
    /// Department code
    pub code: String,
    /// Department name
    pub name: String,
    /// Associated cost center
    #[serde(default)]
    pub cost_center: Option<String>,
    /// Primary business processes
    #[serde(default)]
    pub primary_processes: Vec<String>,
    /// Parent department code
    #[serde(default)]
    pub parent_code: Option<String>,
}

// ============================================================================
// Master Data Configuration
// ============================================================================

/// Master data generation configuration.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MasterDataConfig {
    /// Vendor master data settings
    #[serde(default)]
    pub vendors: VendorMasterConfig,
    /// Customer master data settings
    #[serde(default)]
    pub customers: CustomerMasterConfig,
    /// Material master data settings
    #[serde(default)]
    pub materials: MaterialMasterConfig,
    /// Fixed asset master data settings
    #[serde(default)]
    pub fixed_assets: FixedAssetMasterConfig,
    /// Employee master data settings
    #[serde(default)]
    pub employees: EmployeeMasterConfig,
    /// Cost center master data settings
    #[serde(default)]
    pub cost_centers: CostCenterMasterConfig,
}

/// Vendor master data configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VendorMasterConfig {
    /// Number of vendors to generate
    #[serde(default = "default_vendor_count")]
    pub count: usize,
    /// Percentage of vendors that are intercompany (0.0 to 1.0)
    #[serde(default = "default_intercompany_percent")]
    pub intercompany_percent: f64,
    /// Payment terms distribution
    #[serde(default)]
    pub payment_terms_distribution: PaymentTermsDistribution,
    /// Vendor behavior distribution
    #[serde(default)]
    pub behavior_distribution: VendorBehaviorDistribution,
    /// Generate bank account details
    #[serde(default = "default_true")]
    pub generate_bank_accounts: bool,
    /// Generate tax IDs
    #[serde(default = "default_true")]
    pub generate_tax_ids: bool,
}

fn default_vendor_count() -> usize {
    500
}

fn default_intercompany_percent() -> f64 {
    0.05
}

impl Default for VendorMasterConfig {
    fn default() -> Self {
        Self {
            count: default_vendor_count(),
            intercompany_percent: default_intercompany_percent(),
            payment_terms_distribution: PaymentTermsDistribution::default(),
            behavior_distribution: VendorBehaviorDistribution::default(),
            generate_bank_accounts: true,
            generate_tax_ids: true,
        }
    }
}

/// Payment terms distribution for vendors.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaymentTermsDistribution {
    /// Net 30 days
    pub net_30: f64,
    /// Net 60 days
    pub net_60: f64,
    /// Net 90 days
    pub net_90: f64,
    /// 2% 10 Net 30 (early payment discount)
    pub two_ten_net_30: f64,
    /// Due on receipt
    pub due_on_receipt: f64,
    /// End of month
    pub end_of_month: f64,
}

impl Default for PaymentTermsDistribution {
    fn default() -> Self {
        Self {
            net_30: 0.40,
            net_60: 0.20,
            net_90: 0.10,
            two_ten_net_30: 0.15,
            due_on_receipt: 0.05,
            end_of_month: 0.10,
        }
    }
}

/// Vendor behavior distribution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VendorBehaviorDistribution {
    /// Reliable vendors (consistent delivery, quality)
    pub reliable: f64,
    /// Sometimes late vendors
    pub sometimes_late: f64,
    /// Inconsistent quality vendors
    pub inconsistent_quality: f64,
    /// Premium vendors (high quality, premium pricing)
    pub premium: f64,
    /// Budget vendors (lower quality, lower pricing)
    pub budget: f64,
}

impl Default for VendorBehaviorDistribution {
    fn default() -> Self {
        Self {
            reliable: 0.50,
            sometimes_late: 0.20,
            inconsistent_quality: 0.10,
            premium: 0.10,
            budget: 0.10,
        }
    }
}

/// Customer master data configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomerMasterConfig {
    /// Number of customers to generate
    #[serde(default = "default_customer_count")]
    pub count: usize,
    /// Percentage of customers that are intercompany (0.0 to 1.0)
    #[serde(default = "default_intercompany_percent")]
    pub intercompany_percent: f64,
    /// Credit rating distribution
    #[serde(default)]
    pub credit_rating_distribution: CreditRatingDistribution,
    /// Payment behavior distribution
    #[serde(default)]
    pub payment_behavior_distribution: PaymentBehaviorDistribution,
    /// Generate credit limits based on rating
    #[serde(default = "default_true")]
    pub generate_credit_limits: bool,
}

fn default_customer_count() -> usize {
    2000
}

impl Default for CustomerMasterConfig {
    fn default() -> Self {
        Self {
            count: default_customer_count(),
            intercompany_percent: default_intercompany_percent(),
            credit_rating_distribution: CreditRatingDistribution::default(),
            payment_behavior_distribution: PaymentBehaviorDistribution::default(),
            generate_credit_limits: true,
        }
    }
}

/// Credit rating distribution for customers.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreditRatingDistribution {
    /// AAA rating
    pub aaa: f64,
    /// AA rating
    pub aa: f64,
    /// A rating
    pub a: f64,
    /// BBB rating
    pub bbb: f64,
    /// BB rating
    pub bb: f64,
    /// B rating
    pub b: f64,
    /// Below B rating
    pub below_b: f64,
}

impl Default for CreditRatingDistribution {
    fn default() -> Self {
        Self {
            aaa: 0.05,
            aa: 0.10,
            a: 0.20,
            bbb: 0.30,
            bb: 0.20,
            b: 0.10,
            below_b: 0.05,
        }
    }
}

/// Payment behavior distribution for customers.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaymentBehaviorDistribution {
    /// Always pays early
    pub early_payer: f64,
    /// Pays on time
    pub on_time: f64,
    /// Occasionally late
    pub occasional_late: f64,
    /// Frequently late
    pub frequent_late: f64,
    /// Takes early payment discounts
    pub discount_taker: f64,
}

impl Default for PaymentBehaviorDistribution {
    fn default() -> Self {
        Self {
            early_payer: 0.10,
            on_time: 0.50,
            occasional_late: 0.25,
            frequent_late: 0.10,
            discount_taker: 0.05,
        }
    }
}

/// Material master data configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MaterialMasterConfig {
    /// Number of materials to generate
    #[serde(default = "default_material_count")]
    pub count: usize,
    /// Material type distribution
    #[serde(default)]
    pub type_distribution: MaterialTypeDistribution,
    /// Valuation method distribution
    #[serde(default)]
    pub valuation_distribution: ValuationMethodDistribution,
    /// Percentage of materials with BOM (bill of materials)
    #[serde(default = "default_bom_percent")]
    pub bom_percent: f64,
    /// Maximum BOM depth
    #[serde(default = "default_max_bom_depth")]
    pub max_bom_depth: u8,
}

fn default_material_count() -> usize {
    5000
}

fn default_bom_percent() -> f64 {
    0.20
}

fn default_max_bom_depth() -> u8 {
    3
}

impl Default for MaterialMasterConfig {
    fn default() -> Self {
        Self {
            count: default_material_count(),
            type_distribution: MaterialTypeDistribution::default(),
            valuation_distribution: ValuationMethodDistribution::default(),
            bom_percent: default_bom_percent(),
            max_bom_depth: default_max_bom_depth(),
        }
    }
}

/// Material type distribution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MaterialTypeDistribution {
    /// Raw materials
    pub raw_material: f64,
    /// Semi-finished goods
    pub semi_finished: f64,
    /// Finished goods
    pub finished_good: f64,
    /// Trading goods (purchased for resale)
    pub trading_good: f64,
    /// Operating supplies
    pub operating_supply: f64,
    /// Services
    pub service: f64,
}

impl Default for MaterialTypeDistribution {
    fn default() -> Self {
        Self {
            raw_material: 0.30,
            semi_finished: 0.15,
            finished_good: 0.25,
            trading_good: 0.15,
            operating_supply: 0.10,
            service: 0.05,
        }
    }
}

/// Valuation method distribution for materials.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValuationMethodDistribution {
    /// Standard cost
    pub standard_cost: f64,
    /// Moving average
    pub moving_average: f64,
    /// FIFO (First In, First Out)
    pub fifo: f64,
    /// LIFO (Last In, First Out)
    pub lifo: f64,
}

impl Default for ValuationMethodDistribution {
    fn default() -> Self {
        Self {
            standard_cost: 0.50,
            moving_average: 0.30,
            fifo: 0.15,
            lifo: 0.05,
        }
    }
}

/// Fixed asset master data configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FixedAssetMasterConfig {
    /// Number of fixed assets to generate
    #[serde(default = "default_asset_count")]
    pub count: usize,
    /// Asset class distribution
    #[serde(default)]
    pub class_distribution: AssetClassDistribution,
    /// Depreciation method distribution
    #[serde(default)]
    pub depreciation_distribution: DepreciationMethodDistribution,
    /// Percentage of assets that are fully depreciated
    #[serde(default = "default_fully_depreciated_percent")]
    pub fully_depreciated_percent: f64,
    /// Generate acquisition history
    #[serde(default = "default_true")]
    pub generate_acquisition_history: bool,
}

fn default_asset_count() -> usize {
    800
}

fn default_fully_depreciated_percent() -> f64 {
    0.15
}

impl Default for FixedAssetMasterConfig {
    fn default() -> Self {
        Self {
            count: default_asset_count(),
            class_distribution: AssetClassDistribution::default(),
            depreciation_distribution: DepreciationMethodDistribution::default(),
            fully_depreciated_percent: default_fully_depreciated_percent(),
            generate_acquisition_history: true,
        }
    }
}

/// Asset class distribution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetClassDistribution {
    /// Buildings and structures
    pub buildings: f64,
    /// Machinery and equipment
    pub machinery: f64,
    /// Vehicles
    pub vehicles: f64,
    /// IT equipment
    pub it_equipment: f64,
    /// Furniture and fixtures
    pub furniture: f64,
    /// Land (non-depreciable)
    pub land: f64,
    /// Leasehold improvements
    pub leasehold: f64,
}

impl Default for AssetClassDistribution {
    fn default() -> Self {
        Self {
            buildings: 0.15,
            machinery: 0.30,
            vehicles: 0.15,
            it_equipment: 0.20,
            furniture: 0.10,
            land: 0.05,
            leasehold: 0.05,
        }
    }
}

/// Depreciation method distribution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DepreciationMethodDistribution {
    /// Straight line
    pub straight_line: f64,
    /// Declining balance
    pub declining_balance: f64,
    /// Double declining balance
    pub double_declining: f64,
    /// Sum of years' digits
    pub sum_of_years: f64,
    /// Units of production
    pub units_of_production: f64,
}

impl Default for DepreciationMethodDistribution {
    fn default() -> Self {
        Self {
            straight_line: 0.60,
            declining_balance: 0.20,
            double_declining: 0.10,
            sum_of_years: 0.05,
            units_of_production: 0.05,
        }
    }
}

/// Employee master data configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmployeeMasterConfig {
    /// Number of employees to generate
    #[serde(default = "default_employee_count")]
    pub count: usize,
    /// Generate organizational hierarchy
    #[serde(default = "default_true")]
    pub generate_hierarchy: bool,
    /// Maximum hierarchy depth
    #[serde(default = "default_hierarchy_depth")]
    pub max_hierarchy_depth: u8,
    /// Average span of control (direct reports per manager)
    #[serde(default = "default_span_of_control")]
    pub average_span_of_control: f64,
    /// Approval limit distribution by job level
    #[serde(default)]
    pub approval_limits: ApprovalLimitDistribution,
    /// Department distribution
    #[serde(default)]
    pub department_distribution: EmployeeDepartmentDistribution,
}

fn default_employee_count() -> usize {
    1500
}

fn default_hierarchy_depth() -> u8 {
    6
}

fn default_span_of_control() -> f64 {
    5.0
}

impl Default for EmployeeMasterConfig {
    fn default() -> Self {
        Self {
            count: default_employee_count(),
            generate_hierarchy: true,
            max_hierarchy_depth: default_hierarchy_depth(),
            average_span_of_control: default_span_of_control(),
            approval_limits: ApprovalLimitDistribution::default(),
            department_distribution: EmployeeDepartmentDistribution::default(),
        }
    }
}

/// Approval limit distribution by job level.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApprovalLimitDistribution {
    /// Staff level approval limit
    #[serde(default = "default_staff_limit")]
    pub staff: f64,
    /// Senior staff approval limit
    #[serde(default = "default_senior_limit")]
    pub senior: f64,
    /// Manager approval limit
    #[serde(default = "default_manager_limit")]
    pub manager: f64,
    /// Director approval limit
    #[serde(default = "default_director_limit")]
    pub director: f64,
    /// VP approval limit
    #[serde(default = "default_vp_limit")]
    pub vp: f64,
    /// Executive approval limit
    #[serde(default = "default_executive_limit")]
    pub executive: f64,
}

fn default_staff_limit() -> f64 {
    1000.0
}
fn default_senior_limit() -> f64 {
    5000.0
}
fn default_manager_limit() -> f64 {
    25000.0
}
fn default_director_limit() -> f64 {
    100000.0
}
fn default_vp_limit() -> f64 {
    500000.0
}
fn default_executive_limit() -> f64 {
    f64::INFINITY
}

impl Default for ApprovalLimitDistribution {
    fn default() -> Self {
        Self {
            staff: default_staff_limit(),
            senior: default_senior_limit(),
            manager: default_manager_limit(),
            director: default_director_limit(),
            vp: default_vp_limit(),
            executive: default_executive_limit(),
        }
    }
}

/// Employee distribution across departments.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmployeeDepartmentDistribution {
    /// Finance and Accounting
    pub finance: f64,
    /// Procurement
    pub procurement: f64,
    /// Sales
    pub sales: f64,
    /// Warehouse and Logistics
    pub warehouse: f64,
    /// IT
    pub it: f64,
    /// Human Resources
    pub hr: f64,
    /// Operations
    pub operations: f64,
    /// Executive
    pub executive: f64,
}

impl Default for EmployeeDepartmentDistribution {
    fn default() -> Self {
        Self {
            finance: 0.12,
            procurement: 0.10,
            sales: 0.25,
            warehouse: 0.15,
            it: 0.10,
            hr: 0.05,
            operations: 0.20,
            executive: 0.03,
        }
    }
}

/// Cost center master data configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CostCenterMasterConfig {
    /// Number of cost centers to generate
    #[serde(default = "default_cost_center_count")]
    pub count: usize,
    /// Generate cost center hierarchy
    #[serde(default = "default_true")]
    pub generate_hierarchy: bool,
    /// Maximum hierarchy depth
    #[serde(default = "default_cc_hierarchy_depth")]
    pub max_hierarchy_depth: u8,
}

fn default_cost_center_count() -> usize {
    50
}

fn default_cc_hierarchy_depth() -> u8 {
    3
}

impl Default for CostCenterMasterConfig {
    fn default() -> Self {
        Self {
            count: default_cost_center_count(),
            generate_hierarchy: true,
            max_hierarchy_depth: default_cc_hierarchy_depth(),
        }
    }
}

// ============================================================================
// Document Flow Configuration
// ============================================================================

/// Document flow generation configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentFlowConfig {
    /// P2P (Procure-to-Pay) flow configuration
    #[serde(default)]
    pub p2p: P2PFlowConfig,
    /// O2C (Order-to-Cash) flow configuration
    #[serde(default)]
    pub o2c: O2CFlowConfig,
    /// Generate document reference chains
    #[serde(default = "default_true")]
    pub generate_document_references: bool,
    /// Export document flow graph
    #[serde(default)]
    pub export_flow_graph: bool,
}

impl Default for DocumentFlowConfig {
    fn default() -> Self {
        Self {
            p2p: P2PFlowConfig::default(),
            o2c: O2CFlowConfig::default(),
            generate_document_references: true,
            export_flow_graph: false,
        }
    }
}

/// P2P (Procure-to-Pay) flow configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct P2PFlowConfig {
    /// Enable P2P document flow generation
    #[serde(default = "default_true")]
    pub enabled: bool,
    /// Three-way match success rate (PO-GR-Invoice)
    #[serde(default = "default_three_way_match_rate")]
    pub three_way_match_rate: f64,
    /// Rate of partial deliveries
    #[serde(default = "default_partial_delivery_rate")]
    pub partial_delivery_rate: f64,
    /// Rate of price variances between PO and Invoice
    #[serde(default = "default_price_variance_rate")]
    pub price_variance_rate: f64,
    /// Maximum price variance percentage
    #[serde(default = "default_max_price_variance")]
    pub max_price_variance_percent: f64,
    /// Rate of quantity variances between PO/GR and Invoice
    #[serde(default = "default_quantity_variance_rate")]
    pub quantity_variance_rate: f64,
    /// Average days from PO to goods receipt
    #[serde(default = "default_po_to_gr_days")]
    pub average_po_to_gr_days: u32,
    /// Average days from GR to invoice
    #[serde(default = "default_gr_to_invoice_days")]
    pub average_gr_to_invoice_days: u32,
    /// Average days from invoice to payment
    #[serde(default = "default_invoice_to_payment_days")]
    pub average_invoice_to_payment_days: u32,
    /// PO line count distribution
    #[serde(default)]
    pub line_count_distribution: DocumentLineCountDistribution,
    /// Payment behavior configuration
    #[serde(default)]
    pub payment_behavior: P2PPaymentBehaviorConfig,
}

fn default_three_way_match_rate() -> f64 {
    0.95
}

fn default_partial_delivery_rate() -> f64 {
    0.15
}

fn default_price_variance_rate() -> f64 {
    0.08
}

fn default_max_price_variance() -> f64 {
    0.05
}

fn default_quantity_variance_rate() -> f64 {
    0.05
}

fn default_po_to_gr_days() -> u32 {
    14
}

fn default_gr_to_invoice_days() -> u32 {
    5
}

fn default_invoice_to_payment_days() -> u32 {
    30
}

impl Default for P2PFlowConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            three_way_match_rate: default_three_way_match_rate(),
            partial_delivery_rate: default_partial_delivery_rate(),
            price_variance_rate: default_price_variance_rate(),
            max_price_variance_percent: default_max_price_variance(),
            quantity_variance_rate: default_quantity_variance_rate(),
            average_po_to_gr_days: default_po_to_gr_days(),
            average_gr_to_invoice_days: default_gr_to_invoice_days(),
            average_invoice_to_payment_days: default_invoice_to_payment_days(),
            line_count_distribution: DocumentLineCountDistribution::default(),
            payment_behavior: P2PPaymentBehaviorConfig::default(),
        }
    }
}

// ============================================================================
// P2P Payment Behavior Configuration
// ============================================================================

/// P2P payment behavior configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct P2PPaymentBehaviorConfig {
    /// Rate of late payments (beyond due date)
    #[serde(default = "default_p2p_late_payment_rate")]
    pub late_payment_rate: f64,
    /// Distribution of late payment days
    #[serde(default)]
    pub late_payment_days_distribution: LatePaymentDaysDistribution,
    /// Rate of partial payments
    #[serde(default = "default_p2p_partial_payment_rate")]
    pub partial_payment_rate: f64,
    /// Rate of payment corrections (NSF, chargebacks, reversals)
    #[serde(default = "default_p2p_payment_correction_rate")]
    pub payment_correction_rate: f64,
}

fn default_p2p_late_payment_rate() -> f64 {
    0.15
}

fn default_p2p_partial_payment_rate() -> f64 {
    0.05
}

fn default_p2p_payment_correction_rate() -> f64 {
    0.02
}

impl Default for P2PPaymentBehaviorConfig {
    fn default() -> Self {
        Self {
            late_payment_rate: default_p2p_late_payment_rate(),
            late_payment_days_distribution: LatePaymentDaysDistribution::default(),
            partial_payment_rate: default_p2p_partial_payment_rate(),
            payment_correction_rate: default_p2p_payment_correction_rate(),
        }
    }
}

/// Distribution of late payment days for P2P.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LatePaymentDaysDistribution {
    /// 1-7 days late (slightly late)
    #[serde(default = "default_slightly_late")]
    pub slightly_late_1_to_7: f64,
    /// 8-14 days late
    #[serde(default = "default_late_8_14")]
    pub late_8_to_14: f64,
    /// 15-30 days late (very late)
    #[serde(default = "default_very_late")]
    pub very_late_15_to_30: f64,
    /// 31-60 days late (severely late)
    #[serde(default = "default_severely_late")]
    pub severely_late_31_to_60: f64,
    /// Over 60 days late (extremely late)
    #[serde(default = "default_extremely_late")]
    pub extremely_late_over_60: f64,
}

fn default_slightly_late() -> f64 {
    0.50
}

fn default_late_8_14() -> f64 {
    0.25
}

fn default_very_late() -> f64 {
    0.15
}

fn default_severely_late() -> f64 {
    0.07
}

fn default_extremely_late() -> f64 {
    0.03
}

impl Default for LatePaymentDaysDistribution {
    fn default() -> Self {
        Self {
            slightly_late_1_to_7: default_slightly_late(),
            late_8_to_14: default_late_8_14(),
            very_late_15_to_30: default_very_late(),
            severely_late_31_to_60: default_severely_late(),
            extremely_late_over_60: default_extremely_late(),
        }
    }
}

/// O2C (Order-to-Cash) flow configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct O2CFlowConfig {
    /// Enable O2C document flow generation
    #[serde(default = "default_true")]
    pub enabled: bool,
    /// Credit check failure rate
    #[serde(default = "default_credit_check_failure_rate")]
    pub credit_check_failure_rate: f64,
    /// Rate of partial shipments
    #[serde(default = "default_partial_shipment_rate")]
    pub partial_shipment_rate: f64,
    /// Rate of returns
    #[serde(default = "default_return_rate")]
    pub return_rate: f64,
    /// Bad debt write-off rate
    #[serde(default = "default_bad_debt_rate")]
    pub bad_debt_rate: f64,
    /// Average days from SO to delivery
    #[serde(default = "default_so_to_delivery_days")]
    pub average_so_to_delivery_days: u32,
    /// Average days from delivery to invoice
    #[serde(default = "default_delivery_to_invoice_days")]
    pub average_delivery_to_invoice_days: u32,
    /// Average days from invoice to receipt
    #[serde(default = "default_invoice_to_receipt_days")]
    pub average_invoice_to_receipt_days: u32,
    /// SO line count distribution
    #[serde(default)]
    pub line_count_distribution: DocumentLineCountDistribution,
    /// Cash discount configuration
    #[serde(default)]
    pub cash_discount: CashDiscountConfig,
    /// Payment behavior configuration
    #[serde(default)]
    pub payment_behavior: O2CPaymentBehaviorConfig,
}

fn default_credit_check_failure_rate() -> f64 {
    0.02
}

fn default_partial_shipment_rate() -> f64 {
    0.10
}

fn default_return_rate() -> f64 {
    0.03
}

fn default_bad_debt_rate() -> f64 {
    0.01
}

fn default_so_to_delivery_days() -> u32 {
    7
}

fn default_delivery_to_invoice_days() -> u32 {
    1
}

fn default_invoice_to_receipt_days() -> u32 {
    45
}

impl Default for O2CFlowConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            credit_check_failure_rate: default_credit_check_failure_rate(),
            partial_shipment_rate: default_partial_shipment_rate(),
            return_rate: default_return_rate(),
            bad_debt_rate: default_bad_debt_rate(),
            average_so_to_delivery_days: default_so_to_delivery_days(),
            average_delivery_to_invoice_days: default_delivery_to_invoice_days(),
            average_invoice_to_receipt_days: default_invoice_to_receipt_days(),
            line_count_distribution: DocumentLineCountDistribution::default(),
            cash_discount: CashDiscountConfig::default(),
            payment_behavior: O2CPaymentBehaviorConfig::default(),
        }
    }
}

// ============================================================================
// O2C Payment Behavior Configuration
// ============================================================================

/// O2C payment behavior configuration.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct O2CPaymentBehaviorConfig {
    /// Dunning (Mahnung) configuration
    #[serde(default)]
    pub dunning: DunningConfig,
    /// Partial payment configuration
    #[serde(default)]
    pub partial_payments: PartialPaymentConfig,
    /// Short payment configuration (unauthorized deductions)
    #[serde(default)]
    pub short_payments: ShortPaymentConfig,
    /// On-account payment configuration (unapplied payments)
    #[serde(default)]
    pub on_account_payments: OnAccountPaymentConfig,
    /// Payment correction configuration (NSF, chargebacks)
    #[serde(default)]
    pub payment_corrections: PaymentCorrectionConfig,
}

/// Dunning (Mahnungen) configuration for AR collections.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DunningConfig {
    /// Enable dunning process
    #[serde(default)]
    pub enabled: bool,
    /// Days overdue for level 1 dunning (1st reminder)
    #[serde(default = "default_dunning_level_1_days")]
    pub level_1_days_overdue: u32,
    /// Days overdue for level 2 dunning (2nd reminder)
    #[serde(default = "default_dunning_level_2_days")]
    pub level_2_days_overdue: u32,
    /// Days overdue for level 3 dunning (final notice)
    #[serde(default = "default_dunning_level_3_days")]
    pub level_3_days_overdue: u32,
    /// Days overdue for collection handover
    #[serde(default = "default_collection_days")]
    pub collection_days_overdue: u32,
    /// Payment rates after each dunning level
    #[serde(default)]
    pub payment_after_dunning_rates: DunningPaymentRates,
    /// Rate of invoices blocked from dunning (disputes)
    #[serde(default = "default_dunning_block_rate")]
    pub dunning_block_rate: f64,
    /// Interest rate per year for overdue amounts
    #[serde(default = "default_dunning_interest_rate")]
    pub interest_rate_per_year: f64,
    /// Fixed dunning charge per letter
    #[serde(default = "default_dunning_charge")]
    pub dunning_charge: f64,
}

fn default_dunning_level_1_days() -> u32 {
    14
}

fn default_dunning_level_2_days() -> u32 {
    28
}

fn default_dunning_level_3_days() -> u32 {
    42
}

fn default_collection_days() -> u32 {
    60
}

fn default_dunning_block_rate() -> f64 {
    0.05
}

fn default_dunning_interest_rate() -> f64 {
    0.09
}

fn default_dunning_charge() -> f64 {
    25.0
}

impl Default for DunningConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            level_1_days_overdue: default_dunning_level_1_days(),
            level_2_days_overdue: default_dunning_level_2_days(),
            level_3_days_overdue: default_dunning_level_3_days(),
            collection_days_overdue: default_collection_days(),
            payment_after_dunning_rates: DunningPaymentRates::default(),
            dunning_block_rate: default_dunning_block_rate(),
            interest_rate_per_year: default_dunning_interest_rate(),
            dunning_charge: default_dunning_charge(),
        }
    }
}

/// Payment rates after each dunning level.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DunningPaymentRates {
    /// Rate that pays after level 1 reminder
    #[serde(default = "default_after_level_1")]
    pub after_level_1: f64,
    /// Rate that pays after level 2 reminder
    #[serde(default = "default_after_level_2")]
    pub after_level_2: f64,
    /// Rate that pays after level 3 final notice
    #[serde(default = "default_after_level_3")]
    pub after_level_3: f64,
    /// Rate that pays during collection
    #[serde(default = "default_during_collection")]
    pub during_collection: f64,
    /// Rate that never pays (becomes bad debt)
    #[serde(default = "default_never_pay")]
    pub never_pay: f64,
}

fn default_after_level_1() -> f64 {
    0.40
}

fn default_after_level_2() -> f64 {
    0.30
}

fn default_after_level_3() -> f64 {
    0.15
}

fn default_during_collection() -> f64 {
    0.05
}

fn default_never_pay() -> f64 {
    0.10
}

impl Default for DunningPaymentRates {
    fn default() -> Self {
        Self {
            after_level_1: default_after_level_1(),
            after_level_2: default_after_level_2(),
            after_level_3: default_after_level_3(),
            during_collection: default_during_collection(),
            never_pay: default_never_pay(),
        }
    }
}

/// Partial payment configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PartialPaymentConfig {
    /// Rate of invoices paid partially
    #[serde(default = "default_partial_payment_rate")]
    pub rate: f64,
    /// Distribution of partial payment percentages
    #[serde(default)]
    pub percentage_distribution: PartialPaymentPercentageDistribution,
    /// Average days until remainder is paid
    #[serde(default = "default_avg_days_until_remainder")]
    pub avg_days_until_remainder: u32,
}

fn default_partial_payment_rate() -> f64 {
    0.08
}

fn default_avg_days_until_remainder() -> u32 {
    30
}

impl Default for PartialPaymentConfig {
    fn default() -> Self {
        Self {
            rate: default_partial_payment_rate(),
            percentage_distribution: PartialPaymentPercentageDistribution::default(),
            avg_days_until_remainder: default_avg_days_until_remainder(),
        }
    }
}

/// Distribution of partial payment percentages.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PartialPaymentPercentageDistribution {
    /// Pay 25% of invoice
    #[serde(default = "default_partial_25")]
    pub pay_25_percent: f64,
    /// Pay 50% of invoice
    #[serde(default = "default_partial_50")]
    pub pay_50_percent: f64,
    /// Pay 75% of invoice
    #[serde(default = "default_partial_75")]
    pub pay_75_percent: f64,
    /// Pay random percentage
    #[serde(default = "default_partial_random")]
    pub pay_random_percent: f64,
}

fn default_partial_25() -> f64 {
    0.15
}

fn default_partial_50() -> f64 {
    0.50
}

fn default_partial_75() -> f64 {
    0.25
}

fn default_partial_random() -> f64 {
    0.10
}

impl Default for PartialPaymentPercentageDistribution {
    fn default() -> Self {
        Self {
            pay_25_percent: default_partial_25(),
            pay_50_percent: default_partial_50(),
            pay_75_percent: default_partial_75(),
            pay_random_percent: default_partial_random(),
        }
    }
}

/// Short payment configuration (unauthorized deductions).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShortPaymentConfig {
    /// Rate of payments that are short
    #[serde(default = "default_short_payment_rate")]
    pub rate: f64,
    /// Distribution of short payment reasons
    #[serde(default)]
    pub reason_distribution: ShortPaymentReasonDistribution,
    /// Maximum percentage that can be short
    #[serde(default = "default_max_short_percent")]
    pub max_short_percent: f64,
}

fn default_short_payment_rate() -> f64 {
    0.03
}

fn default_max_short_percent() -> f64 {
    0.10
}

impl Default for ShortPaymentConfig {
    fn default() -> Self {
        Self {
            rate: default_short_payment_rate(),
            reason_distribution: ShortPaymentReasonDistribution::default(),
            max_short_percent: default_max_short_percent(),
        }
    }
}

/// Distribution of short payment reasons.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShortPaymentReasonDistribution {
    /// Pricing dispute
    #[serde(default = "default_pricing_dispute")]
    pub pricing_dispute: f64,
    /// Quality issue
    #[serde(default = "default_quality_issue")]
    pub quality_issue: f64,
    /// Quantity discrepancy
    #[serde(default = "default_quantity_discrepancy")]
    pub quantity_discrepancy: f64,
    /// Unauthorized deduction
    #[serde(default = "default_unauthorized_deduction")]
    pub unauthorized_deduction: f64,
    /// Early payment discount taken incorrectly
    #[serde(default = "default_incorrect_discount")]
    pub incorrect_discount: f64,
}

fn default_pricing_dispute() -> f64 {
    0.30
}

fn default_quality_issue() -> f64 {
    0.20
}

fn default_quantity_discrepancy() -> f64 {
    0.20
}

fn default_unauthorized_deduction() -> f64 {
    0.15
}

fn default_incorrect_discount() -> f64 {
    0.15
}

impl Default for ShortPaymentReasonDistribution {
    fn default() -> Self {
        Self {
            pricing_dispute: default_pricing_dispute(),
            quality_issue: default_quality_issue(),
            quantity_discrepancy: default_quantity_discrepancy(),
            unauthorized_deduction: default_unauthorized_deduction(),
            incorrect_discount: default_incorrect_discount(),
        }
    }
}

/// On-account payment configuration (unapplied payments).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OnAccountPaymentConfig {
    /// Rate of payments that are on-account (unapplied)
    #[serde(default = "default_on_account_rate")]
    pub rate: f64,
    /// Average days until on-account payments are applied
    #[serde(default = "default_avg_days_until_applied")]
    pub avg_days_until_applied: u32,
}

fn default_on_account_rate() -> f64 {
    0.02
}

fn default_avg_days_until_applied() -> u32 {
    14
}

impl Default for OnAccountPaymentConfig {
    fn default() -> Self {
        Self {
            rate: default_on_account_rate(),
            avg_days_until_applied: default_avg_days_until_applied(),
        }
    }
}

/// Payment correction configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaymentCorrectionConfig {
    /// Rate of payments requiring correction
    #[serde(default = "default_payment_correction_rate")]
    pub rate: f64,
    /// Distribution of correction types
    #[serde(default)]
    pub type_distribution: PaymentCorrectionTypeDistribution,
}

fn default_payment_correction_rate() -> f64 {
    0.02
}

impl Default for PaymentCorrectionConfig {
    fn default() -> Self {
        Self {
            rate: default_payment_correction_rate(),
            type_distribution: PaymentCorrectionTypeDistribution::default(),
        }
    }
}

/// Distribution of payment correction types.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaymentCorrectionTypeDistribution {
    /// NSF (Non-sufficient funds) / bounced check
    #[serde(default = "default_nsf_rate")]
    pub nsf: f64,
    /// Chargeback
    #[serde(default = "default_chargeback_rate")]
    pub chargeback: f64,
    /// Wrong amount applied
    #[serde(default = "default_wrong_amount_rate")]
    pub wrong_amount: f64,
    /// Wrong customer applied
    #[serde(default = "default_wrong_customer_rate")]
    pub wrong_customer: f64,
    /// Duplicate payment
    #[serde(default = "default_duplicate_payment_rate")]
    pub duplicate_payment: f64,
}

fn default_nsf_rate() -> f64 {
    0.30
}

fn default_chargeback_rate() -> f64 {
    0.20
}

fn default_wrong_amount_rate() -> f64 {
    0.20
}

fn default_wrong_customer_rate() -> f64 {
    0.15
}

fn default_duplicate_payment_rate() -> f64 {
    0.15
}

impl Default for PaymentCorrectionTypeDistribution {
    fn default() -> Self {
        Self {
            nsf: default_nsf_rate(),
            chargeback: default_chargeback_rate(),
            wrong_amount: default_wrong_amount_rate(),
            wrong_customer: default_wrong_customer_rate(),
            duplicate_payment: default_duplicate_payment_rate(),
        }
    }
}

/// Document line count distribution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentLineCountDistribution {
    /// Minimum number of lines
    #[serde(default = "default_min_lines")]
    pub min_lines: u32,
    /// Maximum number of lines
    #[serde(default = "default_max_lines")]
    pub max_lines: u32,
    /// Most common line count (mode)
    #[serde(default = "default_mode_lines")]
    pub mode_lines: u32,
}

fn default_min_lines() -> u32 {
    1
}

fn default_max_lines() -> u32 {
    20
}

fn default_mode_lines() -> u32 {
    3
}

impl Default for DocumentLineCountDistribution {
    fn default() -> Self {
        Self {
            min_lines: default_min_lines(),
            max_lines: default_max_lines(),
            mode_lines: default_mode_lines(),
        }
    }
}

/// Cash discount configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CashDiscountConfig {
    /// Percentage of invoices eligible for cash discount
    #[serde(default = "default_discount_eligible_rate")]
    pub eligible_rate: f64,
    /// Rate at which customers take the discount
    #[serde(default = "default_discount_taken_rate")]
    pub taken_rate: f64,
    /// Standard discount percentage
    #[serde(default = "default_discount_percent")]
    pub discount_percent: f64,
    /// Days within which discount must be taken
    #[serde(default = "default_discount_days")]
    pub discount_days: u32,
}

fn default_discount_eligible_rate() -> f64 {
    0.30
}

fn default_discount_taken_rate() -> f64 {
    0.60
}

fn default_discount_percent() -> f64 {
    0.02
}

fn default_discount_days() -> u32 {
    10
}

impl Default for CashDiscountConfig {
    fn default() -> Self {
        Self {
            eligible_rate: default_discount_eligible_rate(),
            taken_rate: default_discount_taken_rate(),
            discount_percent: default_discount_percent(),
            discount_days: default_discount_days(),
        }
    }
}

// ============================================================================
// Intercompany Configuration
// ============================================================================

/// Intercompany transaction configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntercompanyConfig {
    /// Enable intercompany transaction generation
    #[serde(default)]
    pub enabled: bool,
    /// Rate of transactions that are intercompany
    #[serde(default = "default_ic_transaction_rate")]
    pub ic_transaction_rate: f64,
    /// Transfer pricing method
    #[serde(default)]
    pub transfer_pricing_method: TransferPricingMethod,
    /// Transfer pricing markup percentage (for cost-plus)
    #[serde(default = "default_markup_percent")]
    pub markup_percent: f64,
    /// Generate matched IC pairs (offsetting entries)
    #[serde(default = "default_true")]
    pub generate_matched_pairs: bool,
    /// IC transaction type distribution
    #[serde(default)]
    pub transaction_type_distribution: ICTransactionTypeDistribution,
    /// Generate elimination entries for consolidation
    #[serde(default)]
    pub generate_eliminations: bool,
}

fn default_ic_transaction_rate() -> f64 {
    0.15
}

fn default_markup_percent() -> f64 {
    0.05
}

impl Default for IntercompanyConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            ic_transaction_rate: default_ic_transaction_rate(),
            transfer_pricing_method: TransferPricingMethod::default(),
            markup_percent: default_markup_percent(),
            generate_matched_pairs: true,
            transaction_type_distribution: ICTransactionTypeDistribution::default(),
            generate_eliminations: false,
        }
    }
}

/// Transfer pricing method.
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TransferPricingMethod {
    /// Cost plus a markup
    #[default]
    CostPlus,
    /// Comparable uncontrolled price
    ComparableUncontrolled,
    /// Resale price method
    ResalePrice,
    /// Transactional net margin method
    TransactionalNetMargin,
    /// Profit split method
    ProfitSplit,
}

/// IC transaction type distribution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ICTransactionTypeDistribution {
    /// Goods sales between entities
    pub goods_sale: f64,
    /// Services provided
    pub service_provided: f64,
    /// Intercompany loans
    pub loan: f64,
    /// Dividends
    pub dividend: f64,
    /// Management fees
    pub management_fee: f64,
    /// Royalties
    pub royalty: f64,
    /// Cost sharing
    pub cost_sharing: f64,
}

impl Default for ICTransactionTypeDistribution {
    fn default() -> Self {
        Self {
            goods_sale: 0.35,
            service_provided: 0.20,
            loan: 0.10,
            dividend: 0.05,
            management_fee: 0.15,
            royalty: 0.10,
            cost_sharing: 0.05,
        }
    }
}

// ============================================================================
// Balance Configuration
// ============================================================================

/// Balance and trial balance configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BalanceConfig {
    /// Generate opening balances
    #[serde(default)]
    pub generate_opening_balances: bool,
    /// Generate trial balances
    #[serde(default = "default_true")]
    pub generate_trial_balances: bool,
    /// Target gross margin (for revenue/COGS coherence)
    #[serde(default = "default_gross_margin")]
    pub target_gross_margin: f64,
    /// Target DSO (Days Sales Outstanding)
    #[serde(default = "default_dso")]
    pub target_dso_days: u32,
    /// Target DPO (Days Payable Outstanding)
    #[serde(default = "default_dpo")]
    pub target_dpo_days: u32,
    /// Target current ratio
    #[serde(default = "default_current_ratio")]
    pub target_current_ratio: f64,
    /// Target debt-to-equity ratio
    #[serde(default = "default_debt_equity")]
    pub target_debt_to_equity: f64,
    /// Validate balance sheet equation (A = L + E)
    #[serde(default = "default_true")]
    pub validate_balance_equation: bool,
    /// Reconcile subledgers to GL control accounts
    #[serde(default = "default_true")]
    pub reconcile_subledgers: bool,
}

fn default_gross_margin() -> f64 {
    0.35
}

fn default_dso() -> u32 {
    45
}

fn default_dpo() -> u32 {
    30
}

fn default_current_ratio() -> f64 {
    1.5
}

fn default_debt_equity() -> f64 {
    0.5
}

impl Default for BalanceConfig {
    fn default() -> Self {
        Self {
            generate_opening_balances: false,
            generate_trial_balances: true,
            target_gross_margin: default_gross_margin(),
            target_dso_days: default_dso(),
            target_dpo_days: default_dpo(),
            target_current_ratio: default_current_ratio(),
            target_debt_to_equity: default_debt_equity(),
            validate_balance_equation: true,
            reconcile_subledgers: true,
        }
    }
}

// ==========================================================================
// OCPM (Object-Centric Process Mining) Configuration
// ==========================================================================

/// OCPM (Object-Centric Process Mining) configuration.
///
/// Controls generation of OCEL 2.0 compatible event logs with
/// many-to-many event-to-object relationships.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OcpmConfig {
    /// Enable OCPM event log generation
    #[serde(default)]
    pub enabled: bool,

    /// Generate lifecycle events (Start/Complete pairs vs atomic events)
    #[serde(default = "default_true")]
    pub generate_lifecycle_events: bool,

    /// Include object-to-object relationships in output
    #[serde(default = "default_true")]
    pub include_object_relationships: bool,

    /// Compute and export process variants
    #[serde(default = "default_true")]
    pub compute_variants: bool,

    /// Maximum variants to track (0 = unlimited)
    #[serde(default)]
    pub max_variants: usize,

    /// P2P process configuration
    #[serde(default)]
    pub p2p_process: OcpmProcessConfig,

    /// O2C process configuration
    #[serde(default)]
    pub o2c_process: OcpmProcessConfig,

    /// Output format configuration
    #[serde(default)]
    pub output: OcpmOutputConfig,
}

impl Default for OcpmConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            generate_lifecycle_events: true,
            include_object_relationships: true,
            compute_variants: true,
            max_variants: 0,
            p2p_process: OcpmProcessConfig::default(),
            o2c_process: OcpmProcessConfig::default(),
            output: OcpmOutputConfig::default(),
        }
    }
}

/// Process-specific OCPM configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OcpmProcessConfig {
    /// Rework probability (0.0-1.0)
    #[serde(default = "default_rework_probability")]
    pub rework_probability: f64,

    /// Skip step probability (0.0-1.0)
    #[serde(default = "default_skip_probability")]
    pub skip_step_probability: f64,

    /// Out-of-order step probability (0.0-1.0)
    #[serde(default = "default_out_of_order_probability")]
    pub out_of_order_probability: f64,
}

fn default_rework_probability() -> f64 {
    0.05
}

fn default_skip_probability() -> f64 {
    0.02
}

fn default_out_of_order_probability() -> f64 {
    0.03
}

impl Default for OcpmProcessConfig {
    fn default() -> Self {
        Self {
            rework_probability: default_rework_probability(),
            skip_step_probability: default_skip_probability(),
            out_of_order_probability: default_out_of_order_probability(),
        }
    }
}

/// OCPM output format configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OcpmOutputConfig {
    /// Export OCEL 2.0 JSON format
    #[serde(default = "default_true")]
    pub ocel_json: bool,

    /// Export OCEL 2.0 XML format
    #[serde(default)]
    pub ocel_xml: bool,

    /// Export flattened CSV for each object type
    #[serde(default = "default_true")]
    pub flattened_csv: bool,

    /// Export event-object relationship table
    #[serde(default = "default_true")]
    pub event_object_csv: bool,

    /// Export object-object relationship table
    #[serde(default = "default_true")]
    pub object_relationship_csv: bool,

    /// Export process variants summary
    #[serde(default = "default_true")]
    pub variants_csv: bool,
}

impl Default for OcpmOutputConfig {
    fn default() -> Self {
        Self {
            ocel_json: true,
            ocel_xml: false,
            flattened_csv: true,
            event_object_csv: true,
            object_relationship_csv: true,
            variants_csv: true,
        }
    }
}

/// Audit engagement and workpaper generation configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditGenerationConfig {
    /// Enable audit engagement generation
    #[serde(default)]
    pub enabled: bool,

    /// Generate engagement documents and workpapers
    #[serde(default = "default_true")]
    pub generate_workpapers: bool,

    /// Default engagement type distribution
    #[serde(default)]
    pub engagement_types: AuditEngagementTypesConfig,

    /// Workpaper configuration
    #[serde(default)]
    pub workpapers: WorkpaperConfig,

    /// Team configuration
    #[serde(default)]
    pub team: AuditTeamConfig,

    /// Review workflow configuration
    #[serde(default)]
    pub review: ReviewWorkflowConfig,
}

impl Default for AuditGenerationConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            generate_workpapers: true,
            engagement_types: AuditEngagementTypesConfig::default(),
            workpapers: WorkpaperConfig::default(),
            team: AuditTeamConfig::default(),
            review: ReviewWorkflowConfig::default(),
        }
    }
}

/// Engagement type distribution configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEngagementTypesConfig {
    /// Financial statement audit probability
    #[serde(default = "default_financial_audit_prob")]
    pub financial_statement: f64,
    /// SOX/ICFR audit probability
    #[serde(default = "default_sox_audit_prob")]
    pub sox_icfr: f64,
    /// Integrated audit probability
    #[serde(default = "default_integrated_audit_prob")]
    pub integrated: f64,
    /// Review engagement probability
    #[serde(default = "default_review_prob")]
    pub review: f64,
    /// Agreed-upon procedures probability
    #[serde(default = "default_aup_prob")]
    pub agreed_upon_procedures: f64,
}

fn default_financial_audit_prob() -> f64 {
    0.40
}
fn default_sox_audit_prob() -> f64 {
    0.20
}
fn default_integrated_audit_prob() -> f64 {
    0.25
}
fn default_review_prob() -> f64 {
    0.10
}
fn default_aup_prob() -> f64 {
    0.05
}

impl Default for AuditEngagementTypesConfig {
    fn default() -> Self {
        Self {
            financial_statement: default_financial_audit_prob(),
            sox_icfr: default_sox_audit_prob(),
            integrated: default_integrated_audit_prob(),
            review: default_review_prob(),
            agreed_upon_procedures: default_aup_prob(),
        }
    }
}

/// Workpaper generation configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkpaperConfig {
    /// Average workpapers per engagement phase
    #[serde(default = "default_workpapers_per_phase")]
    pub average_per_phase: usize,

    /// Include ISA compliance references
    #[serde(default = "default_true")]
    pub include_isa_references: bool,

    /// Generate sample details
    #[serde(default = "default_true")]
    pub include_sample_details: bool,

    /// Include cross-references between workpapers
    #[serde(default = "default_true")]
    pub include_cross_references: bool,

    /// Sampling configuration
    #[serde(default)]
    pub sampling: SamplingConfig,
}

fn default_workpapers_per_phase() -> usize {
    5
}

impl Default for WorkpaperConfig {
    fn default() -> Self {
        Self {
            average_per_phase: default_workpapers_per_phase(),
            include_isa_references: true,
            include_sample_details: true,
            include_cross_references: true,
            sampling: SamplingConfig::default(),
        }
    }
}

/// Sampling method configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SamplingConfig {
    /// Statistical sampling rate (0.0-1.0)
    #[serde(default = "default_statistical_rate")]
    pub statistical_rate: f64,
    /// Judgmental sampling rate (0.0-1.0)
    #[serde(default = "default_judgmental_rate")]
    pub judgmental_rate: f64,
    /// Haphazard sampling rate (0.0-1.0)
    #[serde(default = "default_haphazard_rate")]
    pub haphazard_rate: f64,
    /// 100% examination rate (0.0-1.0)
    #[serde(default = "default_complete_examination_rate")]
    pub complete_examination_rate: f64,
}

fn default_statistical_rate() -> f64 {
    0.40
}
fn default_judgmental_rate() -> f64 {
    0.30
}
fn default_haphazard_rate() -> f64 {
    0.20
}
fn default_complete_examination_rate() -> f64 {
    0.10
}

impl Default for SamplingConfig {
    fn default() -> Self {
        Self {
            statistical_rate: default_statistical_rate(),
            judgmental_rate: default_judgmental_rate(),
            haphazard_rate: default_haphazard_rate(),
            complete_examination_rate: default_complete_examination_rate(),
        }
    }
}

/// Audit team configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditTeamConfig {
    /// Minimum team size
    #[serde(default = "default_min_team_size")]
    pub min_team_size: usize,
    /// Maximum team size
    #[serde(default = "default_max_team_size")]
    pub max_team_size: usize,
    /// Probability of having a specialist on the team
    #[serde(default = "default_specialist_probability")]
    pub specialist_probability: f64,
}

fn default_min_team_size() -> usize {
    3
}
fn default_max_team_size() -> usize {
    8
}
fn default_specialist_probability() -> f64 {
    0.30
}

impl Default for AuditTeamConfig {
    fn default() -> Self {
        Self {
            min_team_size: default_min_team_size(),
            max_team_size: default_max_team_size(),
            specialist_probability: default_specialist_probability(),
        }
    }
}

/// Review workflow configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewWorkflowConfig {
    /// Average days between preparer completion and first review
    #[serde(default = "default_review_delay_days")]
    pub average_review_delay_days: u32,
    /// Probability of review notes requiring rework
    #[serde(default = "default_rework_probability_review")]
    pub rework_probability: f64,
    /// Require partner sign-off for all workpapers
    #[serde(default = "default_true")]
    pub require_partner_signoff: bool,
}

fn default_review_delay_days() -> u32 {
    2
}
fn default_rework_probability_review() -> f64 {
    0.15
}

impl Default for ReviewWorkflowConfig {
    fn default() -> Self {
        Self {
            average_review_delay_days: default_review_delay_days(),
            rework_probability: default_rework_probability_review(),
            require_partner_signoff: true,
        }
    }
}

// =============================================================================
// Data Quality Configuration
// =============================================================================

/// Data quality variation settings for realistic flakiness injection.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataQualitySchemaConfig {
    /// Enable data quality variations
    #[serde(default)]
    pub enabled: bool,
    /// Preset to use (overrides individual settings if set)
    #[serde(default)]
    pub preset: DataQualityPreset,
    /// Missing value injection settings
    #[serde(default)]
    pub missing_values: MissingValuesSchemaConfig,
    /// Typo injection settings
    #[serde(default)]
    pub typos: TypoSchemaConfig,
    /// Format variation settings
    #[serde(default)]
    pub format_variations: FormatVariationSchemaConfig,
    /// Duplicate injection settings
    #[serde(default)]
    pub duplicates: DuplicateSchemaConfig,
    /// Encoding issue settings
    #[serde(default)]
    pub encoding_issues: EncodingIssueSchemaConfig,
    /// Generate quality issue labels for ML training
    #[serde(default)]
    pub generate_labels: bool,
    /// Per-sink quality profiles (different settings for CSV vs JSON etc.)
    #[serde(default)]
    pub sink_profiles: SinkQualityProfiles,
}

impl Default for DataQualitySchemaConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            preset: DataQualityPreset::None,
            missing_values: MissingValuesSchemaConfig::default(),
            typos: TypoSchemaConfig::default(),
            format_variations: FormatVariationSchemaConfig::default(),
            duplicates: DuplicateSchemaConfig::default(),
            encoding_issues: EncodingIssueSchemaConfig::default(),
            generate_labels: true,
            sink_profiles: SinkQualityProfiles::default(),
        }
    }
}

/// Preset configurations for common data quality scenarios.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum DataQualityPreset {
    /// No data quality variations (clean data)
    #[default]
    None,
    /// Minimal variations (very clean data with rare issues)
    Minimal,
    /// Normal variations (realistic enterprise data quality)
    Normal,
    /// High variations (messy data for stress testing)
    High,
    /// Custom (use individual settings)
    Custom,
}

/// Missing value injection configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MissingValuesSchemaConfig {
    /// Enable missing value injection
    #[serde(default)]
    pub enabled: bool,
    /// Global missing rate (0.0 to 1.0)
    #[serde(default = "default_missing_rate")]
    pub rate: f64,
    /// Missing value strategy
    #[serde(default)]
    pub strategy: MissingValueStrategy,
    /// Field-specific rates (field name -> rate)
    #[serde(default)]
    pub field_rates: std::collections::HashMap<String, f64>,
    /// Fields that should never have missing values
    #[serde(default)]
    pub protected_fields: Vec<String>,
}

fn default_missing_rate() -> f64 {
    0.01
}

impl Default for MissingValuesSchemaConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            rate: default_missing_rate(),
            strategy: MissingValueStrategy::Mcar,
            field_rates: std::collections::HashMap::new(),
            protected_fields: vec![
                "document_id".to_string(),
                "company_code".to_string(),
                "posting_date".to_string(),
            ],
        }
    }
}

/// Missing value strategy types.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum MissingValueStrategy {
    /// Missing Completely At Random - equal probability for all values
    #[default]
    Mcar,
    /// Missing At Random - depends on other observed values
    Mar,
    /// Missing Not At Random - depends on the value itself
    Mnar,
    /// Systematic - entire field groups missing together
    Systematic,
}

/// Typo injection configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TypoSchemaConfig {
    /// Enable typo injection
    #[serde(default)]
    pub enabled: bool,
    /// Character error rate (per character, not per field)
    #[serde(default = "default_typo_rate")]
    pub char_error_rate: f64,
    /// Typo type weights
    #[serde(default)]
    pub type_weights: TypoTypeWeights,
    /// Fields that should never have typos
    #[serde(default)]
    pub protected_fields: Vec<String>,
}

fn default_typo_rate() -> f64 {
    0.001
}

impl Default for TypoSchemaConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            char_error_rate: default_typo_rate(),
            type_weights: TypoTypeWeights::default(),
            protected_fields: vec![
                "document_id".to_string(),
                "gl_account".to_string(),
                "company_code".to_string(),
            ],
        }
    }
}

/// Weights for different typo types.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TypoTypeWeights {
    /// Keyboard-adjacent substitution (e.g., 'a' -> 's')
    #[serde(default = "default_substitution_weight")]
    pub substitution: f64,
    /// Adjacent character transposition (e.g., 'ab' -> 'ba')
    #[serde(default = "default_transposition_weight")]
    pub transposition: f64,
    /// Character insertion
    #[serde(default = "default_insertion_weight")]
    pub insertion: f64,
    /// Character deletion
    #[serde(default = "default_deletion_weight")]
    pub deletion: f64,
    /// OCR-style errors (e.g., '0' -> 'O')
    #[serde(default = "default_ocr_weight")]
    pub ocr_errors: f64,
    /// Homophone substitution (e.g., 'their' -> 'there')
    #[serde(default = "default_homophone_weight")]
    pub homophones: f64,
}

fn default_substitution_weight() -> f64 {
    0.35
}
fn default_transposition_weight() -> f64 {
    0.25
}
fn default_insertion_weight() -> f64 {
    0.10
}
fn default_deletion_weight() -> f64 {
    0.15
}
fn default_ocr_weight() -> f64 {
    0.10
}
fn default_homophone_weight() -> f64 {
    0.05
}

impl Default for TypoTypeWeights {
    fn default() -> Self {
        Self {
            substitution: default_substitution_weight(),
            transposition: default_transposition_weight(),
            insertion: default_insertion_weight(),
            deletion: default_deletion_weight(),
            ocr_errors: default_ocr_weight(),
            homophones: default_homophone_weight(),
        }
    }
}

/// Format variation configuration.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct FormatVariationSchemaConfig {
    /// Enable format variations
    #[serde(default)]
    pub enabled: bool,
    /// Date format variation settings
    #[serde(default)]
    pub dates: DateFormatVariationConfig,
    /// Amount format variation settings
    #[serde(default)]
    pub amounts: AmountFormatVariationConfig,
    /// Identifier format variation settings
    #[serde(default)]
    pub identifiers: IdentifierFormatVariationConfig,
}

/// Date format variation configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DateFormatVariationConfig {
    /// Enable date format variations
    #[serde(default)]
    pub enabled: bool,
    /// Overall variation rate
    #[serde(default = "default_date_variation_rate")]
    pub rate: f64,
    /// Include ISO format (2024-01-15)
    #[serde(default = "default_true")]
    pub iso_format: bool,
    /// Include US format (01/15/2024)
    #[serde(default)]
    pub us_format: bool,
    /// Include EU format (15.01.2024)
    #[serde(default)]
    pub eu_format: bool,
    /// Include long format (January 15, 2024)
    #[serde(default)]
    pub long_format: bool,
}

fn default_date_variation_rate() -> f64 {
    0.05
}

impl Default for DateFormatVariationConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            rate: default_date_variation_rate(),
            iso_format: true,
            us_format: false,
            eu_format: false,
            long_format: false,
        }
    }
}

/// Amount format variation configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AmountFormatVariationConfig {
    /// Enable amount format variations
    #[serde(default)]
    pub enabled: bool,
    /// Overall variation rate
    #[serde(default = "default_amount_variation_rate")]
    pub rate: f64,
    /// Include US comma format (1,234.56)
    #[serde(default)]
    pub us_comma_format: bool,
    /// Include EU format (1.234,56)
    #[serde(default)]
    pub eu_format: bool,
    /// Include currency prefix ($1,234.56)
    #[serde(default)]
    pub currency_prefix: bool,
    /// Include accounting format with parentheses for negatives
    #[serde(default)]
    pub accounting_format: bool,
}

fn default_amount_variation_rate() -> f64 {
    0.02
}

impl Default for AmountFormatVariationConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            rate: default_amount_variation_rate(),
            us_comma_format: false,
            eu_format: false,
            currency_prefix: false,
            accounting_format: false,
        }
    }
}

/// Identifier format variation configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdentifierFormatVariationConfig {
    /// Enable identifier format variations
    #[serde(default)]
    pub enabled: bool,
    /// Overall variation rate
    #[serde(default = "default_identifier_variation_rate")]
    pub rate: f64,
    /// Case variations (uppercase, lowercase, mixed)
    #[serde(default)]
    pub case_variations: bool,
    /// Padding variations (leading zeros)
    #[serde(default)]
    pub padding_variations: bool,
    /// Separator variations (dash vs underscore)
    #[serde(default)]
    pub separator_variations: bool,
}

fn default_identifier_variation_rate() -> f64 {
    0.02
}

impl Default for IdentifierFormatVariationConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            rate: default_identifier_variation_rate(),
            case_variations: false,
            padding_variations: false,
            separator_variations: false,
        }
    }
}

/// Duplicate injection configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DuplicateSchemaConfig {
    /// Enable duplicate injection
    #[serde(default)]
    pub enabled: bool,
    /// Overall duplicate rate
    #[serde(default = "default_duplicate_rate")]
    pub rate: f64,
    /// Exact duplicate proportion (out of duplicates)
    #[serde(default = "default_exact_duplicate_ratio")]
    pub exact_duplicate_ratio: f64,
    /// Near duplicate proportion (slight variations)
    #[serde(default = "default_near_duplicate_ratio")]
    pub near_duplicate_ratio: f64,
    /// Fuzzy duplicate proportion (typos in key fields)
    #[serde(default = "default_fuzzy_duplicate_ratio")]
    pub fuzzy_duplicate_ratio: f64,
    /// Maximum date offset for near/fuzzy duplicates (days)
    #[serde(default = "default_max_date_offset")]
    pub max_date_offset_days: u32,
    /// Maximum amount variance for near duplicates (fraction)
    #[serde(default = "default_max_amount_variance")]
    pub max_amount_variance: f64,
}

fn default_duplicate_rate() -> f64 {
    0.005
}
fn default_exact_duplicate_ratio() -> f64 {
    0.4
}
fn default_near_duplicate_ratio() -> f64 {
    0.35
}
fn default_fuzzy_duplicate_ratio() -> f64 {
    0.25
}
fn default_max_date_offset() -> u32 {
    3
}
fn default_max_amount_variance() -> f64 {
    0.01
}

impl Default for DuplicateSchemaConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            rate: default_duplicate_rate(),
            exact_duplicate_ratio: default_exact_duplicate_ratio(),
            near_duplicate_ratio: default_near_duplicate_ratio(),
            fuzzy_duplicate_ratio: default_fuzzy_duplicate_ratio(),
            max_date_offset_days: default_max_date_offset(),
            max_amount_variance: default_max_amount_variance(),
        }
    }
}

/// Encoding issue configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncodingIssueSchemaConfig {
    /// Enable encoding issue injection
    #[serde(default)]
    pub enabled: bool,
    /// Overall encoding issue rate
    #[serde(default = "default_encoding_rate")]
    pub rate: f64,
    /// Include mojibake (UTF-8/Latin-1 confusion)
    #[serde(default)]
    pub mojibake: bool,
    /// Include HTML entity corruption
    #[serde(default)]
    pub html_entities: bool,
    /// Include BOM issues
    #[serde(default)]
    pub bom_issues: bool,
}

fn default_encoding_rate() -> f64 {
    0.001
}

impl Default for EncodingIssueSchemaConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            rate: default_encoding_rate(),
            mojibake: false,
            html_entities: false,
            bom_issues: false,
        }
    }
}

/// Per-sink quality profiles for different output formats.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SinkQualityProfiles {
    /// CSV-specific quality settings
    #[serde(default)]
    pub csv: Option<SinkQualityOverride>,
    /// JSON-specific quality settings
    #[serde(default)]
    pub json: Option<SinkQualityOverride>,
    /// Parquet-specific quality settings
    #[serde(default)]
    pub parquet: Option<SinkQualityOverride>,
}

/// Quality setting overrides for a specific sink type.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SinkQualityOverride {
    /// Override enabled state
    pub enabled: Option<bool>,
    /// Override missing value rate
    pub missing_rate: Option<f64>,
    /// Override typo rate
    pub typo_rate: Option<f64>,
    /// Override format variation rate
    pub format_variation_rate: Option<f64>,
    /// Override duplicate rate
    pub duplicate_rate: Option<f64>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::presets::demo_preset;

    // ==========================================================================
    // Serialization/Deserialization Tests
    // ==========================================================================

    #[test]
    fn test_config_yaml_roundtrip() {
        let config = demo_preset();
        let yaml = serde_yaml::to_string(&config).expect("Failed to serialize to YAML");
        let deserialized: GeneratorConfig =
            serde_yaml::from_str(&yaml).expect("Failed to deserialize from YAML");

        assert_eq!(
            config.global.period_months,
            deserialized.global.period_months
        );
        assert_eq!(config.global.industry, deserialized.global.industry);
        assert_eq!(config.companies.len(), deserialized.companies.len());
        assert_eq!(config.companies[0].code, deserialized.companies[0].code);
    }

    #[test]
    fn test_config_json_roundtrip() {
        // Create a config without infinity values (JSON can't serialize f64::INFINITY)
        let mut config = demo_preset();
        // Replace infinity with a large but finite value for JSON compatibility
        config.master_data.employees.approval_limits.executive = 1e12;

        let json = serde_json::to_string(&config).expect("Failed to serialize to JSON");
        let deserialized: GeneratorConfig =
            serde_json::from_str(&json).expect("Failed to deserialize from JSON");

        assert_eq!(
            config.global.period_months,
            deserialized.global.period_months
        );
        assert_eq!(config.global.industry, deserialized.global.industry);
        assert_eq!(config.companies.len(), deserialized.companies.len());
    }

    #[test]
    fn test_transaction_volume_serialization() {
        // Test various transaction volumes serialize correctly
        let volumes = vec![
            (TransactionVolume::TenK, "ten_k"),
            (TransactionVolume::HundredK, "hundred_k"),
            (TransactionVolume::OneM, "one_m"),
            (TransactionVolume::TenM, "ten_m"),
            (TransactionVolume::HundredM, "hundred_m"),
        ];

        for (volume, expected_key) in volumes {
            let json = serde_json::to_string(&volume).expect("Failed to serialize");
            assert!(
                json.contains(expected_key),
                "Expected {} in JSON: {}",
                expected_key,
                json
            );
        }
    }

    #[test]
    fn test_transaction_volume_custom_serialization() {
        let volume = TransactionVolume::Custom(12345);
        let json = serde_json::to_string(&volume).expect("Failed to serialize");
        let deserialized: TransactionVolume =
            serde_json::from_str(&json).expect("Failed to deserialize");
        assert_eq!(deserialized.count(), 12345);
    }

    #[test]
    fn test_output_mode_serialization() {
        let modes = vec![
            OutputMode::Streaming,
            OutputMode::FlatFile,
            OutputMode::Both,
        ];

        for mode in modes {
            let json = serde_json::to_string(&mode).expect("Failed to serialize");
            let deserialized: OutputMode =
                serde_json::from_str(&json).expect("Failed to deserialize");
            assert!(format!("{:?}", mode) == format!("{:?}", deserialized));
        }
    }

    #[test]
    fn test_file_format_serialization() {
        let formats = vec![
            FileFormat::Csv,
            FileFormat::Parquet,
            FileFormat::Json,
            FileFormat::JsonLines,
        ];

        for format in formats {
            let json = serde_json::to_string(&format).expect("Failed to serialize");
            let deserialized: FileFormat =
                serde_json::from_str(&json).expect("Failed to deserialize");
            assert!(format!("{:?}", format) == format!("{:?}", deserialized));
        }
    }

    #[test]
    fn test_compression_algorithm_serialization() {
        let algos = vec![
            CompressionAlgorithm::Gzip,
            CompressionAlgorithm::Zstd,
            CompressionAlgorithm::Lz4,
            CompressionAlgorithm::Snappy,
        ];

        for algo in algos {
            let json = serde_json::to_string(&algo).expect("Failed to serialize");
            let deserialized: CompressionAlgorithm =
                serde_json::from_str(&json).expect("Failed to deserialize");
            assert!(format!("{:?}", algo) == format!("{:?}", deserialized));
        }
    }

    #[test]
    fn test_transfer_pricing_method_serialization() {
        let methods = vec![
            TransferPricingMethod::CostPlus,
            TransferPricingMethod::ComparableUncontrolled,
            TransferPricingMethod::ResalePrice,
            TransferPricingMethod::TransactionalNetMargin,
            TransferPricingMethod::ProfitSplit,
        ];

        for method in methods {
            let json = serde_json::to_string(&method).expect("Failed to serialize");
            let deserialized: TransferPricingMethod =
                serde_json::from_str(&json).expect("Failed to deserialize");
            assert!(format!("{:?}", method) == format!("{:?}", deserialized));
        }
    }

    #[test]
    fn test_benford_exemption_serialization() {
        let exemptions = vec![
            BenfordExemption::Recurring,
            BenfordExemption::Payroll,
            BenfordExemption::FixedFees,
            BenfordExemption::RoundAmounts,
        ];

        for exemption in exemptions {
            let json = serde_json::to_string(&exemption).expect("Failed to serialize");
            let deserialized: BenfordExemption =
                serde_json::from_str(&json).expect("Failed to deserialize");
            assert!(format!("{:?}", exemption) == format!("{:?}", deserialized));
        }
    }

    // ==========================================================================
    // Default Value Tests
    // ==========================================================================

    #[test]
    fn test_global_config_defaults() {
        let yaml = r#"
            industry: manufacturing
            start_date: "2024-01-01"
            period_months: 6
        "#;
        let config: GlobalConfig = serde_yaml::from_str(yaml).expect("Failed to parse");
        assert_eq!(config.group_currency, "USD");
        assert!(config.parallel);
        assert_eq!(config.worker_threads, 0);
        assert_eq!(config.memory_limit_mb, 0);
    }

    #[test]
    fn test_fraud_config_defaults() {
        let config = FraudConfig::default();
        assert!(!config.enabled);
        assert_eq!(config.fraud_rate, 0.005);
        assert!(!config.clustering_enabled);
    }

    #[test]
    fn test_internal_controls_config_defaults() {
        let config = InternalControlsConfig::default();
        assert!(!config.enabled);
        assert_eq!(config.exception_rate, 0.02);
        assert_eq!(config.sod_violation_rate, 0.01);
        assert!(config.export_control_master_data);
        assert_eq!(config.sox_materiality_threshold, 10000.0);
    }

    #[test]
    fn test_output_config_defaults() {
        let config = OutputConfig::default();
        assert!(matches!(config.mode, OutputMode::FlatFile));
        assert_eq!(config.formats, vec![FileFormat::Parquet]);
        assert!(config.compression.enabled);
        assert!(matches!(
            config.compression.algorithm,
            CompressionAlgorithm::Zstd
        ));
        assert!(config.include_acdoca);
        assert!(!config.include_bseg);
        assert!(config.partition_by_period);
        assert!(!config.partition_by_company);
    }

    #[test]
    fn test_approval_config_defaults() {
        let config = ApprovalConfig::default();
        assert!(!config.enabled);
        assert_eq!(config.auto_approve_threshold, 1000.0);
        assert_eq!(config.rejection_rate, 0.02);
        assert_eq!(config.revision_rate, 0.05);
        assert_eq!(config.average_approval_delay_hours, 4.0);
        assert_eq!(config.thresholds.len(), 4);
    }

    #[test]
    fn test_p2p_flow_config_defaults() {
        let config = P2PFlowConfig::default();
        assert!(config.enabled);
        assert_eq!(config.three_way_match_rate, 0.95);
        assert_eq!(config.partial_delivery_rate, 0.15);
        assert_eq!(config.average_po_to_gr_days, 14);
    }

    #[test]
    fn test_o2c_flow_config_defaults() {
        let config = O2CFlowConfig::default();
        assert!(config.enabled);
        assert_eq!(config.credit_check_failure_rate, 0.02);
        assert_eq!(config.return_rate, 0.03);
        assert_eq!(config.bad_debt_rate, 0.01);
    }

    #[test]
    fn test_balance_config_defaults() {
        let config = BalanceConfig::default();
        assert!(!config.generate_opening_balances);
        assert!(config.generate_trial_balances);
        assert_eq!(config.target_gross_margin, 0.35);
        assert!(config.validate_balance_equation);
        assert!(config.reconcile_subledgers);
    }

    // ==========================================================================
    // Partial Config Deserialization Tests
    // ==========================================================================

    #[test]
    fn test_partial_config_with_defaults() {
        // Minimal config that should use all defaults
        let yaml = r#"
            global:
              industry: manufacturing
              start_date: "2024-01-01"
              period_months: 3
            companies:
              - code: "TEST"
                name: "Test Company"
                currency: "USD"
                country: "US"
                annual_transaction_volume: ten_k
            chart_of_accounts:
              complexity: small
            output:
              output_directory: "./output"
        "#;

        let config: GeneratorConfig = serde_yaml::from_str(yaml).expect("Failed to parse");
        assert_eq!(config.global.period_months, 3);
        assert_eq!(config.companies.len(), 1);
        assert!(!config.fraud.enabled); // Default
        assert!(!config.internal_controls.enabled); // Default
    }

    #[test]
    fn test_config_with_fraud_enabled() {
        let yaml = r#"
            global:
              industry: retail
              start_date: "2024-01-01"
              period_months: 12
            companies:
              - code: "RETAIL"
                name: "Retail Co"
                currency: "USD"
                country: "US"
                annual_transaction_volume: hundred_k
            chart_of_accounts:
              complexity: medium
            output:
              output_directory: "./output"
            fraud:
              enabled: true
              fraud_rate: 0.05
              clustering_enabled: true
        "#;

        let config: GeneratorConfig = serde_yaml::from_str(yaml).expect("Failed to parse");
        assert!(config.fraud.enabled);
        assert_eq!(config.fraud.fraud_rate, 0.05);
        assert!(config.fraud.clustering_enabled);
    }

    #[test]
    fn test_config_with_multiple_companies() {
        let yaml = r#"
            global:
              industry: manufacturing
              start_date: "2024-01-01"
              period_months: 6
            companies:
              - code: "HQ"
                name: "Headquarters"
                currency: "USD"
                country: "US"
                annual_transaction_volume: hundred_k
                volume_weight: 1.0
              - code: "EU"
                name: "European Subsidiary"
                currency: "EUR"
                country: "DE"
                annual_transaction_volume: hundred_k
                volume_weight: 0.5
              - code: "APAC"
                name: "Asia Pacific"
                currency: "JPY"
                country: "JP"
                annual_transaction_volume: ten_k
                volume_weight: 0.3
            chart_of_accounts:
              complexity: large
            output:
              output_directory: "./output"
        "#;

        let config: GeneratorConfig = serde_yaml::from_str(yaml).expect("Failed to parse");
        assert_eq!(config.companies.len(), 3);
        assert_eq!(config.companies[0].code, "HQ");
        assert_eq!(config.companies[1].currency, "EUR");
        assert_eq!(config.companies[2].volume_weight, 0.3);
    }

    #[test]
    fn test_intercompany_config() {
        let yaml = r#"
            enabled: true
            ic_transaction_rate: 0.20
            transfer_pricing_method: cost_plus
            markup_percent: 0.08
            generate_matched_pairs: true
            generate_eliminations: true
        "#;

        let config: IntercompanyConfig = serde_yaml::from_str(yaml).expect("Failed to parse");
        assert!(config.enabled);
        assert_eq!(config.ic_transaction_rate, 0.20);
        assert!(matches!(
            config.transfer_pricing_method,
            TransferPricingMethod::CostPlus
        ));
        assert_eq!(config.markup_percent, 0.08);
        assert!(config.generate_eliminations);
    }

    // ==========================================================================
    // Company Config Tests
    // ==========================================================================

    #[test]
    fn test_company_config_defaults() {
        let yaml = r#"
            code: "TEST"
            name: "Test Company"
            currency: "USD"
            country: "US"
            annual_transaction_volume: ten_k
        "#;

        let config: CompanyConfig = serde_yaml::from_str(yaml).expect("Failed to parse");
        assert_eq!(config.fiscal_year_variant, "K4"); // Default
        assert_eq!(config.volume_weight, 1.0); // Default
    }

    // ==========================================================================
    // Chart of Accounts Config Tests
    // ==========================================================================

    #[test]
    fn test_coa_config_defaults() {
        let yaml = r#"
            complexity: medium
        "#;

        let config: ChartOfAccountsConfig = serde_yaml::from_str(yaml).expect("Failed to parse");
        assert!(config.industry_specific); // Default true
        assert!(config.custom_accounts.is_none());
        assert_eq!(config.min_hierarchy_depth, 2); // Default
        assert_eq!(config.max_hierarchy_depth, 5); // Default
    }
}
