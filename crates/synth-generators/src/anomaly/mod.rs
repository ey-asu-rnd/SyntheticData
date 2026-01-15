//! Anomaly injection framework for synthetic data generation.
//!
//! This module provides comprehensive anomaly injection capabilities:
//! - Configurable anomaly rates per category
//! - Temporal patterns (year-end spikes, clustering)
//! - Labeled output for supervised learning
//! - Multiple injection strategies

mod types;
mod strategies;
mod patterns;
mod injector;

pub use types::*;
pub use strategies::*;
pub use patterns::*;
pub use injector::*;
