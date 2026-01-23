//! Anomaly fingerprint models.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

/// Anomaly fingerprint containing anomaly patterns and rates.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnomalyFingerprint {
    /// Overall anomaly statistics.
    pub overall: AnomalyOverview,

    /// Anomaly profiles by type.
    pub profiles: Vec<AnomalyProfile>,

    /// Temporal patterns of anomalies.
    pub temporal_patterns: TemporalAnomalyPatterns,

    /// Entity-level anomaly patterns.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub entity_patterns: Option<EntityAnomalyPatterns>,

    /// Clustering patterns.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub clustering: Option<AnomalyClustering>,
}

impl AnomalyFingerprint {
    /// Create a new anomaly fingerprint.
    pub fn new(overall: AnomalyOverview) -> Self {
        Self {
            overall,
            profiles: Vec::new(),
            temporal_patterns: TemporalAnomalyPatterns::default(),
            entity_patterns: None,
            clustering: None,
        }
    }

    /// Add an anomaly profile.
    pub fn add_profile(&mut self, profile: AnomalyProfile) {
        self.profiles.push(profile);
    }
}

/// Overall anomaly statistics.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnomalyOverview {
    /// Total records analyzed.
    pub total_records: u64,

    /// Total anomalies detected/labeled.
    pub total_anomalies: u64,

    /// Overall anomaly rate.
    pub anomaly_rate: f64,

    /// Distribution by anomaly category.
    pub category_distribution: HashMap<String, f64>,

    /// Number of distinct anomaly types.
    pub type_count: usize,

    /// Whether anomalies are labeled in source data.
    pub has_labels: bool,

    /// Label field name if present.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label_field: Option<String>,
}

impl AnomalyOverview {
    /// Create a new anomaly overview.
    pub fn new(total_records: u64, total_anomalies: u64) -> Self {
        let anomaly_rate = if total_records > 0 {
            total_anomalies as f64 / total_records as f64
        } else {
            0.0
        };

        Self {
            total_records,
            total_anomalies,
            anomaly_rate,
            category_distribution: HashMap::new(),
            type_count: 0,
            has_labels: false,
            label_field: None,
        }
    }
}

/// Profile for a specific anomaly type.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnomalyProfile {
    /// Anomaly type identifier.
    pub anomaly_type: String,

    /// Human-readable name.
    pub name: String,

    /// Category (fraud, error, process_issue, statistical, relational).
    pub category: AnomalyCategory,

    /// Rate of this anomaly type.
    pub rate: f64,

    /// Count of this anomaly type.
    pub count: u64,

    /// Severity level (1-5).
    pub severity: u8,

    /// Characteristics specific to this anomaly type.
    pub characteristics: AnomalyCharacteristics,

    /// Detection features useful for ML.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub detection_features: Vec<String>,
}

impl AnomalyProfile {
    /// Create a new anomaly profile.
    pub fn new(
        anomaly_type: impl Into<String>,
        name: impl Into<String>,
        category: AnomalyCategory,
        rate: f64,
    ) -> Self {
        Self {
            anomaly_type: anomaly_type.into(),
            name: name.into(),
            category,
            rate,
            count: 0,
            severity: 3,
            characteristics: AnomalyCharacteristics::default(),
            detection_features: Vec::new(),
        }
    }
}

/// Anomaly category.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AnomalyCategory {
    /// Intentional misrepresentation.
    Fraud,
    /// Unintentional mistakes.
    Error,
    /// Process or control failures.
    ProcessIssue,
    /// Statistical outliers.
    Statistical,
    /// Relationship anomalies.
    Relational,
}

impl std::fmt::Display for AnomalyCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Fraud => write!(f, "fraud"),
            Self::Error => write!(f, "error"),
            Self::ProcessIssue => write!(f, "process_issue"),
            Self::Statistical => write!(f, "statistical"),
            Self::Relational => write!(f, "relational"),
        }
    }
}

