//! Root fingerprint structure.

use serde::{Deserialize, Serialize};

use super::{
    AnomalyFingerprint, CorrelationFingerprint, IntegrityFingerprint, Manifest, PrivacyAudit,
    RulesFingerprint, SchemaFingerprint, StatisticsFingerprint,
};

/// The root fingerprint structure containing all extracted components.
///
/// A fingerprint captures the statistical properties of a dataset without
/// storing any individual records, enabling privacy-preserving synthetic
/// data generation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Fingerprint {
    /// Metadata about the fingerprint (version, source, privacy config).
    pub manifest: Manifest,

    /// Schema information (tables, columns, types, relationships).
    pub schema: SchemaFingerprint,

    /// Statistical distributions for numeric and categorical columns.
    pub statistics: StatisticsFingerprint,

    /// Correlation matrices and copulas for preserving relationships.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub correlations: Option<CorrelationFingerprint>,

    /// Referential integrity (foreign keys, cardinalities).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub integrity: Option<IntegrityFingerprint>,

    /// Business rules (balance constraints, approval thresholds).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rules: Option<RulesFingerprint>,

    /// Anomaly patterns (rates, type distribution, temporal patterns).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub anomalies: Option<AnomalyFingerprint>,

    /// Privacy audit trail documenting all privacy decisions.
    pub privacy_audit: PrivacyAudit,
}

impl Fingerprint {
    /// Create a new fingerprint with required components.
    pub fn new(
        manifest: Manifest,
        schema: SchemaFingerprint,
        statistics: StatisticsFingerprint,
        privacy_audit: PrivacyAudit,
    ) -> Self {
        Self {
            manifest,
            schema,
            statistics,
            correlations: None,
            integrity: None,
            rules: None,
            anomalies: None,
            privacy_audit,
        }
    }

    /// Add correlation fingerprint.
    pub fn with_correlations(mut self, correlations: CorrelationFingerprint) -> Self {
        self.correlations = Some(correlations);
        self
    }

    /// Add integrity fingerprint.
    pub fn with_integrity(mut self, integrity: IntegrityFingerprint) -> Self {
        self.integrity = Some(integrity);
        self
    }

    /// Add rules fingerprint.
    pub fn with_rules(mut self, rules: RulesFingerprint) -> Self {
        self.rules = Some(rules);
        self
    }

    /// Add anomaly fingerprint.
    pub fn with_anomalies(mut self, anomalies: AnomalyFingerprint) -> Self {
        self.anomalies = Some(anomalies);
        self
    }

    /// Get the fingerprint version.
    pub fn version(&self) -> &str {
        &self.manifest.version
    }

    /// Check if the fingerprint has correlation data.
    pub fn has_correlations(&self) -> bool {
        self.correlations.is_some()
    }

    /// Check if the fingerprint has integrity constraints.
    pub fn has_integrity(&self) -> bool {
        self.integrity.is_some()
    }

    /// Check if the fingerprint has business rules.
    pub fn has_rules(&self) -> bool {
        self.rules.is_some()
    }

    /// Check if the fingerprint has anomaly patterns.
    pub fn has_anomalies(&self) -> bool {
        self.anomalies.is_some()
    }

    /// Get total epsilon spent on privacy.
    pub fn epsilon_spent(&self) -> f64 {
        self.privacy_audit.total_epsilon_spent
    }
}
