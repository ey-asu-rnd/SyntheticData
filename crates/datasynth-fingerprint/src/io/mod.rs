//! I/O operations for fingerprint files.
//!
//! This module handles reading and writing `.dsf` (DataSynth Fingerprint) files,
//! which are ZIP archives containing YAML/JSON component files.
//!
//! # Overview
//!
//! The `.dsf` format is a portable, self-contained format for storing fingerprints.
//! Files can optionally be signed with HMAC-SHA256 for authenticity verification.
//!
//! # Writing Fingerprints
//!
//! ```ignore
//! use datasynth_fingerprint::io::FingerprintWriter;
//! use std::path::Path;
//!
//! let writer = FingerprintWriter::new();
//! writer.write_to_file(&fingerprint, Path::new("output.dsf"))?;
//! ```
//!
//! # Reading Fingerprints
//!
//! ```ignore
//! use datasynth_fingerprint::io::FingerprintReader;
//! use std::path::Path;
//!
//! let reader = FingerprintReader::new();
//! let fingerprint = reader.read_from_file(Path::new("fingerprint.dsf"))?;
//! ```
//!
//! # Digital Signatures
//!
//! Fingerprints can be signed to ensure authenticity and integrity:
//!
//! ```ignore
//! use datasynth_fingerprint::io::{SigningKey, DsfSigner, DsfVerifier};
//!
//! // Create a signing key (store securely!)
//! let key = SigningKey::generate("my-key-id");
//!
//! // Sign when writing
//! let signer = DsfSigner::new(key.clone());
//! writer.write_to_file_signed(&fingerprint, Path::new("signed.dsf"), &signer)?;
//!
//! // Verify when reading
//! let verifier = DsfVerifier::new(key);
//! let fp = reader.read_from_file_verified(Path::new("signed.dsf"), &verifier)?;
//!
//! // Check if a file is signed without reading it fully
//! let is_signed = reader.is_signed(Path::new("signed.dsf"))?;
//! ```
//!
//! # Validation
//!
//! The [`FingerprintValidator`] can check `.dsf` file integrity:
//!
//! ```ignore
//! use datasynth_fingerprint::io::FingerprintValidator;
//!
//! let validator = FingerprintValidator::new();
//! let result = validator.validate(Path::new("fingerprint.dsf"))?;
//!
//! if result.is_valid {
//!     println!("File is valid");
//! } else {
//!     for error in &result.errors {
//!         eprintln!("Error: {}", error);
//!     }
//! }
//! ```
//!
//! # File Structure
//!
//! A `.dsf` file is a ZIP archive containing:
//!
//! | File | Format | Required | Description |
//! |------|--------|----------|-------------|
//! | `manifest.json` | JSON | Yes | Version, checksums, signature |
//! | `schema.yaml` | YAML | Yes | Table and column definitions |
//! | `statistics.yaml` | YAML | Yes | Distribution parameters |
//! | `correlations.yaml` | YAML | No | Correlation matrices |
//! | `integrity.yaml` | YAML | No | FK relationships |
//! | `rules.yaml` | YAML | No | Business rules |
//! | `anomalies.yaml` | YAML | No | Anomaly profiles |
//! | `privacy_audit.json` | JSON | Yes | Privacy audit trail |
//!
//! [`FingerprintValidator`]: validation::FingerprintValidator

mod reader;
pub mod signing;
mod validation;
mod writer;

pub use reader::*;
pub use signing::{
    canonical_manifest_json, DsfSigner, DsfVerifier, SigningKey, ALGORITHM_HMAC_SHA256,
};
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
