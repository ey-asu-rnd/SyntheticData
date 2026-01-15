//! # synth-core
//!
//! Core domain models, traits, and distributions for synthetic accounting data generation.
//!
//! This crate provides the foundational types used throughout the synthetic data factory:
//! - Journal Entry models (header and line items)
//! - Chart of Accounts structures
//! - SAP HANA ACDOCA/BSEG compatible event log formats
//! - Generator and Sink traits for extensibility
//! - Statistical distribution samplers based on empirical research
//! - Templates for realistic data generation (names, descriptions, references)

pub mod distributions;
pub mod error;
pub mod models;
pub mod templates;
pub mod traits;

pub use distributions::*;
pub use error::*;
pub use models::*;
pub use templates::*;
pub use traits::*;
