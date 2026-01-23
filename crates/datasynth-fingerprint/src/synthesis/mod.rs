//! Config synthesis from fingerprints.
//!
//! This module converts fingerprints into generator configurations
//! that can be used to generate matching synthetic data.
//!
//! # Overview
//!
//! The synthesis process takes a [`Fingerprint`] and produces:
//!
//! 1. A [`ConfigPatch`] with configuration values for DataSynth generators
//! 2. Optional [`CopulaGenerator`]s for preserving correlations
//!
//! # Basic Usage
//!
//! ```ignore
//! use datasynth_fingerprint::synthesis::{ConfigSynthesizer, SynthesisOptions};
//!
//! // Create synthesizer with default options
//! let synthesizer = ConfigSynthesizer::new();
//!
//! // Basic synthesis (config only)
//! let config_patch = synthesizer.synthesize(&fingerprint)?;
//!
//! // Full synthesis with copula generators
//! let result = synthesizer.synthesize_full(&fingerprint, 42)?;
//!
//! // Use the config patch
//! let yaml = result.config_patch.to_yaml()?;
//!
//! // Use copula generators for correlated sampling
//! for copula_spec in result.copula_generators {
//!     let sample = copula_spec.generator.sample();
//!     // sample is a Vec<f64> with correlated values
//! }
//! ```
//!
//! # Synthesis Options
//!
//! Control the synthesis process with [`SynthesisOptions`]:
//!
//! ```ignore
//! let options = SynthesisOptions {
//!     scale: 2.0,              // Generate 2x the original row count
//!     seed: Some(42),          // Reproducible generation
//!     preserve_correlations: true,   // Create copula generators
//!     inject_anomalies: true,  // Apply anomaly rates from fingerprint
//! };
//!
//! let synthesizer = ConfigSynthesizer::with_options(options);
//! ```
//!
//! # Configuration Mappings
//!
//! The synthesizer maps fingerprint components to config values:
//!
//! | Fingerprint Component | Config Target | Description |
//! |-----------------------|---------------|-------------|
//! | `statistics.numeric_columns[].distribution` | `transactions.amounts.*` | Amount distribution |
//! | `schema.tables[].row_count` | `transactions.count` | Row count (scaled) |
//! | `anomalies.overall.rate` | `anomaly_injection.overall_rate` | Anomaly rate |
//! | `correlations.copulas[]` | Copula generators | Correlation preservation |
//!
//! # Copula Generators
//!
//! For preserving correlations between columns, the synthesizer creates
//! Gaussian copula generators:
//!
//! ```ignore
//! // The synthesis result includes copula generators
//! let result = synthesizer.synthesize_full(&fingerprint, seed)?;
//!
//! for spec in result.copula_generators {
//!     println!("Copula for table {}: columns {:?}",
//!         spec.table, spec.columns);
//!
//!     // Sample correlated uniform values
//!     let uniforms = spec.generator.sample();
//!
//!     // Transform to target distributions
//!     // uniforms[i] corresponds to spec.columns[i]
//! }
//! ```
//!
//! # Distribution Fitting
//!
//! The distribution fitting functions map fingerprint distributions to generator parameters:
//!
//! - Log-normal → `lognormal_mu`, `lognormal_sigma`
//! - Normal → Converted to log-normal approximation
//! - Empirical → Percentile-based estimation
//!
//! [`Fingerprint`]: crate::models::Fingerprint
//! [`ConfigPatch`]: config_synthesizer::ConfigPatch
//! [`CopulaGenerator`]: copula::CopulaGenerator
//! [`SynthesisOptions`]: config_synthesizer::SynthesisOptions

mod config_synthesizer;
mod copula;
mod distribution_fitter;

pub use config_synthesizer::*;
pub use copula::*;
pub use distribution_fitter::*;
