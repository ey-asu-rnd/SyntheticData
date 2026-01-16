//! Anomaly injection framework for synthetic data generation.
//!
//! This module provides comprehensive anomaly injection capabilities:
//! - Configurable anomaly rates per category
//! - Temporal patterns (year-end spikes, clustering)
//! - Labeled output for supervised learning
//! - Multiple injection strategies

mod injector;
mod patterns;
mod strategies;
mod types;

pub use injector::*;
pub use patterns::*;
pub use strategies::*;
pub use types::*;
