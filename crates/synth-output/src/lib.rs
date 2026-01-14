//! # synth-output
//!
//! Output sinks for CSV, Parquet, JSON, and streaming formats.

pub mod csv_sink;
pub mod parquet_sink;
pub mod json_sink;

pub use csv_sink::*;
pub use parquet_sink::*;
pub use json_sink::*;