/// Characteristics of an anomaly type.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AnomalyCharacteristics {
    /// Amount-related characteristics.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub amount: Option<AmountCharacteristics>,

    /// Timing-related characteristics.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timing: Option<TimingCharacteristics>,

    /// Entity-related characteristics.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub entity: Option<EntityCharacteristics>,

    /// Additional properties.
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub properties: HashMap<String, String>,
}

/// Amount-related anomaly characteristics.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AmountCharacteristics {
    /// Whether amounts cluster near thresholds.
    pub threshold_adjacent: bool,

    /// Typical amount range.
    pub typical_range: Option<(f64, f64)>,

    /// Whether amounts are unusually round.
    pub round_amounts: bool,

    /// Distribution of amounts for this anomaly type.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub amount_distribution: Option<AmountDistribution>,
}

/// Distribution of amounts.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AmountDistribution {
    /// Mean amount.
    pub mean: f64,
    /// Standard deviation.
    pub std_dev: f64,
    /// Median.
    pub median: f64,
    /// Percentile 95.
    pub p95: f64,
}

/// Timing-related anomaly characteristics.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimingCharacteristics {
    /// Whether anomalies cluster at period end.
    pub period_end_spike: bool,

    /// Whether anomalies occur outside business hours.
    pub off_hours: bool,

    /// Day-of-week distribution (higher = more anomalies).
    pub day_of_week_weights: [f64; 7],

    /// Month distribution (higher = more anomalies).
    pub month_weights: [f64; 12],
}

/// Entity-related anomaly characteristics.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntityCharacteristics {
    /// Whether certain entities are repeat offenders.
    pub repeat_offender_rate: f64,

    /// Whether high-volume entities have more anomalies.
    pub volume_correlation: f64,

    /// Typical number of anomalies per affected entity.
    pub anomalies_per_entity: f64,
}

/// Temporal patterns of anomalies.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TemporalAnomalyPatterns {
    /// Rate multiplier for year-end periods.
    pub year_end_multiplier: f64,

    /// Rate multiplier for quarter-end periods.
    pub quarter_end_multiplier: f64,

    /// Rate multiplier for month-end periods.
    pub month_end_multiplier: f64,

    /// Monthly anomaly rates.
    pub monthly_rates: Vec<MonthlyRate>,

    /// Trend direction (-1 = decreasing, 0 = stable, 1 = increasing).
    pub trend: i8,

    /// Seasonality strength (0.0 = none, 1.0 = strong).
    pub seasonality_strength: f64,
}

/// Monthly anomaly rate.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonthlyRate {
    /// Year-month (YYYY-MM).
    pub period: String,
    /// Anomaly rate for this period.
    pub rate: f64,
    /// Count of anomalies.
    pub count: u64,
}

/// Entity-level anomaly patterns.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntityAnomalyPatterns {
    /// Proportion of entities with any anomalies.
    pub affected_entity_rate: f64,

    /// Distribution of anomalies per entity.
    pub anomalies_per_entity_distribution: Vec<(u32, f64)>,

    /// Entity types with higher anomaly rates.
    pub high_risk_entity_types: Vec<EntityRisk>,
}

/// Entity risk profile.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntityRisk {
    /// Entity type (e.g., "vendor", "customer").
    pub entity_type: String,
    /// Relative risk (1.0 = baseline).
    pub relative_risk: f64,
    /// Count of entities.
    pub entity_count: u64,
}

/// Anomaly clustering patterns.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnomalyClustering {
    /// Whether anomalies cluster in time.
    pub temporal_clustering: bool,

    /// Typical cluster size.
    pub typical_cluster_size: f64,

    /// Typical time window for clusters (in days).
    pub cluster_window_days: f64,

    /// Whether anomalies cluster by entity.
    pub entity_clustering: bool,

    /// Whether multiple anomaly types co-occur.
    pub type_co_occurrence: HashMap<String, Vec<String>>,
}
