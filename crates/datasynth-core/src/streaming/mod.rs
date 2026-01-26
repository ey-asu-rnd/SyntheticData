//! Streaming infrastructure for real-time data generation.
//!
//! This module provides utilities for streaming generation including:
//! - Channel management for producer-consumer patterns
//! - Backpressure handling strategies
//! - Stream coordination and control

mod backpressure;
mod channel;

pub use backpressure::*;
pub use channel::*;
