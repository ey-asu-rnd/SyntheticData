//! Statistical distribution samplers for realistic data generation.
//!
//! Based on empirical findings from the accounting network generation paper,
//! these samplers produce data that matches real-world distributions.

mod amount;
mod line_item;
mod temporal;

pub use amount::*;
pub use line_item::*;
pub use temporal::*;
