//! Anomaly types and labels for synthetic data generation.
//!
//! This module provides comprehensive anomaly classification for:
//! - Fraud detection training
//! - Error detection systems
//! - Process compliance monitoring
//! - Statistical anomaly detection
//! - Graph-based anomaly detection

use chrono::{NaiveDate, NaiveDateTime};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Causal reason explaining why an anomaly was injected.
///
/// This enables provenance tracking for understanding the "why" behind each anomaly.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum AnomalyCausalReason {
    /// Injected due to random rate selection.
    RandomRate {
        /// Base rate used for selection.
        base_rate: f64,
    },
    /// Injected due to temporal pattern matching.
    TemporalPattern {
        /// Name of the temporal pattern (e.g., "year_end_spike", "month_end").
        pattern_name: String,
    },
    /// Injected based on entity targeting rules.
    EntityTargeting {
        /// Type of entity targeted (e.g., "vendor", "user", "account").
        target_type: String,
        /// ID of the targeted entity.
        target_id: String,
    },
    /// Part of an anomaly cluster.
    ClusterMembership {
        /// ID of the cluster this anomaly belongs to.
        cluster_id: String,
    },
    /// Part of a multi-step scenario.
    ScenarioStep {
        /// Type of scenario (e.g., "kickback_scheme", "round_tripping").
        scenario_type: String,
        /// Step number within the scenario.
        step_number: u32,
    },
    /// Injected based on data quality profile.
    DataQualityProfile {
        /// Profile name (e.g., "noisy", "legacy", "clean").
        profile: String,
    },
    /// Injected for ML training balance.
    MLTrainingBalance {
        /// Target class being balanced.
        target_class: String,
    },
}

/// Structured injection strategy with captured parameters.
///
/// Unlike the string-based `injection_strategy` field, this enum captures
/// the exact parameters used during injection for full reproducibility.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum InjectionStrategy {
    /// Amount was manipulated by a factor.
    AmountManipulation {
        /// Original amount before manipulation.
        original: Decimal,
        /// Multiplication factor applied.
        factor: f64,
    },
    /// Amount adjusted to avoid a threshold.
    ThresholdAvoidance {
        /// Threshold being avoided.
        threshold: Decimal,
        /// Final amount after adjustment.
        adjusted_amount: Decimal,
    },
    /// Date was backdated or forward-dated.
    DateShift {
        /// Number of days shifted (negative = backdated).
        days_shifted: i32,
        /// Original date before shift.
        original_date: NaiveDate,
    },
    /// User approved their own transaction.
    SelfApproval {
        /// User who created and approved.
        user_id: String,
    },
    /// Segregation of duties violation.
    SoDViolation {
        /// First duty involved.
        duty1: String,
        /// Second duty involved.
        duty2: String,
        /// User who performed both duties.
        violating_user: String,
    },
    /// Exact duplicate of another document.
    ExactDuplicate {
        /// ID of the original document.
        original_doc_id: String,
    },
    /// Near-duplicate with small variations.
    NearDuplicate {
        /// ID of the original document.
        original_doc_id: String,
        /// Fields that were varied.
        varied_fields: Vec<String>,
    },
    /// Circular flow of funds/goods.
    CircularFlow {
        /// Chain of entities involved.
        entity_chain: Vec<String>,
    },
    /// Split transaction to avoid threshold.
    SplitTransaction {
        /// Original total amount.
        original_amount: Decimal,
        /// Number of splits.
        split_count: u32,
        /// IDs of the split documents.
        split_doc_ids: Vec<String>,
    },
    /// Round number manipulation.
    RoundNumbering {
        /// Original precise amount.
        original_amount: Decimal,
        /// Rounded amount.
        rounded_amount: Decimal,
    },
    /// Timing manipulation (weekend, after-hours, etc.).
    TimingManipulation {
        /// Type of timing issue.
        timing_type: String,
        /// Original timestamp.
        original_time: Option<NaiveDateTime>,
    },
    /// Account misclassification.
    AccountMisclassification {
        /// Correct account.
        correct_account: String,
        /// Incorrect account used.
        incorrect_account: String,
    },
    /// Missing required field.
    MissingField {
        /// Name of the missing field.
        field_name: String,
    },
    /// Custom injection strategy.
    Custom {
        /// Strategy name.
        name: String,
        /// Additional parameters.
        parameters: HashMap<String, String>,
    },
}

impl InjectionStrategy {
    /// Returns a human-readable description of the strategy.
    pub fn description(&self) -> String {
        match self {
            InjectionStrategy::AmountManipulation { factor, .. } => {
                format!("Amount multiplied by {:.2}", factor)
            }
            InjectionStrategy::ThresholdAvoidance { threshold, .. } => {
                format!("Amount adjusted to avoid {} threshold", threshold)
            }
            InjectionStrategy::DateShift { days_shifted, .. } => {
                if *days_shifted < 0 {
                    format!("Date backdated by {} days", days_shifted.abs())
                } else {
                    format!("Date forward-dated by {} days", days_shifted)
                }
            }
            InjectionStrategy::SelfApproval { user_id } => {
                format!("Self-approval by user {}", user_id)
            }
            InjectionStrategy::SoDViolation { duty1, duty2, .. } => {
                format!("SoD violation: {} and {}", duty1, duty2)
            }
            InjectionStrategy::ExactDuplicate { original_doc_id } => {
                format!("Exact duplicate of {}", original_doc_id)
            }
            InjectionStrategy::NearDuplicate {
                original_doc_id,
                varied_fields,
            } => {
                format!(
                    "Near-duplicate of {} (varied: {:?})",
                    original_doc_id, varied_fields
                )
            }
            InjectionStrategy::CircularFlow { entity_chain } => {
                format!("Circular flow through {} entities", entity_chain.len())
            }
            InjectionStrategy::SplitTransaction { split_count, .. } => {
                format!("Split into {} transactions", split_count)
            }
            InjectionStrategy::RoundNumbering { .. } => "Amount rounded to even number".to_string(),
            InjectionStrategy::TimingManipulation { timing_type, .. } => {
                format!("Timing manipulation: {}", timing_type)
            }
            InjectionStrategy::AccountMisclassification {
                correct_account,
                incorrect_account,
            } => {
                format!(
                    "Misclassified from {} to {}",
                    correct_account, incorrect_account
                )
            }
            InjectionStrategy::MissingField { field_name } => {
                format!("Missing required field: {}", field_name)
            }
            InjectionStrategy::Custom { name, .. } => format!("Custom: {}", name),
        }
    }

    /// Returns the strategy type name.
    pub fn strategy_type(&self) -> &'static str {
        match self {
            InjectionStrategy::AmountManipulation { .. } => "AmountManipulation",
            InjectionStrategy::ThresholdAvoidance { .. } => "ThresholdAvoidance",
            InjectionStrategy::DateShift { .. } => "DateShift",
            InjectionStrategy::SelfApproval { .. } => "SelfApproval",
            InjectionStrategy::SoDViolation { .. } => "SoDViolation",
            InjectionStrategy::ExactDuplicate { .. } => "ExactDuplicate",
            InjectionStrategy::NearDuplicate { .. } => "NearDuplicate",
            InjectionStrategy::CircularFlow { .. } => "CircularFlow",
            InjectionStrategy::SplitTransaction { .. } => "SplitTransaction",
            InjectionStrategy::RoundNumbering { .. } => "RoundNumbering",
            InjectionStrategy::TimingManipulation { .. } => "TimingManipulation",
            InjectionStrategy::AccountMisclassification { .. } => "AccountMisclassification",
            InjectionStrategy::MissingField { .. } => "MissingField",
            InjectionStrategy::Custom { .. } => "Custom",
        }
    }
}

/// Primary anomaly classification.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AnomalyType {
    /// Fraudulent activity.
    Fraud(FraudType),
    /// Data entry or processing error.
    Error(ErrorType),
    /// Process or control issue.
    ProcessIssue(ProcessIssueType),
    /// Statistical anomaly.
    Statistical(StatisticalAnomalyType),
    /// Relational/graph anomaly.
    Relational(RelationalAnomalyType),
    /// Custom anomaly type.
    Custom(String),
}

