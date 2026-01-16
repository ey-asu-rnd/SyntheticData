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
    /// Injection strategy used (if injected).
    pub injection_strategy: Option<String>,
    /// Cluster ID if part of an anomaly cluster.
    pub cluster_id: Option<String>,
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

    /// Sets the injection strategy.
    pub fn with_injection_strategy(mut self, strategy: &str) -> Self {
        self.injection_strategy = Some(strategy.to_string());
        self
    }

    /// Sets the cluster ID.
    pub fn with_cluster(mut self, cluster_id: &str) -> Self {
        self.cluster_id = Some(cluster_id.to_string());
        self
    }

    /// Converts to a feature vector for ML.
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

        features
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
}
