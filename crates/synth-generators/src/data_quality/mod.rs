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

mod missing_values;
mod format_variations;
mod duplicates;
mod typos;
mod injector;

pub use missing_values::*;
pub use format_variations::*;
pub use duplicates::*;
pub use typos::*;
pub use injector::*;