impl AnomalyType {
    /// Returns the category name.
    pub fn category(&self) -> &'static str {
        match self {
            AnomalyType::Fraud(_) => "Fraud",
            AnomalyType::Error(_) => "Error",
            AnomalyType::ProcessIssue(_) => "ProcessIssue",
            AnomalyType::Statistical(_) => "Statistical",
            AnomalyType::Relational(_) => "Relational",
            AnomalyType::Custom(_) => "Custom",
        }
    }

    /// Returns the specific type name.
    pub fn type_name(&self) -> String {
        match self {
            AnomalyType::Fraud(t) => format!("{:?}", t),
            AnomalyType::Error(t) => format!("{:?}", t),
            AnomalyType::ProcessIssue(t) => format!("{:?}", t),
            AnomalyType::Statistical(t) => format!("{:?}", t),
            AnomalyType::Relational(t) => format!("{:?}", t),
            AnomalyType::Custom(s) => s.clone(),
        }
    }

    /// Returns the severity level (1-5, 5 being most severe).
    pub fn severity(&self) -> u8 {
        match self {
            AnomalyType::Fraud(t) => t.severity(),
            AnomalyType::Error(t) => t.severity(),
            AnomalyType::ProcessIssue(t) => t.severity(),
            AnomalyType::Statistical(t) => t.severity(),
            AnomalyType::Relational(t) => t.severity(),
            AnomalyType::Custom(_) => 3,
        }
    }

    /// Returns whether this anomaly is typically intentional.
    pub fn is_intentional(&self) -> bool {
        matches!(self, AnomalyType::Fraud(_))
    }
}

/// Fraud types for detection training.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum FraudType {
    // Journal Entry Fraud
    /// Fictitious journal entry with no business purpose.
    FictitiousEntry,
    /// Fictitious transaction (alias for FictitiousEntry).
    FictitiousTransaction,
    /// Round-dollar amounts suggesting manual manipulation.
    RoundDollarManipulation,
    /// Entry posted just below approval threshold.
    JustBelowThreshold,
    /// Revenue recognition manipulation.
    RevenueManipulation,
    /// Expense capitalization fraud.
    ImproperCapitalization,
    /// Improperly capitalizing expenses as assets.
    ExpenseCapitalization,
    /// Cookie jar reserves manipulation.
    ReserveManipulation,
    /// Round-tripping funds through suspense/clearing accounts.
    SuspenseAccountAbuse,
    /// Splitting transactions to stay below approval thresholds.
    SplitTransaction,
    /// Unusual timing (weekend, holiday, after-hours postings).
    TimingAnomaly,
    /// Posting to unauthorized accounts.
    UnauthorizedAccess,

    // Approval Fraud
    /// User approving their own request.
    SelfApproval,
    /// Approval beyond authorized limit.
    ExceededApprovalLimit,
    /// Segregation of duties violation.
    SegregationOfDutiesViolation,
    /// Approval by unauthorized user.
    UnauthorizedApproval,
    /// Collusion between approver and requester.
    CollusiveApproval,

    // Vendor/Payment Fraud
    /// Fictitious vendor.
    FictitiousVendor,
    /// Duplicate payment to vendor.
    DuplicatePayment,
    /// Payment to shell company.
    ShellCompanyPayment,
    /// Kickback scheme.
    Kickback,
    /// Kickback scheme (alias).
    KickbackScheme,
    /// Invoice manipulation.
    InvoiceManipulation,

    // Asset Fraud
    /// Misappropriation of assets.
    AssetMisappropriation,
    /// Inventory theft.
    InventoryTheft,
    /// Ghost employee.
    GhostEmployee,

    // Financial Statement Fraud
    /// Premature revenue recognition.
    PrematureRevenue,
    /// Understated liabilities.
    UnderstatedLiabilities,
    /// Overstated assets.
    OverstatedAssets,
    /// Channel stuffing.
    ChannelStuffing,
}

impl FraudType {
    /// Returns severity level (1-5).
    pub fn severity(&self) -> u8 {
        match self {
            FraudType::RoundDollarManipulation => 2,
            FraudType::JustBelowThreshold => 3,
            FraudType::SelfApproval => 3,
            FraudType::ExceededApprovalLimit => 3,
            FraudType::DuplicatePayment => 3,
            FraudType::FictitiousEntry => 4,
            FraudType::RevenueManipulation => 5,
            FraudType::FictitiousVendor => 5,
            FraudType::ShellCompanyPayment => 5,
            FraudType::AssetMisappropriation => 5,
            FraudType::SegregationOfDutiesViolation => 4,
            FraudType::CollusiveApproval => 5,
            _ => 4,
        }
    }
}

/// Error types for error detection.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ErrorType {
    // Data Entry Errors
    /// Duplicate document entry.
    DuplicateEntry,
    /// Reversed debit/credit amounts.
    ReversedAmount,
    /// Transposed digits in amount.
    TransposedDigits,
    /// Wrong decimal placement.
    DecimalError,
    /// Missing required field.
    MissingField,
    /// Invalid account code.
    InvalidAccount,

    // Timing Errors
    /// Posted to wrong period.
    WrongPeriod,
    /// Backdated entry.
    BackdatedEntry,
    /// Future-dated entry.
    FutureDatedEntry,
    /// Cutoff error.
    CutoffError,

    // Classification Errors
    /// Wrong account classification.
    MisclassifiedAccount,
    /// Wrong cost center.
    WrongCostCenter,
    /// Wrong company code.
    WrongCompanyCode,

    // Calculation Errors
    /// Unbalanced journal entry.
    UnbalancedEntry,
    /// Rounding error.
    RoundingError,
    /// Currency conversion error.
    CurrencyError,
    /// Tax calculation error.
    TaxCalculationError,
}

impl ErrorType {
    /// Returns severity level (1-5).
    pub fn severity(&self) -> u8 {
        match self {
            ErrorType::RoundingError => 1,
            ErrorType::MissingField => 2,
            ErrorType::TransposedDigits => 2,
            ErrorType::DecimalError => 3,
            ErrorType::DuplicateEntry => 3,
            ErrorType::ReversedAmount => 3,
            ErrorType::WrongPeriod => 4,
            ErrorType::UnbalancedEntry => 5,
            ErrorType::CurrencyError => 4,
            _ => 3,
        }
    }
}

/// Process issue types.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ProcessIssueType {
    // Approval Issues
    /// Approval skipped entirely.
    SkippedApproval,
    /// Late approval (after posting).
    LateApproval,
    /// Missing supporting documentation.
    MissingDocumentation,
    /// Incomplete approval chain.
    IncompleteApprovalChain,

    // Timing Issues
    /// Late posting.
    LatePosting,
    /// Posting outside business hours.
    AfterHoursPosting,
    /// Weekend/holiday posting.
    WeekendPosting,
    /// Rushed period-end posting.
    RushedPeriodEnd,

    // Control Issues
    /// Manual override of system control.
    ManualOverride,
    /// Unusual user access pattern.
    UnusualAccess,
    /// System bypass.
    SystemBypass,
    /// Batch processing anomaly.
    BatchAnomaly,

    // Documentation Issues
    /// Vague or missing description.
    VagueDescription,
    /// Changed after posting.
    PostFactoChange,
    /// Incomplete audit trail.
    IncompleteAuditTrail,
}

impl ProcessIssueType {
    /// Returns severity level (1-5).
    pub fn severity(&self) -> u8 {
        match self {
            ProcessIssueType::VagueDescription => 1,
            ProcessIssueType::LatePosting => 2,
            ProcessIssueType::AfterHoursPosting => 2,
            ProcessIssueType::WeekendPosting => 2,
            ProcessIssueType::SkippedApproval => 4,
            ProcessIssueType::ManualOverride => 4,
            ProcessIssueType::SystemBypass => 5,
            ProcessIssueType::IncompleteAuditTrail => 4,
            _ => 3,
        }
    }
}

