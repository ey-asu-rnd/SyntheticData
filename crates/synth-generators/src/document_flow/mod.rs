//! Document flow generators for P2P and O2C processes.
//!
//! This module provides generators for complete document flows:
//! - P2P (Procure-to-Pay): PO → GR → Invoice → Payment
//! - O2C (Order-to-Cash): SO → Delivery → Invoice → Receipt

mod document_chain_manager;
mod o2c_generator;
mod p2p_generator;

pub use document_chain_manager::*;
pub use o2c_generator::*;
pub use p2p_generator::*;
