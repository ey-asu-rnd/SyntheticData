//! Report generation module.
//!
//! Generates evaluation reports in various formats (JSON, HTML) with
//! pass/fail criteria and baseline comparison.

mod comparison;
mod html;
mod json;
mod thresholds;

pub use comparison::{BaselineComparison, ComparisonResult, MetricChange};
pub use html::HtmlReportGenerator;
pub use json::JsonReportGenerator;
pub use thresholds::{ThresholdChecker, ThresholdResult};

use crate::coherence::CoherenceEvaluation;
use crate::ml::MLReadinessEvaluation;
use crate::quality::QualityEvaluation;
use crate::statistical::StatisticalEvaluation;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Complete evaluation report.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvaluationReport {
    /// Report metadata.
    pub metadata: ReportMetadata,
    /// Statistical evaluation results.
    pub statistical: Option<StatisticalEvaluation>,
    /// Coherence evaluation results.
    pub coherence: Option<CoherenceEvaluation>,
    /// Quality evaluation results.
    pub quality: Option<QualityEvaluation>,
    /// ML-readiness evaluation results.
    pub ml_readiness: Option<MLReadinessEvaluation>,
    /// Overall pass/fail status.
    pub passes: bool,
    /// Summary of all issues found.
    pub all_issues: Vec<ReportIssue>,
    /// Overall score (0.0-1.0).
    pub overall_score: f64,
    /// Comparison with baseline (if provided).
    pub baseline_comparison: Option<BaselineComparison>,
}

/// Report metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportMetadata {
    /// Report generation timestamp.
    pub generated_at: DateTime<Utc>,
    /// Evaluation version.
    pub version: String,
    /// Input data source.
    pub data_source: String,
    /// Thresholds used.
    pub thresholds_name: String,
    /// Number of records evaluated.
    pub records_evaluated: usize,
    /// Evaluation duration in milliseconds.
    pub duration_ms: u64,
}

/// An issue found during evaluation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportIssue {
    /// Issue category.
    pub category: IssueCategory,
    /// Issue severity.
    pub severity: IssueSeverity,
    /// Issue description.
    pub description: String,
    /// Metric name (if applicable).
    pub metric: Option<String>,
    /// Actual value (if applicable).
    pub actual_value: Option<String>,
    /// Threshold value (if applicable).
    pub threshold_value: Option<String>,
}

/// Category of issue.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum IssueCategory {
    Statistical,
    Coherence,
    Quality,
    MLReadiness,
}

/// Severity of issue.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum IssueSeverity {
    /// Critical issue that fails the evaluation.
    Critical,
    /// Warning that may need attention.
    Warning,
    /// Informational note.
    Info,
}

impl EvaluationReport {
    /// Create a new report with the given results.
    pub fn new(
        metadata: ReportMetadata,
        statistical: Option<StatisticalEvaluation>,
        coherence: Option<CoherenceEvaluation>,
        quality: Option<QualityEvaluation>,
        ml_readiness: Option<MLReadinessEvaluation>,
    ) -> Self {
        let mut report = Self {
            metadata,
            statistical,
            coherence,
            quality,
            ml_readiness,
            passes: true,
            all_issues: Vec::new(),
            overall_score: 1.0,
            baseline_comparison: None,
        };
        report.aggregate_results();
        report
    }

    /// Aggregate results from all evaluations.
    fn aggregate_results(&mut self) {
        let mut scores = Vec::new();

        // Collect statistical issues
        if let Some(ref stat) = self.statistical {
            if !stat.passes {
                self.passes = false;
            }
            scores.push(stat.overall_score);
            for issue in &stat.issues {
                self.all_issues.push(ReportIssue {
                    category: IssueCategory::Statistical,
                    severity: IssueSeverity::Critical,
                    description: issue.clone(),
                    metric: None,
                    actual_value: None,
                    threshold_value: None,
                });
            }
        }

        // Collect coherence issues
        if let Some(ref coh) = self.coherence {
            if !coh.passes {
                self.passes = false;
            }
            for failure in &coh.failures {
                self.all_issues.push(ReportIssue {
                    category: IssueCategory::Coherence,
                    severity: IssueSeverity::Critical,
                    description: failure.clone(),
                    metric: None,
                    actual_value: None,
                    threshold_value: None,
                });
            }
        }

        // Collect quality issues
        if let Some(ref qual) = self.quality {
            if !qual.passes {
                self.passes = false;
            }
            scores.push(qual.overall_score);
            for issue in &qual.issues {
                self.all_issues.push(ReportIssue {
                    category: IssueCategory::Quality,
                    severity: IssueSeverity::Critical,
                    description: issue.clone(),
                    metric: None,
                    actual_value: None,
                    threshold_value: None,
                });
            }
        }

        // Collect ML issues
        if let Some(ref ml) = self.ml_readiness {
            if !ml.passes {
                self.passes = false;
            }
            scores.push(ml.overall_score);
            for issue in &ml.issues {
                self.all_issues.push(ReportIssue {
                    category: IssueCategory::MLReadiness,
                    severity: IssueSeverity::Critical,
                    description: issue.clone(),
                    metric: None,
                    actual_value: None,
                    threshold_value: None,
                });
            }
        }

        // Calculate overall score
        self.overall_score = if scores.is_empty() {
            1.0
        } else {
            scores.iter().sum::<f64>() / scores.len() as f64
        };
    }

    /// Set baseline comparison.
    pub fn with_baseline_comparison(mut self, comparison: BaselineComparison) -> Self {
        self.baseline_comparison = Some(comparison);
        self
    }

    /// Get issues by category.
    pub fn issues_by_category(&self, category: IssueCategory) -> Vec<&ReportIssue> {
        self.all_issues
            .iter()
            .filter(|i| i.category == category)
            .collect()
    }

    /// Get critical issues only.
    pub fn critical_issues(&self) -> Vec<&ReportIssue> {
        self.all_issues
            .iter()
            .filter(|i| i.severity == IssueSeverity::Critical)
            .collect()
    }
}

/// Report format options.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReportFormat {
    /// JSON format.
    Json,
    /// HTML format.
    Html,
    /// Both JSON and HTML.
    Both,
}

/// Report generator trait.
pub trait ReportGenerator {
    /// Generate report to string.
    fn generate(&self, report: &EvaluationReport) -> crate::error::EvalResult<String>;

    /// Generate report to file.
    fn generate_to_file(
        &self,
        report: &EvaluationReport,
        path: &std::path::Path,
    ) -> crate::error::EvalResult<()> {
        let content = self.generate(report)?;
        std::fs::write(path, content)?;
        Ok(())
    }
}
