//! AML typology injection module.
//!
//! This module provides injection of various AML patterns:
//! - Structuring / Smurfing
//! - Funnel accounts
//! - Layering chains
//! - Round-tripping
//! - Money mule networks
//! - Fraud patterns
//! - Spoofing mode

mod funnel;
mod injector;
mod layering;
mod mule;
mod spoofing;
mod structuring;

pub use funnel::*;
pub use injector::*;
pub use layering::*;
pub use mule::*;
pub use spoofing::*;
pub use structuring::*;
