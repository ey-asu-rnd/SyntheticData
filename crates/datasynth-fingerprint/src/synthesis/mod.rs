//! Config synthesis from fingerprints.
//!
//! This module converts fingerprints into generator configurations
//! that can be used to generate matching synthetic data.

mod config_synthesizer;
mod copula;
mod distribution_fitter;

pub use config_synthesizer::*;
pub use copula::*;
pub use distribution_fitter::*;
