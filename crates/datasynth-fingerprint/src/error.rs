//! Error types for the fingerprint crate.

use thiserror::Error;

/// Result type for fingerprint operations.
pub type FingerprintResult<T> = Result<T, FingerprintError>;

/// Errors that can occur during fingerprint operations.
#[derive(Debug, Error)]
pub enum FingerprintError {
    /// I/O error during file operations.
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// Error during ZIP archive operations.
    #[error("ZIP archive error: {0}")]
    Zip(#[from] zip::result::ZipError),

    /// Error during JSON serialization/deserialization.
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    /// Error during YAML serialization/deserialization.
    #[error("YAML error: {0}")]
    Yaml(#[from] serde_yaml::Error),

    /// Error during CSV parsing.
    #[error("CSV error: {0}")]
    Csv(#[from] csv::Error),

    /// Invalid fingerprint format.
    #[error("Invalid fingerprint format: {0}")]
    InvalidFormat(String),

    /// Missing required component in fingerprint.
    #[error("Missing required component: {0}")]
    MissingComponent(String),

    /// Checksum mismatch.
    #[error("Checksum mismatch for {file}: expected {expected}, got {actual}")]
    ChecksumMismatch {
        file: String,
        expected: String,
        actual: String,
    },

    /// Version mismatch.
    #[error("Unsupported fingerprint version: {0}")]
    UnsupportedVersion(String),

    /// Privacy budget exhausted.
    #[error("Privacy budget exhausted: epsilon={spent}, limit={limit}")]
    PrivacyBudgetExhausted { spent: f64, limit: f64 },

    /// Insufficient data for extraction.
    #[error("Insufficient data: need at least {required} rows, got {actual}")]
    InsufficientData { required: usize, actual: usize },

    /// Statistical computation error.
    #[error("Statistical error: {0}")]
    StatisticalError(String),

    /// Data validation error.
    #[error("Validation error: {0}")]
    ValidationError(String),

    /// Configuration synthesis error.
    #[error("Config synthesis error: {0}")]
    SynthesisError(String),

    /// Extraction error.
    #[error("Extraction error in {extractor}: {message}")]
    ExtractionError { extractor: String, message: String },

    /// Privacy constraint violation.
    #[error("Privacy constraint violated: {0}")]
    PrivacyViolation(String),

    /// Matrix operation error.
    #[error("Matrix operation error: {0}")]
    MatrixError(String),

    /// Distribution fitting error.
    #[error("Distribution fitting error: {0}")]
    DistributionFitError(String),
}

impl FingerprintError {
    /// Create an extraction error.
    pub fn extraction(extractor: &str, message: impl Into<String>) -> Self {
        Self::ExtractionError {
            extractor: extractor.to_string(),
            message: message.into(),
        }
    }
}
