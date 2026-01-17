//! Statistical quality evaluation module.
//!
//! Provides statistical tests and analyses for validating that generated
//! synthetic data follows expected distributions.

mod benford;
mod amount_distribution;
mod line_item;
mod temporal;

pub use benford::{BenfordAnalysis, BenfordAnalyzer, BenfordConformity};
pub use amount_distribution::{AmountDistributionAnalysis, AmountDistributionAnalyzer};
pub use line_item::{LineItemAnalysis, LineItemAnalyzer};
pub use temporal::{TemporalAnalysis, TemporalAnalyzer};

use serde::{Deserialize, Serialize};

/// Combined statistical evaluation results.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatisticalEvaluation {
    /// Benford's Law analysis results.
    pub benford: Option<BenfordAnalysis>,
    /// Amount distribution analysis results.
    pub amount_distribution: Option<AmountDistributionAnalysis>,
    /// Line item distribution analysis results.
    pub line_item: Option<LineItemAnalysis>,
    /// Temporal pattern analysis results.
    pub temporal: Option<TemporalAnalysis>,
    /// Overall pass/fail status.
    pub passes: bool,
    /// Summary of failed checks.
    pub failures: Vec<String>,
    /// Summary of issues (alias for failures).
    pub issues: Vec<String>,
    /// Overall statistical quality score (0.0-1.0).
    pub overall_score: f64,
}

impl StatisticalEvaluation {
    /// Create a new empty evaluation.
    pub fn new() -> Self {
        Self {
            benford: None,
            amount_distribution: None,
            line_item: None,
            temporal: None,
            passes: true,
            failures: Vec::new(),
            issues: Vec::new(),
            overall_score: 1.0,
        }
    }

    /// Check all results against thresholds and update pass status.
    pub fn check_thresholds(&mut self, thresholds: &crate::config::EvaluationThresholds) {
        self.failures.clear();
        self.issues.clear();
        let mut scores = Vec::new();

        if let Some(ref benford) = self.benford {
            if benford.p_value < thresholds.benford_p_value_min {
                self.failures.push(format!(
                    "Benford p-value {} < {} (threshold)",
                    benford.p_value, thresholds.benford_p_value_min
                ));
            }
            if benford.mad > thresholds.benford_mad_max {
                self.failures.push(format!(
                    "Benford MAD {} > {} (threshold)",
                    benford.mad, thresholds.benford_mad_max
                ));
            }
            // Benford score: higher p-value and lower MAD are better
            let p_score = (benford.p_value / 0.5).min(1.0);
            let mad_score = 1.0 - (benford.mad / 0.05).min(1.0);
            scores.push((p_score + mad_score) / 2.0);
        }

        if let Some(ref amount) = self.amount_distribution {
            if let Some(p_value) = amount.lognormal_ks_pvalue {
                if p_value < thresholds.amount_ks_p_value_min {
                    self.failures.push(format!(
                        "Amount KS p-value {} < {} (threshold)",
                        p_value, thresholds.amount_ks_p_value_min
                    ));
                }
                scores.push((p_value / 0.5).min(1.0));
            }
        }

        if let Some(ref temporal) = self.temporal {
            if temporal.pattern_correlation < thresholds.temporal_correlation_min {
                self.failures.push(format!(
                    "Temporal correlation {} < {} (threshold)",
                    temporal.pattern_correlation, thresholds.temporal_correlation_min
                ));
            }
            scores.push(temporal.pattern_correlation);
        }

        // Sync issues with failures
        self.issues = self.failures.clone();
        self.passes = self.failures.is_empty();

        // Calculate overall score
        self.overall_score = if scores.is_empty() {
            1.0
        } else {
            scores.iter().sum::<f64>() / scores.len() as f64
        };
    }
}

impl Default for StatisticalEvaluation {
    fn default() -> Self {
        Self::new()
    }
}
