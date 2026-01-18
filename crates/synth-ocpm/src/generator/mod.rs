//! OCPM event generators.
//!
//! This module provides generators for creating OCPM events from document flows
//! and business processes.

mod event_generator;
mod p2p_generator;
mod o2c_generator;

pub use event_generator::*;
pub use p2p_generator::*;
pub use o2c_generator::*;
