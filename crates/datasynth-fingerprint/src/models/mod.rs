//! Fingerprint data models.
//!
//! This module contains all the data structures that make up a fingerprint,
//! including schema information, statistical distributions, correlations,
//! integrity constraints, business rules, anomaly patterns, and privacy audit.
//!
//! # Fingerprint Structure
//!
//! A [`Fingerprint`] is composed of the following components:
//!
//! | Component | Type | Required | Description |
//! |-----------|------|----------|-------------|
//! | `manifest` | [`Manifest`] | Yes | Version, source metadata, privacy config |
//! | `schema` | [`SchemaFingerprint`] | Yes | Table and column definitions |
//! | `statistics` | [`StatisticsFingerprint`] | Yes | Distribution parameters |
//! | `correlations` | [`CorrelationFingerprint`] | No | Correlation matrices, copulas |
//! | `integrity` | [`IntegrityFingerprint`] | No | Foreign keys, cardinalities |
//! | `rules` | [`RulesFingerprint`] | No | Business rules, balance equations |
//! | `anomalies` | [`AnomalyFingerprint`] | No | Anomaly patterns, rates |
//! | `privacy_audit` | [`PrivacyAudit`] | Yes | Privacy decisions, epsilon tracking |
//!
//! # Privacy Levels
//!
//! The [`PrivacyLevel`] enum controls how much privacy protection is applied:
//!
//! ```ignore
//! use datasynth_fingerprint::models::PrivacyLevel;
//!
//! // Standard is the recommended default
//! let level = PrivacyLevel::Standard;
//!
//! // Get the differential privacy epsilon
//! assert_eq!(level.epsilon(), 1.0);
//!
//! // Get the k-anonymity threshold
//! assert_eq!(level.k_anonymity(), 5);
//! ```
//!
//! # Statistics Types
//!
//! Different column types have specialized statistics:
//!
//! - [`NumericStats`] - For numeric columns (mean, std_dev, percentiles, distribution)
//! - [`CategoricalStats`] - For categorical columns (frequencies, cardinality)
//! - [`TemporalStats`] - For date/time columns (range, seasonality patterns)
//!
//! # Distribution Fitting
//!
//! Numeric columns are fit to parametric distributions when possible:
//!
//! - [`DistributionType::LogNormal`] - Common for financial amounts
//! - [`DistributionType::Normal`] - Gaussian distribution
//! - [`DistributionType::Gamma`] - Positive skewed data
//! - [`DistributionType::Exponential`] - Time between events
//! - [`DistributionType::Empirical`] - Fallback histogram representation

mod anomaly;
mod correlation;
mod fingerprint;
mod integrity;
mod manifest;
mod privacy_audit;
mod rules;
mod schema;
mod statistics;

pub use anomaly::*;
pub use correlation::*;
pub use fingerprint::*;
pub use integrity::*;
pub use manifest::*;
pub use privacy_audit::*;
pub use rules::*;
pub use schema::*;
pub use statistics::*;
