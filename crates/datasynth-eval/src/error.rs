//! Error types for the evaluation framework.

use thiserror::Error;

/// Errors that can occur during evaluation.
#[derive(Debug, Error)]
pub enum EvalError {
    /// Insufficient data for statistical analysis.
    #[error("Insufficient data: need at least {required} samples, got {actual}")]
    InsufficientData { required: usize, actual: usize },

    /// Invalid parameter value.
    #[error("Invalid parameter: {0}")]
    InvalidParameter(String),

    /// Statistical computation error.
    #[error("Statistical computation error: {0}")]
    StatisticalError(String),

    /// IO error during report generation.
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    /// Serialization error.
    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),

    /// Missing required data.
    #[error("Missing required data: {0}")]
    MissingData(String),

    /// Evaluation threshold exceeded.
    #[error("Threshold exceeded for {metric}: {value} (threshold: {threshold})")]
    ThresholdExceeded {
        metric: String,
        value: f64,
        threshold: f64,
    },
}

/// Result type for evaluation operations.
pub type EvalResult<T> = Result<T, EvalError>;
