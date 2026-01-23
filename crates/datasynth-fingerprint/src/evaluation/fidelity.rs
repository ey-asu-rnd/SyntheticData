//! Fidelity evaluation comparing synthetic data to fingerprints.

use std::collections::HashMap;

use serde::Serialize;

use crate::error::FingerprintResult;
use crate::extraction::{DataSource, FingerprintExtractor};
use crate::models::{Fingerprint, NumericStats, StatisticsFingerprint};

/// Configuration for fidelity evaluation.
#[derive(Debug, Clone)]
pub struct FidelityConfig {
    /// Minimum acceptable overall fidelity score (0.0-1.0).
    pub threshold: f64,
    /// Weight for statistical fidelity.
    pub statistical_weight: f64,
    /// Weight for correlation fidelity.
    pub correlation_weight: f64,
    /// Weight for schema fidelity.
    pub schema_weight: f64,
    /// Weight for rule compliance.
    pub rule_weight: f64,
    /// Weight for anomaly fidelity.
    pub anomaly_weight: f64,
}

impl Default for FidelityConfig {
    fn default() -> Self {
        Self {
            threshold: 0.8,
            statistical_weight: 0.30,
            correlation_weight: 0.20,
            schema_weight: 0.20,
            rule_weight: 0.20,
            anomaly_weight: 0.10,
        }
    }
}

/// Report from fidelity evaluation.
#[derive(Debug, Clone, Serialize)]
pub struct FidelityReport {
    /// Overall fidelity score (0.0-1.0).
    pub overall_score: f64,
    /// Statistical fidelity score.
    pub statistical_fidelity: f64,
    /// Correlation fidelity score.
    pub correlation_fidelity: f64,
    /// Schema fidelity score.
    pub schema_fidelity: f64,
    /// Rule compliance score.
    pub rule_compliance: f64,
    /// Anomaly fidelity score.
    pub anomaly_fidelity: f64,
    /// Whether overall score passes threshold.
    pub passes: bool,
    /// Detailed metrics.
    pub details: FidelityDetails,
}

/// Detailed fidelity metrics.
#[derive(Debug, Clone, Default, Serialize)]
pub struct FidelityDetails {
    /// Per-column statistical metrics.
    pub column_metrics: HashMap<String, ColumnFidelityMetrics>,
    /// KS statistics by column.
    pub ks_statistics: HashMap<String, f64>,
    /// Wasserstein distances by column.
    pub wasserstein_distances: HashMap<String, f64>,
    /// Jensen-Shannon divergences by column.
    pub js_divergences: HashMap<String, f64>,
    /// Benford's Law MAD.
    pub benford_mad: Option<f64>,
    /// Correlation matrix RMSE.
    pub correlation_rmse: Option<f64>,
    /// Row count ratio (synthetic / fingerprint).
    pub row_count_ratio: f64,
    /// Warnings.
    pub warnings: Vec<String>,
}

/// Fidelity metrics for a single column.
#[derive(Debug, Clone, Serialize)]
pub struct ColumnFidelityMetrics {
    /// Column name.
    pub name: String,
    /// KS statistic.
    pub ks_statistic: f64,
    /// Wasserstein distance.
    pub wasserstein_distance: f64,
    /// Jensen-Shannon divergence.
    pub js_divergence: f64,
    /// Mean difference (normalized).
    pub mean_diff: f64,
    /// Std dev difference (normalized).
    pub std_dev_diff: f64,
}

/// Evaluator for fidelity between synthetic data and fingerprints.
pub struct FidelityEvaluator {
    config: FidelityConfig,
}

impl FidelityEvaluator {
    /// Create a new evaluator with default configuration.
    pub fn new() -> Self {
        Self {
            config: FidelityConfig::default(),
        }
    }

    /// Create with a specific threshold.
    pub fn with_threshold(threshold: f64) -> Self {
        Self {
            config: FidelityConfig {
                threshold,
                ..Default::default()
            },
        }
    }

    /// Create with custom configuration.
    pub fn with_config(config: FidelityConfig) -> Self {
        Self { config }
    }

