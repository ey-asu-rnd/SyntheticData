//! Privacy mechanisms for fingerprint extraction.
//!
//! This module provides:
//! - **Differential Privacy**: Laplace noise with epsilon budgeting
//! - **K-Anonymity**: Suppression of rare categorical values
//! - **Audit Trail**: Complete logging of privacy decisions

mod audit;
mod differential;
mod kanonymity;

pub use audit::*;
pub use differential::*;
pub use kanonymity::*;

use crate::error::{FingerprintError, FingerprintResult};
use crate::models::{PrivacyAction, PrivacyActionType, PrivacyAudit, PrivacyLevel, PrivacyMetadata};

/// Configuration for privacy mechanisms.
#[derive(Debug, Clone)]
pub struct PrivacyConfig {
    /// Differential privacy epsilon budget.
    pub epsilon: f64,
    /// K-anonymity threshold.
    pub k_anonymity: u32,
    /// Outlier percentile for winsorization.
    pub outlier_percentile: f64,
    /// Minimum occurrence for categorical values.
    pub min_occurrence: u32,
    /// Fields to always suppress.
    pub suppressed_fields: Vec<String>,
}

impl PrivacyConfig {
    /// Create from privacy level.
    pub fn from_level(level: PrivacyLevel) -> Self {
        let metadata = PrivacyMetadata::from_level(level);
        Self {
            epsilon: metadata.epsilon,
            k_anonymity: metadata.k_anonymity,
            outlier_percentile: metadata.outlier_percentile,
            min_occurrence: metadata.min_occurrence,
            suppressed_fields: metadata.suppressed_fields,
        }
    }

    /// Create custom configuration.
    pub fn custom(epsilon: f64, k_anonymity: u32) -> Self {
        Self {
            epsilon,
            k_anonymity,
            outlier_percentile: 95.0,
            min_occurrence: k_anonymity,
            suppressed_fields: Vec::new(),
        }
    }
}

impl Default for PrivacyConfig {
    fn default() -> Self {
        Self::from_level(PrivacyLevel::Standard)
    }
}

/// Privacy engine that applies privacy mechanisms during extraction.
pub struct PrivacyEngine {
    config: PrivacyConfig,
    audit: PrivacyAudit,
    laplace: LaplaceMechanism,
    kanon: KAnonymity,
}

impl PrivacyEngine {
    /// Create a new privacy engine.
    pub fn new(config: PrivacyConfig) -> Self {
        Self {
            audit: PrivacyAudit::new(config.epsilon, config.k_anonymity),
            laplace: LaplaceMechanism::new(config.epsilon),
            kanon: KAnonymity::new(config.k_anonymity, config.min_occurrence),
            config,
        }
    }

    /// Create from privacy level.
    pub fn from_level(level: PrivacyLevel) -> Self {
        Self::new(PrivacyConfig::from_level(level))
    }

    /// Check if budget allows spending epsilon.
    pub fn can_spend(&self, epsilon: f64) -> bool {
        self.audit.remaining_budget() >= epsilon
    }

    /// Add noise to a numeric value.
    pub fn add_noise(&mut self, value: f64, sensitivity: f64, target: &str) -> FingerprintResult<f64> {
        let epsilon_per_query = self.config.epsilon / 100.0; // Budget across many queries

        if !self.can_spend(epsilon_per_query) {
            return Err(FingerprintError::PrivacyBudgetExhausted {
                spent: self.audit.total_epsilon_spent,
                limit: self.config.epsilon,
            });
        }

        let noised = self.laplace.add_noise(value, sensitivity, epsilon_per_query);

        let action = PrivacyAction::new(
            PrivacyActionType::LaplaceNoise,
            target,
            format!("Added Laplace noise with sensitivity={}, epsilon={}", sensitivity, epsilon_per_query),
            "Differential privacy protection",
        )
        .with_epsilon(epsilon_per_query);

        self.audit.record_action(action);
        Ok(noised)
    }

    /// Add noise to a count.
    pub fn add_noise_to_count(&mut self, count: u64, target: &str) -> FingerprintResult<u64> {
        let noised = self.add_noise(count as f64, 1.0, target)?;
        Ok(noised.max(0.0).round() as u64)
    }

    /// Filter categorical frequencies by k-anonymity.
    pub fn filter_categories(
        &mut self,
        frequencies: Vec<(String, u64)>,
        total: u64,
        target: &str,
    ) -> Vec<(String, f64)> {
        let (kept, suppressed) = self.kanon.filter_frequencies(frequencies, total);

        if suppressed > 0 {
            let action = PrivacyAction::new(
                PrivacyActionType::Suppression,
                target,
                format!("Suppressed {} rare categories below k={}", suppressed, self.config.k_anonymity),
                "K-anonymity protection",
            );
            self.audit.record_action(action);
        }

        kept
    }

    /// Winsorize outliers in a sorted list.
    pub fn winsorize(&mut self, values: &mut [f64], target: &str) {
        let percentile = self.config.outlier_percentile;
        let (low_count, high_count) = winsorize_values(values, percentile);

        if low_count > 0 || high_count > 0 {
            let action = PrivacyAction::new(
                PrivacyActionType::Winsorization,
                target,
                format!(
                    "Winsorized {} low and {} high outliers at {}th percentile",
                    low_count, high_count, percentile
                ),
                "Outlier protection",
            );
            self.audit.record_action(action);
        }
    }

    /// Check if a field should be suppressed.
    pub fn should_suppress_field(&self, field: &str) -> bool {
        self.config.suppressed_fields.iter().any(|f| f == field)
    }

    /// Record a custom privacy action.
    pub fn record_action(&mut self, action: PrivacyAction) {
        self.audit.record_action(action);
    }

    /// Get the privacy audit.
    pub fn audit(&self) -> &PrivacyAudit {
        &self.audit
    }

    /// Consume and return the privacy audit.
    pub fn into_audit(self) -> PrivacyAudit {
        self.audit
    }

    /// Get remaining epsilon budget.
    pub fn remaining_budget(&self) -> f64 {
        self.audit.remaining_budget()
    }
}

/// Winsorize values at given percentile.
fn winsorize_values(values: &mut [f64], percentile: f64) -> (usize, usize) {
    if values.is_empty() {
        return (0, 0);
    }

    let n = values.len();
    let low_idx = ((100.0 - percentile) / 100.0 * n as f64).floor() as usize;
    let high_idx = (percentile / 100.0 * n as f64).ceil() as usize;

    // Sort to find percentile values
    let mut sorted = values.to_vec();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());

    let low_threshold = sorted.get(low_idx).copied().unwrap_or(f64::MIN);
    let high_threshold = sorted.get(high_idx.min(n - 1)).copied().unwrap_or(f64::MAX);

    let mut low_count = 0;
    let mut high_count = 0;

    for v in values.iter_mut() {
        if *v < low_threshold {
            *v = low_threshold;
            low_count += 1;
        } else if *v > high_threshold {
            *v = high_threshold;
            high_count += 1;
        }
    }

    (low_count, high_count)
}
