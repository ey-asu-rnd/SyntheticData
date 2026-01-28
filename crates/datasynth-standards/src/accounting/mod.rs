//! Accounting Standards Implementation.
//!
//! This module provides data structures and logic for major accounting
//! standards under both US GAAP and IFRS:
//!
//! - [`revenue`]: Revenue recognition (ASC 606 / IFRS 15)
//! - [`leases`]: Lease accounting (ASC 842 / IFRS 16)
//! - [`fair_value`]: Fair value measurement (ASC 820 / IFRS 13)
//! - [`impairment`]: Asset impairment (ASC 360 / IAS 36)
//! - [`differences`]: Framework difference tracking for dual reporting

pub mod differences;
pub mod fair_value;
pub mod impairment;
pub mod leases;
pub mod revenue;

pub use differences::*;
pub use fair_value::*;
pub use impairment::*;
pub use leases::*;
pub use revenue::*;