/// Statistical anomaly types.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum StatisticalAnomalyType {
    // Amount Anomalies
    /// Amount significantly above normal.
    UnusuallyHighAmount,
    /// Amount significantly below normal.
    UnusuallyLowAmount,
    /// Violates Benford's Law distribution.
    BenfordViolation,
    /// Exact duplicate amount (suspicious).
    ExactDuplicateAmount,
    /// Repeating pattern in amounts.
    RepeatingAmount,

    // Frequency Anomalies
    /// Unusual transaction frequency.
    UnusualFrequency,
    /// Burst of transactions.
    TransactionBurst,
    /// Unusual time of day.
    UnusualTiming,

    // Trend Anomalies
    /// Break in historical trend.
    TrendBreak,
    /// Sudden level shift.
    LevelShift,
    /// Seasonal pattern violation.
    SeasonalAnomaly,

    // Distribution Anomalies
    /// Outlier in distribution.
    StatisticalOutlier,
    /// Change in variance.
    VarianceChange,
    /// Distribution shift.
    DistributionShift,
}

impl StatisticalAnomalyType {
    /// Returns severity level (1-5).
    pub fn severity(&self) -> u8 {
        match self {
            StatisticalAnomalyType::UnusualTiming => 1,
            StatisticalAnomalyType::UnusualFrequency => 2,
            StatisticalAnomalyType::BenfordViolation => 2,
            StatisticalAnomalyType::UnusuallyHighAmount => 3,
            StatisticalAnomalyType::TrendBreak => 3,
            StatisticalAnomalyType::TransactionBurst => 4,
            StatisticalAnomalyType::ExactDuplicateAmount => 3,
            _ => 3,
        }
    }
}

/// Relational/graph anomaly types.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RelationalAnomalyType {
    // Transaction Pattern Anomalies
    /// Circular transaction pattern.
    CircularTransaction,
    /// Unusual account combination.
    UnusualAccountPair,
    /// New trading partner.
    NewCounterparty,
    /// Dormant account suddenly active.
    DormantAccountActivity,

    // Network Anomalies
    /// Unusual network centrality.
    CentralityAnomaly,
    /// Isolated transaction cluster.
    IsolatedCluster,
    /// Bridge node anomaly.
    BridgeNodeAnomaly,
    /// Community structure change.
    CommunityAnomaly,

    // Relationship Anomalies
    /// Missing expected relationship.
    MissingRelationship,
    /// Unexpected relationship.
    UnexpectedRelationship,
    /// Relationship strength change.
    RelationshipStrengthChange,

    // Intercompany Anomalies
    /// Unmatched intercompany transaction.
    UnmatchedIntercompany,
    /// Circular intercompany flow.
    CircularIntercompany,
    /// Transfer pricing anomaly.
    TransferPricingAnomaly,
}

impl RelationalAnomalyType {
    /// Returns severity level (1-5).
    pub fn severity(&self) -> u8 {
        match self {
            RelationalAnomalyType::NewCounterparty => 1,
            RelationalAnomalyType::DormantAccountActivity => 2,
            RelationalAnomalyType::UnusualAccountPair => 2,
            RelationalAnomalyType::CircularTransaction => 4,
            RelationalAnomalyType::CircularIntercompany => 4,
            RelationalAnomalyType::TransferPricingAnomaly => 4,
            RelationalAnomalyType::UnmatchedIntercompany => 3,
            _ => 3,
        }
    }
}

/// A labeled anomaly for supervised learning.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LabeledAnomaly {
    /// Unique anomaly identifier.
    pub anomaly_id: String,
    /// Type of anomaly.
    pub anomaly_type: AnomalyType,
    /// Document or entity that contains the anomaly.
    pub document_id: String,
    /// Document type (JE, PO, Invoice, etc.).
    pub document_type: String,
    /// Company code.
    pub company_code: String,
    /// Date the anomaly occurred.
    pub anomaly_date: NaiveDate,
    /// Timestamp when detected/injected.
    pub detection_timestamp: NaiveDateTime,
    /// Confidence score (0.0 - 1.0) for injected anomalies.
    pub confidence: f64,
    /// Severity (1-5).
    pub severity: u8,
    /// Description of the anomaly.
    pub description: String,
    /// Related entities (user IDs, account codes, etc.).
    pub related_entities: Vec<String>,
    /// Monetary impact if applicable.
    pub monetary_impact: Option<Decimal>,
    /// Additional metadata.
    pub metadata: HashMap<String, String>,
    /// Whether this was injected (true) or naturally occurring (false).
    pub is_injected: bool,
    /// Injection strategy used (if injected) - legacy string field.
    pub injection_strategy: Option<String>,
    /// Cluster ID if part of an anomaly cluster.
    pub cluster_id: Option<String>,

    // ========================================
    // PROVENANCE TRACKING FIELDS (Phase 1.2)
    // ========================================
    /// Hash of the original document before modification.
    /// Enables tracking what the document looked like pre-injection.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub original_document_hash: Option<String>,

    /// Causal reason explaining why this anomaly was injected.
    /// Provides "why" tracking for each anomaly.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub causal_reason: Option<AnomalyCausalReason>,

    /// Structured injection strategy with parameters.
    /// More detailed than the legacy string-based injection_strategy field.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub structured_strategy: Option<InjectionStrategy>,

    /// Parent anomaly ID if this was derived from another anomaly.
    /// Enables anomaly transformation chains.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parent_anomaly_id: Option<String>,

    /// Child anomaly IDs that were derived from this anomaly.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub child_anomaly_ids: Vec<String>,

    /// Scenario ID if this anomaly is part of a multi-step scenario.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub scenario_id: Option<String>,

    /// Generation run ID that produced this anomaly.
    /// Enables tracing anomalies back to their generation run.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub run_id: Option<String>,

    /// Seed used for RNG during generation.
    /// Enables reproducibility.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub generation_seed: Option<u64>,
}

impl LabeledAnomaly {
    /// Creates a new labeled anomaly.
    pub fn new(
        anomaly_id: String,
        anomaly_type: AnomalyType,
        document_id: String,
        document_type: String,
        company_code: String,
        anomaly_date: NaiveDate,
    ) -> Self {
        let severity = anomaly_type.severity();
        let description = format!(
            "{} - {} in document {}",
            anomaly_type.category(),
            anomaly_type.type_name(),
            document_id
        );

        Self {
            anomaly_id,
            anomaly_type,
            document_id,
            document_type,
            company_code,
            anomaly_date,
            detection_timestamp: chrono::Local::now().naive_local(),
            confidence: 1.0,
            severity,
            description,
            related_entities: Vec::new(),
            monetary_impact: None,
            metadata: HashMap::new(),
            is_injected: true,
            injection_strategy: None,
            cluster_id: None,
            // Provenance fields
            original_document_hash: None,
            causal_reason: None,
            structured_strategy: None,
            parent_anomaly_id: None,
            child_anomaly_ids: Vec::new(),
            scenario_id: None,
            run_id: None,
            generation_seed: None,
        }
    }

    /// Sets the description.
    pub fn with_description(mut self, description: &str) -> Self {
        self.description = description.to_string();
        self
    }

    /// Sets the monetary impact.
    pub fn with_monetary_impact(mut self, impact: Decimal) -> Self {
        self.monetary_impact = Some(impact);
        self
    }

    /// Adds a related entity.
    pub fn with_related_entity(mut self, entity: &str) -> Self {
        self.related_entities.push(entity.to_string());
        self
    }

    /// Adds metadata.
    pub fn with_metadata(mut self, key: &str, value: &str) -> Self {
        self.metadata.insert(key.to_string(), value.to_string());
        self
    }

    /// Sets the injection strategy (legacy string).
    pub fn with_injection_strategy(mut self, strategy: &str) -> Self {
        self.injection_strategy = Some(strategy.to_string());
        self
    }

