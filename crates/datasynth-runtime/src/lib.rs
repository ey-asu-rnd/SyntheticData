//! # synth-runtime
//!
//! Runtime orchestration, parallel execution, and memory management.
//!
//! This crate provides two orchestrators:
//! - `GenerationOrchestrator`: Basic orchestrator for CoA and journal entries
//! - `EnhancedOrchestrator`: Full-featured orchestrator with all phases

pub mod enhanced_orchestrator;
pub mod orchestrator;

pub use enhanced_orchestrator::*;
pub use orchestrator::*;
