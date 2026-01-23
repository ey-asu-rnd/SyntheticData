//! Fidelity evaluation for synthetic data.
//!
//! This module compares generated synthetic data against the original
//! fingerprint to assess how well the synthetic data matches the
//! statistical properties of the source.
//!
//! # Overview
//!
//! After generating synthetic data from a fingerprint, use the [`FidelityEvaluator`]
//! to assess how well the synthetic data matches the original statistical properties.
//!
//! # Usage
//!
//! ```ignore
//! use datasynth_fingerprint::evaluation::FidelityEvaluator;
//! use datasynth_fingerprint::extraction::FingerprintExtractor;
//! use datasynth_fingerprint::models::PrivacyLevel;
//!
//! // Extract fingerprint from original data
//! let extractor = FingerprintExtractor::new(PrivacyLevel::Standard);
//! let original_fp = extractor.extract_from_csv(Path::new("original.csv"))?;
//!
//! // Generate synthetic data... (not shown)
//!
//! // Extract fingerprint from synthetic data
//! let synthetic_fp = extractor.extract_from_csv(Path::new("synthetic.csv"))?;
//!
//! // Evaluate fidelity
//! let evaluator = FidelityEvaluator::new();
//! let report = evaluator.evaluate(&original_fp, &synthetic_fp)?;
//!
//! println!("Overall fidelity: {:.2}", report.overall_score);
//! println!("Statistical fidelity: {:.2}", report.statistical_fidelity);
//! println!("Correlation fidelity: {:.2}", report.correlation_fidelity);
//!
//! if report.passes {
//!     println!("Synthetic data passes fidelity check!");
//! }
//! ```
//!
//! # Fidelity Metrics
//!
//! The [`FidelityReport`] includes several metrics:
//!
//! | Metric | Range | Description |
//! |--------|-------|-------------|
//! | `overall_score` | 0.0-1.0 | Weighted combination of all metrics |
//! | `statistical_fidelity` | 0.0-1.0 | Distribution similarity |
//! | `correlation_fidelity` | 0.0-1.0 | Correlation preservation |
//! | `schema_fidelity` | 0.0-1.0 | Schema match (types, constraints) |
//! | `rule_compliance` | 0.0-1.0 | Business rule satisfaction |
//!
//! # Evaluation Criteria
//!
//! Statistical fidelity considers:
//! - Mean and variance similarity
//! - Distribution shape (KS test)
//! - Percentile alignment
//! - Benford's Law compliance (for amounts)
//!
//! Correlation fidelity considers:
//! - Correlation matrix RMSE
//! - Copula structure preservation
//!
//! [`FidelityEvaluator`]: fidelity::FidelityEvaluator
//! [`FidelityReport`]: fidelity::FidelityReport

mod fidelity;

pub use fidelity::*;