    /// Sets the cluster ID.
    pub fn with_cluster(mut self, cluster_id: &str) -> Self {
        self.cluster_id = Some(cluster_id.to_string());
        self
    }

    // ========================================
    // PROVENANCE BUILDER METHODS (Phase 1.2)
    // ========================================

    /// Sets the original document hash for provenance tracking.
    pub fn with_original_document_hash(mut self, hash: &str) -> Self {
        self.original_document_hash = Some(hash.to_string());
        self
    }

    /// Sets the causal reason for this anomaly.
    pub fn with_causal_reason(mut self, reason: AnomalyCausalReason) -> Self {
        self.causal_reason = Some(reason);
        self
    }

    /// Sets the structured injection strategy.
    pub fn with_structured_strategy(mut self, strategy: InjectionStrategy) -> Self {
        // Also set the legacy string field for backward compatibility
        self.injection_strategy = Some(strategy.strategy_type().to_string());
        self.structured_strategy = Some(strategy);
        self
    }

    /// Sets the parent anomaly ID (for anomaly derivation chains).
    pub fn with_parent_anomaly(mut self, parent_id: &str) -> Self {
        self.parent_anomaly_id = Some(parent_id.to_string());
        self
    }

    /// Adds a child anomaly ID.
    pub fn with_child_anomaly(mut self, child_id: &str) -> Self {
        self.child_anomaly_ids.push(child_id.to_string());
        self
    }

    /// Sets the scenario ID for multi-step scenario tracking.
    pub fn with_scenario(mut self, scenario_id: &str) -> Self {
        self.scenario_id = Some(scenario_id.to_string());
        self
    }

    /// Sets the generation run ID.
    pub fn with_run_id(mut self, run_id: &str) -> Self {
        self.run_id = Some(run_id.to_string());
        self
    }

    /// Sets the generation seed for reproducibility.
    pub fn with_generation_seed(mut self, seed: u64) -> Self {
        self.generation_seed = Some(seed);
        self
    }

    /// Sets multiple provenance fields at once for convenience.
    pub fn with_provenance(
        mut self,
        run_id: Option<&str>,
        seed: Option<u64>,
        causal_reason: Option<AnomalyCausalReason>,
    ) -> Self {
        if let Some(id) = run_id {
            self.run_id = Some(id.to_string());
        }
        self.generation_seed = seed;
        self.causal_reason = causal_reason;
        self
    }

    /// Converts to a feature vector for ML.
    ///
    /// Returns a vector of 15 features:
    /// - 6 features: Category one-hot encoding (Fraud, Error, ProcessIssue, Statistical, Relational, Custom)
    /// - 1 feature: Severity (normalized 0-1)
    /// - 1 feature: Confidence
    /// - 1 feature: Has monetary impact (0/1)
    /// - 1 feature: Monetary impact (log-scaled)
    /// - 1 feature: Is intentional (0/1)
    /// - 1 feature: Number of related entities
    /// - 1 feature: Is part of cluster (0/1)
    /// - 1 feature: Is part of scenario (0/1)
    /// - 1 feature: Has parent anomaly (0/1) - indicates derivation
    pub fn to_features(&self) -> Vec<f64> {
        let mut features = Vec::new();

        // Category one-hot encoding
        let categories = [
            "Fraud",
            "Error",
            "ProcessIssue",
            "Statistical",
            "Relational",
            "Custom",
        ];
        for cat in &categories {
            features.push(if self.anomaly_type.category() == *cat {
                1.0
            } else {
                0.0
            });
        }

        // Severity (normalized)
        features.push(self.severity as f64 / 5.0);

        // Confidence
        features.push(self.confidence);

        // Has monetary impact
        features.push(if self.monetary_impact.is_some() {
            1.0
        } else {
            0.0
        });

        // Monetary impact (log-scaled)
        if let Some(impact) = self.monetary_impact {
            let impact_f64: f64 = impact.try_into().unwrap_or(0.0);
            features.push((impact_f64.abs() + 1.0).ln());
        } else {
            features.push(0.0);
        }

        // Is intentional
        features.push(if self.anomaly_type.is_intentional() {
            1.0
        } else {
            0.0
        });

        // Number of related entities
        features.push(self.related_entities.len() as f64);

        // Is part of cluster
        features.push(if self.cluster_id.is_some() { 1.0 } else { 0.0 });

        // Provenance features
        // Is part of scenario
        features.push(if self.scenario_id.is_some() { 1.0 } else { 0.0 });

        // Has parent anomaly (indicates this is a derived anomaly)
        features.push(if self.parent_anomaly_id.is_some() {
            1.0
        } else {
            0.0
        });

        features
    }

    /// Returns the number of features in the feature vector.
    pub fn feature_count() -> usize {
        15 // 6 category + 9 other features
    }

    /// Returns feature names for documentation/ML metadata.
    pub fn feature_names() -> Vec<&'static str> {
        vec![
            "category_fraud",
            "category_error",
            "category_process_issue",
            "category_statistical",
            "category_relational",
            "category_custom",
            "severity_normalized",
            "confidence",
            "has_monetary_impact",
            "monetary_impact_log",
            "is_intentional",
            "related_entity_count",
            "is_clustered",
            "is_scenario_part",
            "is_derived",
        ]
    }
}

/// Summary of anomalies for reporting.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AnomalySummary {
    /// Total anomaly count.
    pub total_count: usize,
    /// Count by category.
    pub by_category: HashMap<String, usize>,
    /// Count by specific type.
    pub by_type: HashMap<String, usize>,
    /// Count by severity.
    pub by_severity: HashMap<u8, usize>,
    /// Count by company.
    pub by_company: HashMap<String, usize>,
    /// Total monetary impact.
    pub total_monetary_impact: Decimal,
    /// Date range.
    pub date_range: Option<(NaiveDate, NaiveDate)>,
    /// Number of clusters.
    pub cluster_count: usize,
}

impl AnomalySummary {
    /// Creates a summary from a list of anomalies.
    pub fn from_anomalies(anomalies: &[LabeledAnomaly]) -> Self {
        let mut summary = AnomalySummary {
            total_count: anomalies.len(),
            ..Default::default()
        };

        let mut min_date: Option<NaiveDate> = None;
        let mut max_date: Option<NaiveDate> = None;
        let mut clusters = std::collections::HashSet::new();

        for anomaly in anomalies {
            // By category
            *summary
                .by_category
                .entry(anomaly.anomaly_type.category().to_string())
                .or_insert(0) += 1;

            // By type
            *summary
                .by_type
                .entry(anomaly.anomaly_type.type_name())
                .or_insert(0) += 1;

            // By severity
            *summary.by_severity.entry(anomaly.severity).or_insert(0) += 1;

            // By company
            *summary
                .by_company
                .entry(anomaly.company_code.clone())
                .or_insert(0) += 1;

            // Monetary impact
            if let Some(impact) = anomaly.monetary_impact {
                summary.total_monetary_impact += impact;
            }

            // Date range
            match min_date {
                None => min_date = Some(anomaly.anomaly_date),
                Some(d) if anomaly.anomaly_date < d => min_date = Some(anomaly.anomaly_date),
                _ => {}
            }
            match max_date {
                None => max_date = Some(anomaly.anomaly_date),
                Some(d) if anomaly.anomaly_date > d => max_date = Some(anomaly.anomaly_date),
                _ => {}
            }

            // Clusters
            if let Some(cluster_id) = &anomaly.cluster_id {
                clusters.insert(cluster_id.clone());
            }
        }

        summary.date_range = min_date.zip(max_date);
        summary.cluster_count = clusters.len();

        summary
    }
}

// ============================================================================
// ENHANCED ANOMALY TAXONOMY (FR-003)
// ============================================================================