    /// Evaluate fidelity of synthetic data against a fingerprint.
    pub fn evaluate(
        &self,
        fingerprint: &Fingerprint,
        synthetic_data: &DataSource,
    ) -> FingerprintResult<FidelityReport> {
        // Extract fingerprint from synthetic data for comparison
        let extractor = FingerprintExtractor::new();
        let synthetic_fp = extractor.extract(synthetic_data)?;

        self.evaluate_fingerprints(fingerprint, &synthetic_fp)
    }

    /// Evaluate fidelity between two fingerprints.
    pub fn evaluate_fingerprints(
        &self,
        original: &Fingerprint,
        synthetic: &Fingerprint,
    ) -> FingerprintResult<FidelityReport> {
        let mut details = FidelityDetails::default();

        // Statistical fidelity
        let statistical_fidelity =
            self.evaluate_statistical(&original.statistics, &synthetic.statistics, &mut details);

        // Correlation fidelity
        let correlation_fidelity = self.evaluate_correlations(original, synthetic, &mut details);

        // Schema fidelity
        let schema_fidelity = self.evaluate_schema(original, synthetic, &mut details);

        // Rule compliance
        let rule_compliance = self.evaluate_rules(original, synthetic, &mut details);

        // Anomaly fidelity
        let anomaly_fidelity = self.evaluate_anomalies(original, synthetic, &mut details);

        // Calculate overall score
        let overall_score = self.config.statistical_weight * statistical_fidelity
            + self.config.correlation_weight * correlation_fidelity
            + self.config.schema_weight * schema_fidelity
            + self.config.rule_weight * rule_compliance
            + self.config.anomaly_weight * anomaly_fidelity;

        let passes = overall_score >= self.config.threshold;

        Ok(FidelityReport {
            overall_score,
            statistical_fidelity,
            correlation_fidelity,
            schema_fidelity,
            rule_compliance,
            anomaly_fidelity,
            passes,
            details,
        })
    }

    /// Evaluate statistical fidelity.
    fn evaluate_statistical(
        &self,
        original: &StatisticsFingerprint,
        synthetic: &StatisticsFingerprint,
        details: &mut FidelityDetails,
    ) -> f64 {
        let mut scores = Vec::new();

        // Compare numeric columns
        for (col_name, orig_stats) in &original.numeric_columns {
            if let Some(syn_stats) = synthetic.numeric_columns.get(col_name) {
                let metrics = self.compare_numeric_stats(col_name, orig_stats, syn_stats);

                // Compute column score (average of metrics)
                let col_score = 1.0
                    - (metrics.ks_statistic
                        + metrics.mean_diff.abs().min(1.0)
                        + metrics.std_dev_diff.abs().min(1.0))
                        / 3.0;

                scores.push(col_score.max(0.0));

                details.ks_statistics.insert(col_name.clone(), metrics.ks_statistic);
                details.column_metrics.insert(col_name.clone(), metrics);
            }
        }

        // Compare Benford's Law if available
        if let (Some(orig_benford), Some(syn_benford)) = (
            &original.benford_analysis,
            &synthetic.benford_analysis,
        ) {
            let benford_mad = compute_benford_mad(
                &orig_benford.observed_frequencies,
                &syn_benford.observed_frequencies,
            );
            details.benford_mad = Some(benford_mad);
            scores.push(1.0 - benford_mad.min(0.1) * 10.0); // Scale MAD to score
        }

        if scores.is_empty() {
            return 1.0; // No numeric columns to compare
        }

        scores.iter().sum::<f64>() / scores.len() as f64
    }

    /// Compare numeric statistics.
    fn compare_numeric_stats(
        &self,
        name: &str,
        original: &NumericStats,
        synthetic: &NumericStats,
    ) -> ColumnFidelityMetrics {
        // KS-like statistic from percentile comparison
        let ks_statistic = self.compute_percentile_ks(original, synthetic);

        // Normalized differences
        let mean_range = (original.max - original.min).max(1.0);
        let mean_diff = (original.mean - synthetic.mean) / mean_range;
        let std_dev_diff = if original.std_dev > 0.0 {
            (original.std_dev - synthetic.std_dev) / original.std_dev
        } else {
            0.0
        };

        // Placeholder for full metrics
        ColumnFidelityMetrics {
            name: name.to_string(),
            ks_statistic,
            wasserstein_distance: mean_diff.abs(), // Simplified
            js_divergence: 0.0, // Would require full distributions
            mean_diff,
            std_dev_diff,
        }
    }

