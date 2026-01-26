//! Rate limiting for controlling generation throughput.
//!
//! This module provides rate limiting utilities for controlling how fast
//! data is generated, useful for:
//! - Preventing resource exhaustion
//! - Simulating real-world data arrival rates
//! - Controlling streaming output bandwidth
//!
//! # Example
//!
//! ```
//! use datasynth_core::rate_limit::{RateLimiter, RateLimitConfig};
//!
//! let config = RateLimitConfig {
//!     entities_per_second: 1000.0,
//!     burst_size: 100,
//!     ..Default::default()
//! };
//!
//! let mut limiter = RateLimiter::new(config);
//!
//! // In a generation loop:
//! // limiter.acquire(); // Blocks if rate exceeded
//! ```

mod limiter;

pub use limiter::*;
