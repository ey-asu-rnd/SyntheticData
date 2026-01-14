//! Statistical distribution samplers for realistic data generation.
//!
//! Based on empirical findings from the accounting network generation paper,
//! these samplers produce data that matches real-world distributions.

mod line_item;
mod amount;
mod temporal;

pub use line_item::*;
pub use amount::*;
pub use temporal::*;
