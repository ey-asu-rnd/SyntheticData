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

    /// Memory limit exceeded
    #[error("Memory limit exceeded: {current_mb} MB used, limit is {limit_mb} MB")]
    MemoryExhausted { current_mb: usize, limit_mb: usize },

    /// Disk space exhausted
    #[error("Disk space exhausted: {available_mb} MB available, need {required_mb} MB")]
    DiskSpaceExhausted {
        available_mb: usize,
        required_mb: usize,
    },

    /// CPU overload
    #[error("CPU overloaded: load {load:.1}% exceeds threshold {threshold:.1}%")]
    CpuOverloaded { load: f64, threshold: f64 },

    /// Resource degradation triggered
    #[error("Resource degradation triggered: {level} - {reason}")]
    DegradationTriggered { level: String, reason: String },

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

    /// Create a resource exhausted error.
    pub fn resource(msg: impl Into<String>) -> Self {
        Self::ResourceExhausted(msg.into())
    }

    /// Create a not supported error.
    pub fn not_supported(msg: impl Into<String>) -> Self {
        Self::NotSupported(msg.into())
    }

    /// Create a memory exhausted error.
    pub fn memory_exhausted(current_mb: usize, limit_mb: usize) -> Self {
        Self::MemoryExhausted {
            current_mb,
            limit_mb,
        }
    }

    /// Create a disk space exhausted error.
    pub fn disk_exhausted(available_mb: usize, required_mb: usize) -> Self {
        Self::DiskSpaceExhausted {
            available_mb,
            required_mb,
        }
    }

    /// Create a CPU overloaded error.
    pub fn cpu_overloaded(load: f64, threshold: f64) -> Self {
        Self::CpuOverloaded { load, threshold }
    }

    /// Create a degradation triggered error.
    pub fn degradation(level: impl Into<String>, reason: impl Into<String>) -> Self {
        Self::DegradationTriggered {
            level: level.into(),
            reason: reason.into(),
        }
    }

    /// Check if this error is recoverable (degradation or soft limits).
    pub fn is_recoverable(&self) -> bool {
        matches!(self, Self::DegradationTriggered { .. })
    }

    /// Check if this error is a resource-related error.
    pub fn is_resource_error(&self) -> bool {
        matches!(
            self,
            Self::ResourceExhausted(_)
                | Self::MemoryExhausted { .. }
                | Self::DiskSpaceExhausted { .. }
                | Self::CpuOverloaded { .. }
                | Self::DegradationTriggered { .. }
        )
    }
}

/// Result type alias for synthetic data operations.
pub type SynthResult<T> = Result<T, SynthError>;
