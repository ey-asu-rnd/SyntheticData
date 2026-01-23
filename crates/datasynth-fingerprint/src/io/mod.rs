//! I/O operations for fingerprint files.
//!
//! This module handles reading and writing .dsf (DataSynth Fingerprint) files,
//! which are ZIP archives containing YAML/JSON component files.

mod reader;
mod validation;
mod writer;

pub use reader::*;
pub use validation::*;
pub use writer::*;

/// File names within a .dsf archive.
pub mod file_names {
    /// Manifest file name.
    pub const MANIFEST: &str = "manifest.json";
    /// Schema file name.
    pub const SCHEMA: &str = "schema.yaml";
    /// Statistics file name.
    pub const STATISTICS: &str = "statistics.yaml";
    /// Correlations file name.
    pub const CORRELATIONS: &str = "correlations.yaml";
    /// Integrity file name.
    pub const INTEGRITY: &str = "integrity.yaml";
    /// Rules file name.
    pub const RULES: &str = "rules.yaml";
    /// Anomalies file name.
    pub const ANOMALIES: &str = "anomalies.yaml";
    /// Privacy audit file name.
    pub const PRIVACY_AUDIT: &str = "privacy_audit.json";
    /// Signature file name (optional).
    pub const SIGNATURE: &str = "signature.sig";
}
