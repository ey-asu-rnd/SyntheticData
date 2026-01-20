//! Banking domain models for KYC/AML synthetic data generation.
//!
//! This module provides shared types used by the banking transaction generator
//! for compliance testing, AML model training, and fraud analytics.

mod account_type;
mod aml_typology;
mod customer_type;
mod risk_tier;
mod transaction_type;

pub use account_type::*;
pub use aml_typology::*;
pub use customer_type::*;
pub use risk_tier::*;
pub use transaction_type::*;
