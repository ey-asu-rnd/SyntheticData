//! Generators for banking synthetic data.
//!
//! This module provides generators for:
//! - Customers (retail, business, trust)
//! - Accounts (checking, savings, business, etc.)
//! - Transactions (persona-based behavioral generation)
//! - Counterparties (merchants, employers, utilities)
//! - KYC profiles (expected activity envelopes)

mod account_generator;
mod counterparty_generator;
mod customer_generator;
mod kyc_generator;
mod transaction_generator;

pub use account_generator::*;
pub use counterparty_generator::*;
pub use customer_generator::*;
pub use kyc_generator::*;
pub use transaction_generator::*;
