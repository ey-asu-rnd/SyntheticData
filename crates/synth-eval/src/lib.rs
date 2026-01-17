//! Synthetic Data Evaluation Framework
//!
//! This crate provides comprehensive evaluation capabilities for validating
//! the quality and correctness of generated synthetic financial data.
//!
//! # Features
//!
//! - **Statistical Quality**: Benford's Law, amount distributions, line item patterns
//! - **Semantic Coherence**: Balance sheet validation, subledger reconciliation
//! - **Data Quality**: Uniqueness, completeness, format consistency
//! - **ML-Readiness**: Feature distributions, label quality, graph structure
//! - **Reporting**: HTML and JSON reports with pass/fail thresholds
//!
//! # Example
//!
//! ```ignore
//! use synth_eval::{Evaluator, EvaluationConfig};
//!
//! let config = EvaluationConfig::default();
//! let evaluator = Evaluator::new(config);
//!
//! // Evaluate generated data
//! let result = evaluator.evaluate(&generation_result)?;
//!
//! // Generate report
//! result.generate_html_report("evaluation_report.html")?;
//! ```

pub mod config;
pub mod error;

pub mod statistical;
pub mod coherence;
pub mod quality;
pub mod ml;
pub mod report;
pub mod tuning;

// Re-exports
pub use config::{EvaluationConfig, EvaluationThresholds};
pub use error::{EvalError, EvalResult};

pub use statistical::{
    BenfordAnalysis, BenfordAnalyzer, BenfordConformity,
    AmountDistributionAnalysis, AmountDistributionAnalyzer,
    LineItemAnalysis, LineItemAnalyzer, LineItemEntry,
    TemporalAnalysis, TemporalAnalyzer, TemporalEntry,
    StatisticalEvaluation,
};

pub use coherence::{
    BalanceSheetEvaluation, BalanceSheetEvaluator,
    SubledgerReconciliationEvaluation, SubledgerEvaluator,
    DocumentChainEvaluation, DocumentChainEvaluator,
    ICMatchingEvaluation, ICMatchingEvaluator,
    ReferentialIntegrityEvaluation, ReferentialIntegrityEvaluator,
    CoherenceEvaluation,
};

pub use quality::{
    UniquenessAnalysis, UniquenessAnalyzer, DuplicateInfo,
    CompletenessAnalysis, CompletenessAnalyzer, FieldCompleteness,
    FormatAnalysis, FormatAnalyzer, FormatVariation,
    ConsistencyAnalysis, ConsistencyAnalyzer, ConsistencyRule,
    QualityEvaluation,
};

pub use ml::{
    FeatureAnalysis, FeatureAnalyzer, FeatureStats,
    LabelAnalysis, LabelAnalyzer, LabelDistribution,
    SplitAnalysis, SplitAnalyzer, SplitMetrics,
    GraphAnalysis, GraphAnalyzer, GraphMetrics,
    MLReadinessEvaluation,
};

pub use report::{
    HtmlReportGenerator, JsonReportGenerator,
    EvaluationReport, ReportMetadata,
    BaselineComparison, ComparisonResult, MetricChange,
    ThresholdChecker, ThresholdResult,
};

pub use tuning::{
    TuningOpportunity, TuningCategory, TuningAnalyzer,
    ConfigSuggestion, ConfigSuggestionGenerator,
};

use serde::{Deserialize, Serialize};

/// Comprehensive evaluation result combining all evaluation modules.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComprehensiveEvaluation {
    /// Statistical quality evaluation.
    pub statistical: StatisticalEvaluation,
    /// Semantic coherence evaluation.
    pub coherence: CoherenceEvaluation,
    /// Data quality evaluation.
    pub quality: QualityEvaluation,
    /// ML-readiness evaluation.
    pub ml_readiness: MLReadinessEvaluation,
    /// Overall pass/fail status.
    pub passes: bool,
    /// Summary of all failures.
    pub failures: Vec<String>,
    /// Tuning opportunities identified.
    pub tuning_opportunities: Vec<TuningOpportunity>,
    /// Configuration suggestions.
    pub config_suggestions: Vec<ConfigSuggestion>,
}

impl ComprehensiveEvaluation {
    /// Create a new empty evaluation.
    pub fn new() -> Self {
        Self {
            statistical: StatisticalEvaluation::default(),
            coherence: CoherenceEvaluation::default(),
            quality: QualityEvaluation::default(),
            ml_readiness: MLReadinessEvaluation::default(),
            passes: true,
            failures: Vec::new(),
            tuning_opportunities: Vec::new(),
            config_suggestions: Vec::new(),
        }
    }

    /// Check all evaluations against thresholds and update overall status.
    pub fn check_all_thresholds(&mut self, thresholds: &EvaluationThresholds) {
        self.failures.clear();

        // Check statistical thresholds
        self.statistical.check_thresholds(thresholds);
        self.failures.extend(self.statistical.failures.clone());

        // Check coherence thresholds
        self.coherence.check_thresholds(thresholds);
        self.failures.extend(self.coherence.failures.clone());

        // Check quality thresholds
        self.quality.check_thresholds(thresholds);
        self.failures.extend(self.quality.failures.clone());

        // Check ML thresholds
        self.ml_readiness.check_thresholds(thresholds);
        self.failures.extend(self.ml_readiness.failures.clone());

        self.passes = self.failures.is_empty();
    }
}

impl Default for ComprehensiveEvaluation {
    fn default() -> Self {
        Self::new()
    }
}

/// Main evaluator that coordinates all evaluation modules.
pub struct Evaluator {
    /// Evaluation configuration.
    config: EvaluationConfig,
}

impl Evaluator {
    /// Create a new evaluator with the given configuration.
    pub fn new(config: EvaluationConfig) -> Self {
        Self { config }
    }

    /// Create an evaluator with default configuration.
    pub fn with_defaults() -> Self {
        Self::new(EvaluationConfig::default())
    }

    /// Get the configuration.
    pub fn config(&self) -> &EvaluationConfig {
        &self.config
    }

    /// Run a comprehensive evaluation and return results.
    ///
    /// This is a placeholder - actual implementation would take
    /// generation results as input.
    pub fn run_evaluation(&self) -> ComprehensiveEvaluation {
        let mut evaluation = ComprehensiveEvaluation::new();
        evaluation.check_all_thresholds(&self.config.thresholds);
        evaluation
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_comprehensive_evaluation_new() {
        let eval = ComprehensiveEvaluation::new();
        assert!(eval.passes);
        assert!(eval.failures.is_empty());
    }

    #[test]
    fn test_evaluator_creation() {
        let evaluator = Evaluator::with_defaults();
        assert_eq!(evaluator.config().thresholds.benford_p_value_min, 0.05);
    }
}
