//! FX (Foreign Exchange) generators.
//!
//! This module provides generators for:
//! - FX rates using Ornstein-Uhlenbeck mean-reverting process
//! - Currency translation for trial balances
//! - Currency Translation Adjustment (CTA) calculations

mod fx_rate_service;
mod currency_translator;
mod cta_generator;

pub use fx_rate_service::*;
pub use currency_translator::*;
pub use cta_generator::*;
