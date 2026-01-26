//! Relationship generation module.
//!
//! This module provides generators for creating relationships between entities,
//! supporting configurable cardinality rules and property generation.
//!
//! # Features
//!
//! - **Cardinality Rules**: OneToOne, OneToMany, ManyToOne, ManyToMany
//! - **Property Generation**: Generate relationship properties from rules
//! - **Orphan Control**: Allow/prevent orphan entities
//! - **Circular Detection**: Detect and optionally prevent circular relationships
//!
//! # Example
//!
//! ```ignore
//! use datasynth_generators::relationships::{RelationshipGenerator, RelationshipConfig};
//!
//! let config = RelationshipConfig::default();
//! let mut generator = RelationshipGenerator::new(config, 42);
//!
//! // Generate relationships between nodes
//! let edges = generator.generate_relationships(&nodes);
//! ```

mod generator;
mod rules;

pub use generator::*;
pub use rules::*;
