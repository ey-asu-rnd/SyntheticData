//! Core traits for generators, output sinks, and post-processors.

mod generator;
mod post_processor;
mod sink;

pub use generator::*;
pub use post_processor::*;
pub use sink::*;
