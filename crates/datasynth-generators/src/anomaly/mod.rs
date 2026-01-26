//! Anomaly injection framework for synthetic data generation.
//!
//! This module provides comprehensive anomaly injection capabilities:
//! - Configurable anomaly rates per category
//! - Temporal patterns (year-end spikes, clustering)
//! - Labeled output for supervised learning
//! - Multiple injection strategies
//! - Document flow anomalies (3-way match fraud)
//! - Dynamic confidence calculation (FR-003)
//! - Contextual severity scoring (FR-003)

pub mod confidence;
mod document_flow_anomalies;
mod injector;
mod patterns;
pub mod severity;
mod strategies;
mod types;

pub use confidence::{ConfidenceCalculator, ConfidenceConfig, ConfidenceContext};
pub use document_flow_anomalies::*;
pub use injector::*;
pub use patterns::*;
pub use severity::{AnomalyScoreCalculator, AnomalyScores, SeverityCalculator, SeverityConfig, SeverityContext};
pub use strategies::*;
pub use types::*;
