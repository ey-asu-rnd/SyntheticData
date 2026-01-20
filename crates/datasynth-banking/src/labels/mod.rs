//! Ground truth label generation module.
//!
//! This module provides comprehensive labeling for ML training:
//! - Transaction-level labels (suspicious, typology, stage)
//! - Entity-level labels (risk tier, mule status, deception)
//! - Relationship-level labels (mule links, shell links, ownership)
//! - Case narratives for explainability

mod entity_labels;
mod narrative_generator;
mod relationship_labels;
mod transaction_labels;

pub use entity_labels::*;
pub use narrative_generator::*;
pub use relationship_labels::*;
pub use transaction_labels::*;
