//! Configuration schema for synthetic data generation.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use synth_core::distributions::{
    AmountDistributionConfig, DebitCreditDistributionConfig, EvenOddDistributionConfig,
    LineItemDistributionConfig, SeasonalityConfig,
};
use synth_core::models::{CoAComplexity, IndustrySector};

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
    pub transactions: TransactionConfig,
    /// Output configuration
    pub output: OutputConfig,
    /// Fraud simulation settings
    #[serde(default)]
    pub fraud: FraudConfig,
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
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
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