    /// Compute KS-like statistic from percentiles.
    fn compute_percentile_ks(&self, original: &NumericStats, synthetic: &NumericStats) -> f64 {
        let orig_pcts = original.percentiles.to_array();
        let syn_pcts = synthetic.percentiles.to_array();

        let range = (original.max - original.min).max(1.0);

        orig_pcts
            .iter()
            .zip(syn_pcts.iter())
            .map(|(&o, &s)| ((o - s) / range).abs())
            .fold(0.0, f64::max)
    }

    /// Evaluate correlation fidelity.
    fn evaluate_correlations(
        &self,
        original: &Fingerprint,
        synthetic: &Fingerprint,
        details: &mut FidelityDetails,
    ) -> f64 {
        let (orig_corr, syn_corr) = match (&original.correlations, &synthetic.correlations) {
            (Some(o), Some(s)) => (o, s),
            _ => return 1.0, // No correlations to compare
        };

        let mut rmse_sum = 0.0;
        let mut count = 0;

        for (table_name, orig_matrix) in &orig_corr.matrices {
            if let Some(syn_matrix) = syn_corr.matrices.get(table_name) {
                // Compare correlation values
                for (i, &orig_val) in orig_matrix.correlations.iter().enumerate() {
                    if let Some(&syn_val) = syn_matrix.correlations.get(i) {
                        rmse_sum += (orig_val - syn_val).powi(2);
                        count += 1;
                    }
                }
            }
        }

        if count == 0 {
            return 1.0;
        }

        let rmse = (rmse_sum / count as f64).sqrt();
        details.correlation_rmse = Some(rmse);

        // Convert RMSE to score (RMSE of 0 = 1.0, RMSE of 1 = 0.0)
        1.0 - rmse.min(1.0)
    }

    /// Evaluate schema fidelity.
    fn evaluate_schema(
        &self,
        original: &Fingerprint,
        synthetic: &Fingerprint,
        details: &mut FidelityDetails,
    ) -> f64 {
        let mut score = 1.0;

        // Check table presence
        let orig_tables: std::collections::HashSet<_> = original.schema.tables.keys().collect();
        let syn_tables: std::collections::HashSet<_> = synthetic.schema.tables.keys().collect();

        if orig_tables != syn_tables {
            let missing = orig_tables.difference(&syn_tables).count();
            score -= 0.1 * missing as f64;
            details.warnings.push(format!("{} tables missing in synthetic data", missing));
        }

        // Check row count ratio
        let orig_rows: u64 = original.schema.tables.values().map(|t| t.row_count).sum();
        let syn_rows: u64 = synthetic.schema.tables.values().map(|t| t.row_count).sum();

        let ratio = if orig_rows > 0 {
            syn_rows as f64 / orig_rows as f64
        } else {
            1.0
        };
        details.row_count_ratio = ratio;

        // Penalize if ratio is too far from 1.0 (unless intentionally scaled)
        let ratio_penalty = (ratio - 1.0).abs().min(1.0) * 0.1;
        score -= ratio_penalty;

        // Check column match
        for (table_name, orig_table) in &original.schema.tables {
            if let Some(syn_table) = synthetic.schema.tables.get(table_name) {
                let orig_cols: std::collections::HashSet<_> =
                    orig_table.columns.iter().map(|c| &c.name).collect();
                let syn_cols: std::collections::HashSet<_> =
                    syn_table.columns.iter().map(|c| &c.name).collect();

                if orig_cols != syn_cols {
                    let missing = orig_cols.difference(&syn_cols).count();
                    score -= 0.05 * missing as f64;
                }
            }
        }

        score.max(0.0)
    }

    /// Evaluate rule compliance.
    fn evaluate_rules(
        &self,
        original: &Fingerprint,
        synthetic: &Fingerprint,
        _details: &mut FidelityDetails,
    ) -> f64 {
        // Compare rule compliance rates if available
        let (orig_rules, syn_rules) = match (&original.rules, &synthetic.rules) {
            (Some(o), Some(s)) => (o, s),
            _ => return 1.0,
        };

        let mut score = 1.0;

        // Compare balance rule compliance
        for orig_rule in &orig_rules.balance_rules {
            if let Some(syn_rule) = syn_rules.balance_rules.iter().find(|r| r.name == orig_rule.name) {
                let diff = (orig_rule.compliance_rate - syn_rule.compliance_rate).abs();
                score -= diff * 0.1;
            }
        }

        score.max(0.0)
    }

