//! Enhancement derivation module for automatic configuration optimization.
//!
//! This module provides tools to analyze evaluation results and derive
//! configuration enhancements that will improve data quality metrics.
//!
//! The enhancement pipeline follows this flow:
//! ```text
//! Evaluation Results → Threshold Check → Gap Analysis → Root Cause → Config Suggestion
//! ```

mod auto_tuner;
mod recommendation_engine;

pub use auto_tuner::*;
pub use recommendation_engine::*;
