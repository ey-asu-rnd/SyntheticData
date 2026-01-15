//! Master data generators for enterprise simulation.
//!
//! This module provides generators for various master data entities:
//! - Vendors (enhanced with payment behavior)
//! - Customers (enhanced with credit management)
//! - Materials (inventory and BOM)
//! - Fixed Assets (with depreciation)
//! - Employees (with org hierarchy)
//! - Entity Registry (central entity management)

mod vendor_generator;
mod customer_generator;
mod material_generator;
mod asset_generator;
mod employee_generator;
mod entity_registry_manager;

pub use vendor_generator::*;
pub use customer_generator::*;
pub use material_generator::*;
pub use asset_generator::*;
pub use employee_generator::*;
pub use entity_registry_manager::*;
