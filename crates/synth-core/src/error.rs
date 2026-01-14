//! Error types for the synthetic data generation system.

use thiserror::Error;

/// Main error type for synthetic data operations.
#[derive(Error, Debug)]
pub enum SynthError {
    /// Configuration error
    #[error("Configuration error: {0}")]
    ConfigError(String),

    /// Validation error
    #[error("Validation error: {0}")]
    ValidationError(String),

    /// Generation error
    #[error("Generation error: {0}")]
    GenerationError(String),

    /// IO error
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    /// Serialization error
    #[error("Serialization error: {0}")]
    SerializationError(String),

    /// Invalid data error
    #[error("Invalid data: {0}")]
    InvalidData(String),

    /// Resource exhausted
    #[error("Resource exhausted: {0}")]
    ResourceExhausted(String),

    /// Channel closed (for streaming)
    #[error("Channel closed unexpectedly")]
    ChannelClosed,

    /// Not supported
    #[error("Operation not supported: {0}")]
    NotSupported(String),
}

impl SynthError {
    /// Create a configuration error.
    pub fn config(msg: impl Into<String>) -> Self {
        Self::ConfigError(msg.into())
    }

    /// Create a validation error.
    pub fn validation(msg: impl Into<String>) -> Self {
        Self::ValidationError(msg.into())
    }

    /// Create a generation error.
    pub fn generation(msg: impl Into<String>) -> Self {
        Self::GenerationError(msg.into())
    }

    /// Create an invalid data error.
    pub fn invalid_data(msg: impl Into<String>) -> Self {
        Self::InvalidData(msg.into())
    }
}

/// Result type alias for synthetic data operations.
pub type SynthResult<T> = Result<T, SynthError>;
