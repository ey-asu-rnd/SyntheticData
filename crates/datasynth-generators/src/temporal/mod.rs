//! Temporal attribute generation module.
//!
//! This module provides generators for adding temporal attributes to entities,
//! supporting bi-temporal data models with valid time and transaction time.
//!
//! # Features
//!
//! - **Valid Time Generation**: Business time ranges for when facts are true
//! - **Transaction Time Generation**: System recording times with optional delays
//! - **Version Chain Generation**: Create version histories for entities
//! - **Configurable Parameters**: Control validity durations, backdating, etc.
//!
//! # Example
//!
//! ```ignore
//! use datasynth_generators::temporal::{TemporalAttributeGenerator, TemporalAttributeConfig};
//!
//! let config = TemporalAttributeConfig::default();
//! let mut generator = TemporalAttributeGenerator::new(config, 42, base_date);
//!
//! // Wrap an entity with temporal attributes
//! let temporal_vendor = generator.generate_temporal(vendor);
//! ```

mod temporal_generator;

pub use temporal_generator::*;
