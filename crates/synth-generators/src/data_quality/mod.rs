//! Data quality variations for realistic synthetic data.
//!
//! This module provides tools to introduce realistic data quality issues:
//! - Missing values (configurable by field)
//! - Format variations (dates, amounts, identifiers)
//! - Duplicates (exact and near-duplicates)
//! - Typos (substitution, transposition, insertion, deletion)
//! - Encoding issues (character corruption)
//!
//! These variations make synthetic data more realistic for testing
//! data cleaning, ETL pipelines, and data quality tools.

mod duplicates;
mod format_variations;
mod injector;
mod missing_values;
mod typos;

pub use duplicates::*;
pub use format_variations::*;
pub use injector::*;
pub use missing_values::*;
pub use typos::*;