/// High-level anomaly category for multi-class classification.
///
/// These categories provide a more granular classification than the base
/// AnomalyType enum, enabling better ML model training and audit reporting.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AnomalyCategory {
    // Vendor-related anomalies
    /// Fictitious or shell vendor.
    FictitiousVendor,
    /// Kickback or collusion with vendor.
    VendorKickback,
    /// Related party vendor transactions.
    RelatedPartyVendor,

    // Transaction-related anomalies
    /// Duplicate payment or invoice.
    DuplicatePayment,
    /// Unauthorized transaction.
    UnauthorizedTransaction,
    /// Structured transactions to avoid thresholds.
    StructuredTransaction,

    // Pattern-based anomalies
    /// Circular flow of funds.
    CircularFlow,
    /// Behavioral anomaly (deviation from normal patterns).
    BehavioralAnomaly,
    /// Timing-based anomaly.
    TimingAnomaly,

    // Journal entry anomalies
    /// Manual journal entry anomaly.
    JournalAnomaly,
    /// Manual override of controls.
    ManualOverride,
    /// Missing approval in chain.
    MissingApproval,

    // Statistical anomalies
    /// Statistical outlier.
    StatisticalOutlier,
    /// Distribution anomaly (Benford, etc.).
    DistributionAnomaly,

    // Custom category
    /// User-defined category.
    Custom(String),
}

impl AnomalyCategory {
    /// Derives an AnomalyCategory from an AnomalyType.
    pub fn from_anomaly_type(anomaly_type: &AnomalyType) -> Self {
        match anomaly_type {
            AnomalyType::Fraud(fraud_type) => match fraud_type {
                FraudType::FictitiousVendor | FraudType::ShellCompanyPayment => {
                    AnomalyCategory::FictitiousVendor
                }
                FraudType::Kickback | FraudType::KickbackScheme => AnomalyCategory::VendorKickback,
                FraudType::DuplicatePayment => AnomalyCategory::DuplicatePayment,
                FraudType::SplitTransaction | FraudType::JustBelowThreshold => {
                    AnomalyCategory::StructuredTransaction
                }
                FraudType::SelfApproval
                | FraudType::UnauthorizedApproval
                | FraudType::CollusiveApproval => AnomalyCategory::UnauthorizedTransaction,
                FraudType::TimingAnomaly
                | FraudType::RoundDollarManipulation
                | FraudType::SuspenseAccountAbuse => AnomalyCategory::JournalAnomaly,
                _ => AnomalyCategory::BehavioralAnomaly,
            },
            AnomalyType::Error(error_type) => match error_type {
                ErrorType::DuplicateEntry => AnomalyCategory::DuplicatePayment,
                ErrorType::WrongPeriod
                | ErrorType::BackdatedEntry
                | ErrorType::FutureDatedEntry => AnomalyCategory::TimingAnomaly,
                _ => AnomalyCategory::JournalAnomaly,
            },
            AnomalyType::ProcessIssue(process_type) => match process_type {
                ProcessIssueType::SkippedApproval | ProcessIssueType::IncompleteApprovalChain => {
                    AnomalyCategory::MissingApproval
                }
                ProcessIssueType::ManualOverride | ProcessIssueType::SystemBypass => {
                    AnomalyCategory::ManualOverride
                }
                ProcessIssueType::AfterHoursPosting | ProcessIssueType::WeekendPosting => {
                    AnomalyCategory::TimingAnomaly
                }
                _ => AnomalyCategory::BehavioralAnomaly,
            },
            AnomalyType::Statistical(stat_type) => match stat_type {
                StatisticalAnomalyType::BenfordViolation
                | StatisticalAnomalyType::DistributionShift => AnomalyCategory::DistributionAnomaly,
                _ => AnomalyCategory::StatisticalOutlier,
            },
            AnomalyType::Relational(rel_type) => match rel_type {
                RelationalAnomalyType::CircularTransaction
                | RelationalAnomalyType::CircularIntercompany => AnomalyCategory::CircularFlow,
                _ => AnomalyCategory::BehavioralAnomaly,
            },
            AnomalyType::Custom(s) => AnomalyCategory::Custom(s.clone()),
        }
    }

    /// Returns the category name as a string.
    pub fn name(&self) -> &str {
        match self {
            AnomalyCategory::FictitiousVendor => "fictitious_vendor",
            AnomalyCategory::VendorKickback => "vendor_kickback",
            AnomalyCategory::RelatedPartyVendor => "related_party_vendor",
            AnomalyCategory::DuplicatePayment => "duplicate_payment",
            AnomalyCategory::UnauthorizedTransaction => "unauthorized_transaction",
            AnomalyCategory::StructuredTransaction => "structured_transaction",
            AnomalyCategory::CircularFlow => "circular_flow",
            AnomalyCategory::BehavioralAnomaly => "behavioral_anomaly",
            AnomalyCategory::TimingAnomaly => "timing_anomaly",
            AnomalyCategory::JournalAnomaly => "journal_anomaly",
            AnomalyCategory::ManualOverride => "manual_override",
            AnomalyCategory::MissingApproval => "missing_approval",
            AnomalyCategory::StatisticalOutlier => "statistical_outlier",
            AnomalyCategory::DistributionAnomaly => "distribution_anomaly",
            AnomalyCategory::Custom(s) => s.as_str(),
        }
    }

    /// Returns the ordinal value for ML encoding.
    pub fn ordinal(&self) -> u8 {
        match self {
            AnomalyCategory::FictitiousVendor => 0,
            AnomalyCategory::VendorKickback => 1,
            AnomalyCategory::RelatedPartyVendor => 2,
            AnomalyCategory::DuplicatePayment => 3,
            AnomalyCategory::UnauthorizedTransaction => 4,
            AnomalyCategory::StructuredTransaction => 5,
            AnomalyCategory::CircularFlow => 6,
            AnomalyCategory::BehavioralAnomaly => 7,
            AnomalyCategory::TimingAnomaly => 8,
            AnomalyCategory::JournalAnomaly => 9,
            AnomalyCategory::ManualOverride => 10,
            AnomalyCategory::MissingApproval => 11,
            AnomalyCategory::StatisticalOutlier => 12,
            AnomalyCategory::DistributionAnomaly => 13,
            AnomalyCategory::Custom(_) => 14,
        }
    }

    /// Returns the total number of categories (excluding Custom).
    pub fn category_count() -> usize {
        15 // 14 fixed categories + Custom
    }
}

/// Type of contributing factor for anomaly confidence/severity calculation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum FactorType {
    /// Amount deviation from expected value.
    AmountDeviation,
    /// Proximity to approval/reporting threshold.
    ThresholdProximity,
    /// Timing-related anomaly indicator.
    TimingAnomaly,
    /// Entity risk score contribution.
    EntityRisk,
    /// Pattern match confidence.
    PatternMatch,
    /// Frequency deviation from normal.
    FrequencyDeviation,
    /// Relationship-based anomaly indicator.
    RelationshipAnomaly,
    /// Control bypass indicator.
    ControlBypass,
    /// Benford's Law violation.
    BenfordViolation,
    /// Duplicate indicator.
    DuplicateIndicator,
    /// Approval chain issue.
    ApprovalChainIssue,
    /// Documentation gap.
    DocumentationGap,
    /// Custom factor type.
    Custom,
}

impl FactorType {
    /// Returns the factor type name.
    pub fn name(&self) -> &'static str {
        match self {
            FactorType::AmountDeviation => "amount_deviation",
            FactorType::ThresholdProximity => "threshold_proximity",
            FactorType::TimingAnomaly => "timing_anomaly",
            FactorType::EntityRisk => "entity_risk",
            FactorType::PatternMatch => "pattern_match",
            FactorType::FrequencyDeviation => "frequency_deviation",
            FactorType::RelationshipAnomaly => "relationship_anomaly",
            FactorType::ControlBypass => "control_bypass",
            FactorType::BenfordViolation => "benford_violation",
            FactorType::DuplicateIndicator => "duplicate_indicator",
            FactorType::ApprovalChainIssue => "approval_chain_issue",
            FactorType::DocumentationGap => "documentation_gap",
            FactorType::Custom => "custom",
        }
    }
}

