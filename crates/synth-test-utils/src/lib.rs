//! Test utilities for synthetic data generation testing.
//!
//! This crate provides common testing utilities including:
//! - Pre-built test fixtures and configurations
//! - Custom assertion macros for accounting invariants
//! - Mock implementations for testing
//! - Test server utilities

pub mod assertions;
pub mod fixtures;
pub mod mocks;
pub mod server;

pub use assertions::*;
pub use fixtures::*;
pub use mocks::*;
pub use server::*;
