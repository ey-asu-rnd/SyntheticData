//! Fidelity evaluation for synthetic data.
//!
//! This module compares generated synthetic data against the original
//! fingerprint to assess how well the synthetic data matches the
//! statistical properties of the source.

mod fidelity;

pub use fidelity::*;
