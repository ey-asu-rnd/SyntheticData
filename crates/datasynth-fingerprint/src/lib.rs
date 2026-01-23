//! DataSynth Fingerprint - Privacy-preserving synthetic data fingerprinting.
//!
//! This crate provides functionality for:
//! - **Extracting** statistical fingerprints from real data
//! - **Applying privacy** mechanisms (differential privacy, k-anonymity)
//! - **Storing** fingerprints in `.dsf` files
//! - **Synthesizing** generator configurations from fingerprints
//! - **Evaluating** fidelity of generated data
//!
//! # Overview
//!
//! A fingerprint captures the statistical properties of a dataset without storing
//! any individual records, enabling privacy-preserving synthetic data generation.
//!
//! ```text
//! Real Data → Extract → .dsf File → Generate → Synthetic Data → Evaluate
//! ```
//!
//! # Quick Start
//!
//! ```ignore
//! use datasynth_fingerprint::{
//!     extraction::FingerprintExtractor,
//!     io::{FingerprintReader, FingerprintWriter},
//!     models::PrivacyLevel,
//! };
//!
//! // Extract fingerprint from data
//! let extractor = FingerprintExtractor::new(PrivacyLevel::Standard);
//! let fingerprint = extractor.extract_from_csv("data.csv")?;
//!
//! // Write to .dsf file
//! let writer = FingerprintWriter::new();
//! writer.write_to_file(&fingerprint, "output.dsf")?;
//!
//! // Read from .dsf file
//! let reader = FingerprintReader::new();
//! let loaded = reader.read_from_file("output.dsf")?;
//! ```
//!
//! # DSF File Format
//!
//! A `.dsf` (DataSynth Fingerprint) file is a ZIP archive containing:
//!
//! - `manifest.json` - Version, checksums, privacy config
//! - `schema.yaml` - Tables, columns, types, relationships
//! - `statistics.yaml` - Distributions, percentiles, Benford analysis
//! - `correlations.yaml` - Correlation matrices, copulas (optional)
//! - `integrity.yaml` - FK relationships, cardinality (optional)
//! - `rules.yaml` - Balance constraints, approval thresholds (optional)
//! - `anomalies.yaml` - Anomaly rates, type distribution (optional)
//! - `privacy_audit.json` - Privacy decisions, epsilon spent
//!
//! # Privacy
//!
//! The fingerprinting process applies multiple privacy mechanisms:
//!
//! - **Differential Privacy**: Laplace noise with configurable epsilon
//! - **K-Anonymity**: Suppression of rare categorical values
//! - **Outlier Handling**: Winsorization at configurable percentiles
//! - **Audit Trail**: Complete log of all privacy decisions
//!
//! # Modules
//!
//! - [`models`] - Data structures for fingerprints
//! - [`io`] - Reading and writing `.dsf` files
//! - [`privacy`] - Privacy mechanisms
//! - [`extraction`] - Extractors for schema, statistics, etc.
//! - [`synthesis`] - Convert fingerprints to generator configs
//! - [`evaluation`] - Fidelity evaluation

pub mod error;
pub mod evaluation;
pub mod extraction;
pub mod io;
pub mod models;
pub mod privacy;
pub mod synthesis;

// Re-export commonly used types
pub use error::{FingerprintError, FingerprintResult};
pub use io::{FingerprintReader, FingerprintWriter, FingerprintValidator};
pub use models::{Fingerprint, Manifest, PrivacyLevel, PrivacyMetadata, SchemaFingerprint};