/// Evidence supporting a contributing factor.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FactorEvidence {
    /// Source of the evidence (e.g., "transaction_history", "entity_registry").
    pub source: String,
    /// Raw evidence data.
    pub data: HashMap<String, String>,
}

/// A contributing factor to anomaly confidence/severity.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContributingFactor {
    /// Type of factor.
    pub factor_type: FactorType,
    /// Observed value.
    pub value: f64,
    /// Threshold or expected value.
    pub threshold: f64,
    /// Direction of comparison (true = value > threshold is anomalous).
    pub direction_greater: bool,
    /// Weight of this factor in overall calculation (0.0 - 1.0).
    pub weight: f64,
    /// Human-readable description.
    pub description: String,
    /// Optional supporting evidence.
    pub evidence: Option<FactorEvidence>,
}

impl ContributingFactor {
    /// Creates a new contributing factor.
    pub fn new(
        factor_type: FactorType,
        value: f64,
        threshold: f64,
        direction_greater: bool,
        weight: f64,
        description: &str,
    ) -> Self {
        Self {
            factor_type,
            value,
            threshold,
            direction_greater,
            weight,
            description: description.to_string(),
            evidence: None,
        }
    }

    /// Adds evidence to the factor.
    pub fn with_evidence(mut self, source: &str, data: HashMap<String, String>) -> Self {
        self.evidence = Some(FactorEvidence {
            source: source.to_string(),
            data,
        });
        self
    }

    /// Calculates the factor's contribution to anomaly score.
    pub fn contribution(&self) -> f64 {
        let deviation = if self.direction_greater {
            (self.value - self.threshold).max(0.0)
        } else {
            (self.threshold - self.value).max(0.0)
        };

        // Normalize by threshold to get relative deviation
        let relative_deviation = if self.threshold.abs() > 0.001 {
            deviation / self.threshold.abs()
        } else {
            deviation
        };

        // Apply weight and cap at 1.0
        (relative_deviation * self.weight).min(1.0)
    }
}

/// Enhanced anomaly label with dynamic confidence and severity.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnhancedAnomalyLabel {
    /// Base labeled anomaly (backward compatible).
    pub base: LabeledAnomaly,
    /// Enhanced category classification.
    pub category: AnomalyCategory,
    /// Dynamically calculated confidence (0.0 - 1.0).
    pub enhanced_confidence: f64,
    /// Contextually calculated severity (0.0 - 1.0).
    pub enhanced_severity: f64,
    /// Factors contributing to confidence/severity.
    pub contributing_factors: Vec<ContributingFactor>,
    /// Secondary categories (for multi-label classification).
    pub secondary_categories: Vec<AnomalyCategory>,
}

impl EnhancedAnomalyLabel {
    /// Creates an enhanced label from a base labeled anomaly.
    pub fn from_base(base: LabeledAnomaly) -> Self {
        let category = AnomalyCategory::from_anomaly_type(&base.anomaly_type);
        let enhanced_confidence = base.confidence;
        let enhanced_severity = base.severity as f64 / 5.0;

        Self {
            base,
            category,
            enhanced_confidence,
            enhanced_severity,
            contributing_factors: Vec::new(),
            secondary_categories: Vec::new(),
        }
    }

    /// Sets the enhanced confidence.
    pub fn with_confidence(mut self, confidence: f64) -> Self {
        self.enhanced_confidence = confidence.clamp(0.0, 1.0);
        self
    }

    /// Sets the enhanced severity.
    pub fn with_severity(mut self, severity: f64) -> Self {
        self.enhanced_severity = severity.clamp(0.0, 1.0);
        self
    }

    /// Adds a contributing factor.
    pub fn with_factor(mut self, factor: ContributingFactor) -> Self {
        self.contributing_factors.push(factor);
        self
    }

    /// Adds a secondary category.
    pub fn with_secondary_category(mut self, category: AnomalyCategory) -> Self {
        if !self.secondary_categories.contains(&category) && category != self.category {
            self.secondary_categories.push(category);
        }
        self
    }

    /// Converts to an extended feature vector.
    ///
    /// Returns base features (15) + enhanced features (10) = 25 features.
    pub fn to_features(&self) -> Vec<f64> {
        let mut features = self.base.to_features();

        // Enhanced features
        features.push(self.enhanced_confidence);
        features.push(self.enhanced_severity);
        features.push(self.category.ordinal() as f64 / AnomalyCategory::category_count() as f64);
        features.push(self.secondary_categories.len() as f64);
        features.push(self.contributing_factors.len() as f64);

        // Max factor weight
        let max_weight = self
            .contributing_factors
            .iter()
            .map(|f| f.weight)
            .fold(0.0, f64::max);
        features.push(max_weight);

        // Factor type indicators (binary flags for key factor types)
        let has_control_bypass = self
            .contributing_factors
            .iter()
            .any(|f| f.factor_type == FactorType::ControlBypass);
        features.push(if has_control_bypass { 1.0 } else { 0.0 });

        let has_amount_deviation = self
            .contributing_factors
            .iter()
            .any(|f| f.factor_type == FactorType::AmountDeviation);
        features.push(if has_amount_deviation { 1.0 } else { 0.0 });

        let has_timing = self
            .contributing_factors
            .iter()
            .any(|f| f.factor_type == FactorType::TimingAnomaly);
        features.push(if has_timing { 1.0 } else { 0.0 });

        let has_pattern_match = self
            .contributing_factors
            .iter()
            .any(|f| f.factor_type == FactorType::PatternMatch);
        features.push(if has_pattern_match { 1.0 } else { 0.0 });

        features
    }

    /// Returns the number of features in the enhanced feature vector.
    pub fn feature_count() -> usize {
        25 // 15 base + 10 enhanced
    }

    /// Returns feature names for the enhanced feature vector.
    pub fn feature_names() -> Vec<&'static str> {
        let mut names = LabeledAnomaly::feature_names();
        names.extend(vec![
            "enhanced_confidence",
            "enhanced_severity",
            "category_ordinal",
            "secondary_category_count",
            "contributing_factor_count",
            "max_factor_weight",
            "has_control_bypass",
            "has_amount_deviation",
            "has_timing_factor",
            "has_pattern_match",
        ]);
        names
    }
}

/// Configuration for anomaly rates.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnomalyRateConfig {
    /// Overall anomaly rate (0.0 - 1.0).
    pub total_rate: f64,
    /// Fraud rate as proportion of anomalies.
    pub fraud_rate: f64,
    /// Error rate as proportion of anomalies.
    pub error_rate: f64,
    /// Process issue rate as proportion of anomalies.
    pub process_issue_rate: f64,
    /// Statistical anomaly rate as proportion of anomalies.
    pub statistical_rate: f64,
    /// Relational anomaly rate as proportion of anomalies.
    pub relational_rate: f64,
}

impl Default for AnomalyRateConfig {
    fn default() -> Self {
        Self {
            total_rate: 0.02,         // 2% of transactions are anomalous
            fraud_rate: 0.25,         // 25% of anomalies are fraud
            error_rate: 0.35,         // 35% of anomalies are errors
            process_issue_rate: 0.20, // 20% are process issues
            statistical_rate: 0.15,   // 15% are statistical
            relational_rate: 0.05,    // 5% are relational
        }
    }
}

