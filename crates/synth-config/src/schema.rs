//! Configuration schema for synthetic data generation.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use synth_core::models::{CoAComplexity, IndustrySector};
use synth_core::distributions::{
    LineItemDistributionConfig, EvenOddDistributionConfig,
    DebitCreditDistributionConfig, AmountDistributionConfig,
    SeasonalityConfig,
};

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
    /// Business process mix
    #[serde(default)]
    pub business_processes: BusinessProcessConfig,
    /// User persona distribution
    #[serde(default)]
    pub user_personas: UserPersonaConfig,
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

fn default_currency() -> String { "USD".to_string() }
fn default_true() -> bool { true }

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

fn default_fiscal_variant() -> String { "K4".to_string() }
fn default_weight() -> f64 { 1.0 }

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

fn default_min_depth() -> u8 { 2 }
fn default_max_depth() -> u8 { 5 }

/// Transaction generation configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
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
}

impl Default for TransactionConfig {
    fn default() -> Self {
        Self {
            line_item_distribution: LineItemDistributionConfig::default(),
            debit_credit_distribution: DebitCreditDistributionConfig::default(),
            even_odd_distribution: EvenOddDistributionConfig::default(),
            source_distribution: SourceDistribution::default(),
            seasonality: SeasonalityConfig::default(),
            amounts: AmountDistributionConfig::default(),
        }
    }
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

fn default_formats() -> Vec<FileFormat> { vec![FileFormat::Parquet] }
fn default_batch_size() -> usize { 100_000 }

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

fn default_compression_level() -> u8 { 3 }

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
}

fn default_fraud_rate() -> f64 { 0.005 }
fn default_clustering_factor() -> f64 { 3.0 }

impl Default for FraudConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            fraud_rate: default_fraud_rate(),
            fraud_type_distribution: FraudTypeDistribution::default(),
            clustering_enabled: false,
            clustering_factor: default_clustering_factor(),
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

fn default_o2c() -> f64 { 0.35 }
fn default_p2p() -> f64 { 0.30 }
fn default_r2r() -> f64 { 0.20 }
fn default_h2r() -> f64 { 0.10 }
fn default_a2r() -> f64 { 0.05 }

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
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserPersonaConfig {
    /// Distribution of user personas
    #[serde(default)]
    pub persona_distribution: PersonaDistribution,
    /// Users per persona type
    #[serde(default)]
    pub users_per_persona: UsersPerPersona,
}

impl Default for UserPersonaConfig {
    fn default() -> Self {
        Self {
            persona_distribution: PersonaDistribution::default(),
            users_per_persona: UsersPerPersona::default(),
        }
    }
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
