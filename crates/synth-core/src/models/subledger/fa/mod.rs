//! Fixed Assets (FA) subledger models.
//!
//! This module provides models for:
//! - Fixed asset register
//! - Depreciation schedules and calculations
//! - Asset disposals and retirements

mod asset;
mod depreciation;
mod disposal;

pub use asset::*;
pub use depreciation::*;
pub use disposal::*;