impl AnomalyRateConfig {
    /// Validates that rates sum to approximately 1.0.
    pub fn validate(&self) -> Result<(), String> {
        let sum = self.fraud_rate
            + self.error_rate
            + self.process_issue_rate
            + self.statistical_rate
            + self.relational_rate;

        if (sum - 1.0).abs() > 0.01 {
            return Err(format!(
                "Anomaly category rates must sum to 1.0, got {}",
                sum
            ));
        }

        if self.total_rate < 0.0 || self.total_rate > 1.0 {
            return Err(format!(
                "Total rate must be between 0.0 and 1.0, got {}",
                self.total_rate
            ));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn test_anomaly_type_category() {
        let fraud = AnomalyType::Fraud(FraudType::SelfApproval);
        assert_eq!(fraud.category(), "Fraud");
        assert!(fraud.is_intentional());

        let error = AnomalyType::Error(ErrorType::DuplicateEntry);
        assert_eq!(error.category(), "Error");
        assert!(!error.is_intentional());
    }

    #[test]
    fn test_labeled_anomaly() {
        let anomaly = LabeledAnomaly::new(
            "ANO001".to_string(),
            AnomalyType::Fraud(FraudType::SelfApproval),
            "JE001".to_string(),
            "JE".to_string(),
            "1000".to_string(),
            NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(),
        )
        .with_description("User approved their own expense report")
        .with_related_entity("USER001");

        assert_eq!(anomaly.severity, 3);
        assert!(anomaly.is_injected);
        assert_eq!(anomaly.related_entities.len(), 1);
    }

    #[test]
    fn test_labeled_anomaly_with_provenance() {
        let anomaly = LabeledAnomaly::new(
            "ANO001".to_string(),
            AnomalyType::Fraud(FraudType::SelfApproval),
            "JE001".to_string(),
            "JE".to_string(),
            "1000".to_string(),
            NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(),
        )
        .with_run_id("run-123")
        .with_generation_seed(42)
        .with_causal_reason(AnomalyCausalReason::RandomRate { base_rate: 0.02 })
        .with_structured_strategy(InjectionStrategy::SelfApproval {
            user_id: "USER001".to_string(),
        })
        .with_scenario("scenario-001")
        .with_original_document_hash("abc123");

        assert_eq!(anomaly.run_id, Some("run-123".to_string()));
        assert_eq!(anomaly.generation_seed, Some(42));
        assert!(anomaly.causal_reason.is_some());
        assert!(anomaly.structured_strategy.is_some());
        assert_eq!(anomaly.scenario_id, Some("scenario-001".to_string()));
        assert_eq!(anomaly.original_document_hash, Some("abc123".to_string()));

        // Check that legacy injection_strategy is also set
        assert_eq!(anomaly.injection_strategy, Some("SelfApproval".to_string()));
    }

    #[test]
    fn test_labeled_anomaly_derivation_chain() {
        let parent = LabeledAnomaly::new(
            "ANO001".to_string(),
            AnomalyType::Fraud(FraudType::DuplicatePayment),
            "JE001".to_string(),
            "JE".to_string(),
            "1000".to_string(),
            NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(),
        );

        let child = LabeledAnomaly::new(
            "ANO002".to_string(),
            AnomalyType::Error(ErrorType::DuplicateEntry),
            "JE002".to_string(),
            "JE".to_string(),
            "1000".to_string(),
            NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(),
        )
        .with_parent_anomaly(&parent.anomaly_id);

        assert_eq!(child.parent_anomaly_id, Some("ANO001".to_string()));
    }

    #[test]
    fn test_injection_strategy_description() {
        let strategy = InjectionStrategy::AmountManipulation {
            original: dec!(1000),
            factor: 2.5,
        };
        assert_eq!(strategy.description(), "Amount multiplied by 2.50");
        assert_eq!(strategy.strategy_type(), "AmountManipulation");

        let strategy = InjectionStrategy::ThresholdAvoidance {
            threshold: dec!(10000),
            adjusted_amount: dec!(9999),
        };
        assert_eq!(
            strategy.description(),
            "Amount adjusted to avoid 10000 threshold"
        );

        let strategy = InjectionStrategy::DateShift {
            days_shifted: -5,
            original_date: NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(),
        };
        assert_eq!(strategy.description(), "Date backdated by 5 days");

        let strategy = InjectionStrategy::DateShift {
            days_shifted: 3,
            original_date: NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(),
        };
        assert_eq!(strategy.description(), "Date forward-dated by 3 days");
    }

    #[test]
    fn test_causal_reason_variants() {
        let reason = AnomalyCausalReason::RandomRate { base_rate: 0.02 };
        if let AnomalyCausalReason::RandomRate { base_rate } = reason {
            assert!((base_rate - 0.02).abs() < 0.001);
        }

        let reason = AnomalyCausalReason::TemporalPattern {
            pattern_name: "year_end_spike".to_string(),
        };
        if let AnomalyCausalReason::TemporalPattern { pattern_name } = reason {
            assert_eq!(pattern_name, "year_end_spike");
        }

        let reason = AnomalyCausalReason::ScenarioStep {
            scenario_type: "kickback".to_string(),
            step_number: 3,
        };
        if let AnomalyCausalReason::ScenarioStep {
            scenario_type,
            step_number,
        } = reason
        {
            assert_eq!(scenario_type, "kickback");
            assert_eq!(step_number, 3);
        }
    }

    #[test]
    fn test_feature_vector_length() {
        let anomaly = LabeledAnomaly::new(
            "ANO001".to_string(),
            AnomalyType::Fraud(FraudType::SelfApproval),
            "JE001".to_string(),
            "JE".to_string(),
            "1000".to_string(),
            NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(),
        );

        let features = anomaly.to_features();
        assert_eq!(features.len(), LabeledAnomaly::feature_count());
        assert_eq!(features.len(), LabeledAnomaly::feature_names().len());
    }

    #[test]
    fn test_feature_vector_with_provenance() {
        let anomaly = LabeledAnomaly::new(
            "ANO001".to_string(),
            AnomalyType::Fraud(FraudType::SelfApproval),
            "JE001".to_string(),
            "JE".to_string(),
            "1000".to_string(),
            NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(),
        )
        .with_scenario("scenario-001")
        .with_parent_anomaly("ANO000");

        let features = anomaly.to_features();

        // Last two features should be 1.0 (has scenario, has parent)
        assert_eq!(features[features.len() - 2], 1.0); // is_scenario_part
        assert_eq!(features[features.len() - 1], 1.0); // is_derived
    }

    #[test]
    fn test_anomaly_summary() {
        let anomalies = vec![
            LabeledAnomaly::new(
                "ANO001".to_string(),
                AnomalyType::Fraud(FraudType::SelfApproval),
                "JE001".to_string(),
                "JE".to_string(),
                "1000".to_string(),
                NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(),
            ),
            LabeledAnomaly::new(
                "ANO002".to_string(),
                AnomalyType::Error(ErrorType::DuplicateEntry),
                "JE002".to_string(),
                "JE".to_string(),
                "1000".to_string(),
                NaiveDate::from_ymd_opt(2024, 1, 16).unwrap(),
            ),
        ];

        let summary = AnomalySummary::from_anomalies(&anomalies);

        assert_eq!(summary.total_count, 2);
        assert_eq!(summary.by_category.get("Fraud"), Some(&1));
        assert_eq!(summary.by_category.get("Error"), Some(&1));
    }

    #[test]
    fn test_rate_config_validation() {
        let config = AnomalyRateConfig::default();
        assert!(config.validate().is_ok());

        let bad_config = AnomalyRateConfig {
            fraud_rate: 0.5,
            error_rate: 0.5,
            process_issue_rate: 0.5, // Sum > 1.0
            ..Default::default()
        };
        assert!(bad_config.validate().is_err());
    }

    #[test]
    fn test_injection_strategy_serialization() {
        let strategy = InjectionStrategy::SoDViolation {
            duty1: "CreatePO".to_string(),
            duty2: "ApprovePO".to_string(),
            violating_user: "USER001".to_string(),
        };

        let json = serde_json::to_string(&strategy).unwrap();
        let deserialized: InjectionStrategy = serde_json::from_str(&json).unwrap();

        assert_eq!(strategy, deserialized);
    }

    #[test]
    fn test_labeled_anomaly_serialization_with_provenance() {
        let anomaly = LabeledAnomaly::new(
            "ANO001".to_string(),
            AnomalyType::Fraud(FraudType::SelfApproval),
            "JE001".to_string(),
            "JE".to_string(),
            "1000".to_string(),
            NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(),
        )
        .with_run_id("run-123")
        .with_generation_seed(42)
        .with_causal_reason(AnomalyCausalReason::RandomRate { base_rate: 0.02 });

        let json = serde_json::to_string(&anomaly).unwrap();
        let deserialized: LabeledAnomaly = serde_json::from_str(&json).unwrap();

        assert_eq!(anomaly.run_id, deserialized.run_id);
        assert_eq!(anomaly.generation_seed, deserialized.generation_seed);
    }

    // ========================================
    // FR-003 ENHANCED TAXONOMY TESTS
    // ========================================

    #[test]
    fn test_anomaly_category_from_anomaly_type() {
        // Fraud mappings
        let fraud_vendor = AnomalyType::Fraud(FraudType::FictitiousVendor);
        assert_eq!(
            AnomalyCategory::from_anomaly_type(&fraud_vendor),
            AnomalyCategory::FictitiousVendor
        );

        let fraud_kickback = AnomalyType::Fraud(FraudType::KickbackScheme);
        assert_eq!(
            AnomalyCategory::from_anomaly_type(&fraud_kickback),
            AnomalyCategory::VendorKickback
        );

        let fraud_structured = AnomalyType::Fraud(FraudType::SplitTransaction);
        assert_eq!(
            AnomalyCategory::from_anomaly_type(&fraud_structured),
            AnomalyCategory::StructuredTransaction
        );

        // Error mappings
        let error_duplicate = AnomalyType::Error(ErrorType::DuplicateEntry);
        assert_eq!(
            AnomalyCategory::from_anomaly_type(&error_duplicate),
            AnomalyCategory::DuplicatePayment
        );

        // Process issue mappings
        let process_skip = AnomalyType::ProcessIssue(ProcessIssueType::SkippedApproval);
        assert_eq!(
            AnomalyCategory::from_anomaly_type(&process_skip),
            AnomalyCategory::MissingApproval
        );

        // Relational mappings
        let relational_circular =
            AnomalyType::Relational(RelationalAnomalyType::CircularTransaction);
        assert_eq!(
            AnomalyCategory::from_anomaly_type(&relational_circular),
            AnomalyCategory::CircularFlow
        );
    }

    #[test]
    fn test_anomaly_category_ordinal() {
        assert_eq!(AnomalyCategory::FictitiousVendor.ordinal(), 0);
        assert_eq!(AnomalyCategory::VendorKickback.ordinal(), 1);
        assert_eq!(AnomalyCategory::Custom("test".to_string()).ordinal(), 14);
    }

    #[test]
    fn test_contributing_factor() {
        let factor = ContributingFactor::new(
            FactorType::AmountDeviation,
            15000.0,
            10000.0,
            true,
            0.5,
            "Amount exceeds threshold",
        );

        assert_eq!(factor.factor_type, FactorType::AmountDeviation);
        assert_eq!(factor.value, 15000.0);
        assert_eq!(factor.threshold, 10000.0);
        assert!(factor.direction_greater);

        // Contribution: (15000 - 10000) / 10000 * 0.5 = 0.25
        let contribution = factor.contribution();
        assert!((contribution - 0.25).abs() < 0.01);
    }

    #[test]
    fn test_contributing_factor_with_evidence() {
        let mut data = HashMap::new();
        data.insert("expected".to_string(), "10000".to_string());
        data.insert("actual".to_string(), "15000".to_string());

        let factor = ContributingFactor::new(
            FactorType::AmountDeviation,
            15000.0,
            10000.0,
            true,
            0.5,
            "Amount deviation detected",
        )
        .with_evidence("transaction_history", data);

        assert!(factor.evidence.is_some());
        let evidence = factor.evidence.unwrap();
        assert_eq!(evidence.source, "transaction_history");
        assert_eq!(evidence.data.get("expected"), Some(&"10000".to_string()));
    }

    #[test]
    fn test_enhanced_anomaly_label() {
        let base = LabeledAnomaly::new(
            "ANO001".to_string(),
            AnomalyType::Fraud(FraudType::DuplicatePayment),
            "JE001".to_string(),
            "JE".to_string(),
            "1000".to_string(),
            NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(),
        );

        let enhanced = EnhancedAnomalyLabel::from_base(base)
            .with_confidence(0.85)
            .with_severity(0.7)
            .with_factor(ContributingFactor::new(
                FactorType::DuplicateIndicator,
                1.0,
                0.5,
                true,
                0.4,
                "Duplicate payment detected",
            ))
            .with_secondary_category(AnomalyCategory::StructuredTransaction);

        assert_eq!(enhanced.category, AnomalyCategory::DuplicatePayment);
        assert_eq!(enhanced.enhanced_confidence, 0.85);
        assert_eq!(enhanced.enhanced_severity, 0.7);
        assert_eq!(enhanced.contributing_factors.len(), 1);
        assert_eq!(enhanced.secondary_categories.len(), 1);
    }

    #[test]
    fn test_enhanced_anomaly_label_features() {
        let base = LabeledAnomaly::new(
            "ANO001".to_string(),
            AnomalyType::Fraud(FraudType::SelfApproval),
            "JE001".to_string(),
            "JE".to_string(),
            "1000".to_string(),
            NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(),
        );

        let enhanced = EnhancedAnomalyLabel::from_base(base)
            .with_confidence(0.9)
            .with_severity(0.8)
            .with_factor(ContributingFactor::new(
                FactorType::ControlBypass,
                1.0,
                0.0,
                true,
                0.5,
                "Control bypass detected",
            ));

        let features = enhanced.to_features();

        // Should have 25 features (15 base + 10 enhanced)
        assert_eq!(features.len(), EnhancedAnomalyLabel::feature_count());
        assert_eq!(features.len(), 25);

        // Check enhanced confidence is in features
        assert_eq!(features[15], 0.9); // enhanced_confidence

        // Check has_control_bypass flag
        assert_eq!(features[21], 1.0); // has_control_bypass
    }

    #[test]
    fn test_enhanced_anomaly_label_feature_names() {
        let names = EnhancedAnomalyLabel::feature_names();
        assert_eq!(names.len(), 25);
        assert!(names.contains(&"enhanced_confidence"));
        assert!(names.contains(&"enhanced_severity"));
        assert!(names.contains(&"has_control_bypass"));
    }

    #[test]
    fn test_factor_type_names() {
        assert_eq!(FactorType::AmountDeviation.name(), "amount_deviation");
        assert_eq!(FactorType::ThresholdProximity.name(), "threshold_proximity");
        assert_eq!(FactorType::ControlBypass.name(), "control_bypass");
    }

    #[test]
    fn test_anomaly_category_serialization() {
        let category = AnomalyCategory::CircularFlow;
        let json = serde_json::to_string(&category).unwrap();
        let deserialized: AnomalyCategory = serde_json::from_str(&json).unwrap();
        assert_eq!(category, deserialized);

        let custom = AnomalyCategory::Custom("custom_type".to_string());
        let json = serde_json::to_string(&custom).unwrap();
        let deserialized: AnomalyCategory = serde_json::from_str(&json).unwrap();
        assert_eq!(custom, deserialized);
    }

    #[test]
    fn test_enhanced_label_secondary_category_dedup() {
        let base = LabeledAnomaly::new(
            "ANO001".to_string(),
            AnomalyType::Fraud(FraudType::DuplicatePayment),
            "JE001".to_string(),
            "JE".to_string(),
            "1000".to_string(),
            NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(),
        );

        let enhanced = EnhancedAnomalyLabel::from_base(base)
            // Try to add the primary category as secondary (should be ignored)
            .with_secondary_category(AnomalyCategory::DuplicatePayment)
            // Add a valid secondary
            .with_secondary_category(AnomalyCategory::TimingAnomaly)
            // Try to add duplicate secondary (should be ignored)
            .with_secondary_category(AnomalyCategory::TimingAnomaly);

        // Should only have 1 secondary category (TimingAnomaly)
        assert_eq!(enhanced.secondary_categories.len(), 1);
        assert_eq!(
            enhanced.secondary_categories[0],
            AnomalyCategory::TimingAnomaly
        );
    }
}
