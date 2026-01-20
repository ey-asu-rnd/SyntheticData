//! AML typology injection module.
//!
//! This module provides injection of various AML patterns:
//! - Structuring / Smurfing
//! - Funnel accounts
//! - Layering chains
//! - Round-tripping
//! - Money mule networks
//! - Fraud patterns (ATO, BEC, fake vendors, APP)
//! - Spoofing mode

mod fraud;
mod funnel;
mod injector;
mod layering;
mod mule;
mod round_tripping;
mod spoofing;
mod structuring;

pub use fraud::*;
pub use funnel::*;
pub use injector::*;
pub use layering::*;
pub use mule::*;
pub use round_tripping::*;
pub use spoofing::*;
pub use structuring::*;