    /// Evaluate anomaly fidelity.
    fn evaluate_anomalies(
        &self,
        original: &Fingerprint,
        synthetic: &Fingerprint,
        _details: &mut FidelityDetails,
    ) -> f64 {
        let (orig_anomalies, syn_anomalies) = match (&original.anomalies, &synthetic.anomalies) {
            (Some(o), Some(s)) => (o, s),
            _ => return 1.0,
        };

        // Compare overall anomaly rates
        let rate_diff = (orig_anomalies.overall.anomaly_rate - syn_anomalies.overall.anomaly_rate).abs();

        // Convert to score (0.1 rate difference = 0.0 score)
        1.0 - (rate_diff * 10.0).min(1.0)
    }
}

impl Default for FidelityEvaluator {
    fn default() -> Self {
        Self::new()
    }
}

/// Compute Benford's Law MAD between two distributions.
fn compute_benford_mad(original: &[f64; 9], synthetic: &[f64; 9]) -> f64 {
    let sum: f64 = original
        .iter()
        .zip(synthetic.iter())
        .map(|(&o, &s)| (o - s).abs())
        .sum();
    sum / 9.0
}

/// Generate an HTML report from fidelity results.
pub fn generate_html_report(report: &FidelityReport) -> String {
    let status_class = if report.passes { "pass" } else { "fail" };
    let status_text = if report.passes { "PASS" } else { "FAIL" };

    format!(
        r#"<!DOCTYPE html>
<html>
<head>
    <title>Fidelity Report</title>
    <style>
        body {{ font-family: Arial, sans-serif; margin: 40px; }}
        .pass {{ color: green; }}
        .fail {{ color: red; }}
        .metric {{ margin: 10px 0; }}
        .score {{ font-weight: bold; }}
        table {{ border-collapse: collapse; width: 100%; }}
        th, td {{ border: 1px solid #ddd; padding: 8px; text-align: left; }}
        th {{ background-color: #4CAF50; color: white; }}
    </style>
</head>
<body>
    <h1>Fidelity Evaluation Report</h1>

    <div class="metric">
        <h2>Overall Score: <span class="score {}">{:.1}%</span></h2>
        <p>Status: <span class="{}">{}</span></p>
    </div>

    <h2>Component Scores</h2>
    <table>
        <tr><th>Component</th><th>Score</th></tr>
        <tr><td>Statistical Fidelity</td><td>{:.1}%</td></tr>
        <tr><td>Correlation Fidelity</td><td>{:.1}%</td></tr>
        <tr><td>Schema Fidelity</td><td>{:.1}%</td></tr>
        <tr><td>Rule Compliance</td><td>{:.1}%</td></tr>
        <tr><td>Anomaly Fidelity</td><td>{:.1}%</td></tr>
    </table>

    <h2>Details</h2>
    <p>Row count ratio: {:.2}</p>
    {}
    {}
</body>
</html>"#,
        status_class,
        report.overall_score * 100.0,
        status_class,
        status_text,
        report.statistical_fidelity * 100.0,
        report.correlation_fidelity * 100.0,
        report.schema_fidelity * 100.0,
        report.rule_compliance * 100.0,
        report.anomaly_fidelity * 100.0,
        report.details.row_count_ratio,
        report.details.benford_mad.map(|m| format!("<p>Benford MAD: {:.4}</p>", m)).unwrap_or_default(),
        report.details.correlation_rmse.map(|r| format!("<p>Correlation RMSE: {:.4}</p>", r)).unwrap_or_default(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_benford_mad() {
        let original = [0.301, 0.176, 0.125, 0.097, 0.079, 0.067, 0.058, 0.051, 0.046];
        let synthetic = [0.301, 0.176, 0.125, 0.097, 0.079, 0.067, 0.058, 0.051, 0.046];

        let mad = compute_benford_mad(&original, &synthetic);
        assert!(mad < 0.001); // Identical distributions
    }
}
