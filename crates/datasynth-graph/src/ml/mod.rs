//! Machine learning utilities for graph neural networks.
//!
//! This module provides comprehensive feature engineering for graph-based ML:
//! - Structural features (degree, clustering, etc.)
//! - Temporal sequence features (velocity, burst, trend)
//! - Motif detection (cycles, stars, back-and-forth)
//! - Relationship features (counterparty concentration, risk)
//! - Entity group detection and aggregation

pub mod aggregation;
pub mod entity_groups;
mod features;
pub mod motifs;
pub mod relationship_features;
mod splits;
pub mod temporal;

pub use aggregation::{
    aggregate_all_groups, aggregate_features, aggregate_node_features, aggregate_values,
    aggregate_weighted, AggregatedFeatures, AggregationType, MultiFeatureAggregation,
};
pub use entity_groups::{
    detect_entity_groups, EntityGroup, GroupDetectionAlgorithm, GroupDetectionConfig,
    GroupDetectionResult, GroupType,
};
pub use features::*;
pub use motifs::{
    compute_motif_features, detect_motifs, find_back_and_forth, find_circular_flows,
    find_star_patterns, CircularFlow, GraphMotif, MotifConfig, MotifDetectionResult,
    MotifInstance,
};
pub use relationship_features::{
    compute_all_combined_features, compute_all_counterparty_risk, compute_all_relationship_features,
    compute_counterparty_risk, compute_relationship_features, CombinedRelationshipFeatures,
    CounterpartyRisk, RelationshipFeatureConfig, RelationshipFeatures,
};
pub use splits::*;
pub use temporal::{
    compute_all_temporal_features, compute_temporal_sequence_features, TemporalConfig,
    TemporalFeatures, TemporalIndex, WindowFeatures,
};
