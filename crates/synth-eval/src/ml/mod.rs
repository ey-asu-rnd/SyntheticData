//! ML-readiness evaluation module.
//!
//! Validates that generated data is suitable for machine learning tasks
//! including feature distributions, label quality, and graph structure.

mod features;
mod graph;
mod labels;
mod splits;

pub use features::{FeatureAnalysis, FeatureAnalyzer, FeatureStats};
pub use graph::{GraphAnalysis, GraphAnalyzer, GraphMetrics};
pub use labels::{LabelAnalysis, LabelAnalyzer, LabelDistribution};
pub use splits::{SplitAnalysis, SplitAnalyzer, SplitMetrics};

use serde::{Deserialize, Serialize};

/// Combined ML-readiness evaluation results.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MLReadinessEvaluation {
    /// Feature distribution analysis.
    pub features: Option<FeatureAnalysis>,
    /// Label quality analysis.
    pub labels: Option<LabelAnalysis>,
    /// Train/test split analysis.
    pub splits: Option<SplitAnalysis>,
    /// Graph structure analysis.
    pub graph: Option<GraphAnalysis>,
    /// Overall ML-readiness score (0.0-1.0).
    pub overall_score: f64,
    /// Whether data meets ML-readiness criteria.
    pub passes: bool,
    /// ML-readiness issues found.
    pub issues: Vec<String>,
    /// ML-readiness failures (alias for issues).
    pub failures: Vec<String>,
}

impl MLReadinessEvaluation {
    /// Create a new empty evaluation.
    pub fn new() -> Self {
        Self {
            features: None,
            labels: None,
            splits: None,
            graph: None,
            overall_score: 1.0,
            passes: true,
            issues: Vec::new(),
            failures: Vec::new(),
        }
    }

    /// Check all results against thresholds.
    pub fn check_thresholds(&mut self, thresholds: &crate::config::EvaluationThresholds) {
        self.issues.clear();
        self.failures.clear();
        let mut scores = Vec::new();

        if let Some(ref labels) = self.labels {
            // Check anomaly rate is within expected range
            if labels.anomaly_rate < thresholds.anomaly_rate_min {
                self.issues.push(format!(
                    "Anomaly rate {} < {} (min threshold)",
                    labels.anomaly_rate, thresholds.anomaly_rate_min
                ));
            }
            if labels.anomaly_rate > thresholds.anomaly_rate_max {
                self.issues.push(format!(
                    "Anomaly rate {} > {} (max threshold)",
                    labels.anomaly_rate, thresholds.anomaly_rate_max
                ));
            }

            // Check label coverage
            if labels.label_coverage < thresholds.label_coverage_min {
                self.issues.push(format!(
                    "Label coverage {} < {} (threshold)",
                    labels.label_coverage, thresholds.label_coverage_min
                ));
            }

            scores.push(labels.quality_score);
        }

        if let Some(ref splits) = self.splits {
            if !splits.is_valid {
                self.issues
                    .push("Train/test split validation failed".to_string());
            }
            scores.push(if splits.is_valid { 1.0 } else { 0.0 });
        }

        if let Some(ref graph) = self.graph {
            if graph.connectivity_score < thresholds.graph_connectivity_min {
                self.issues.push(format!(
                    "Graph connectivity {} < {} (threshold)",
                    graph.connectivity_score, thresholds.graph_connectivity_min
                ));
            }
            scores.push(graph.connectivity_score);
        }

        if let Some(ref features) = self.features {
            scores.push(features.quality_score);
        }

        self.overall_score = if scores.is_empty() {
            1.0
        } else {
            scores.iter().sum::<f64>() / scores.len() as f64
        };

        // Sync failures with issues
        self.failures = self.issues.clone();
        self.passes = self.issues.is_empty();
    }
}

impl Default for MLReadinessEvaluation {
    fn default() -> Self {
        Self::new()
    }
}
