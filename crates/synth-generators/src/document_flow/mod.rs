//! Document flow generators for P2P and O2C processes.
//!
//! This module provides generators for complete document flows:
//! - P2P (Procure-to-Pay): PO → GR → Invoice → Payment
//! - O2C (Order-to-Cash): SO → Delivery → Invoice → Receipt
//!
//! The `document_flow_je_generator` submodule creates corresponding
//! journal entries from document flows to ensure GL coherence.

mod document_chain_manager;
mod document_flow_je_generator;
mod o2c_generator;
mod p2p_generator;
mod three_way_match;

pub use document_chain_manager::*;
pub use document_flow_je_generator::*;
pub use o2c_generator::*;
pub use p2p_generator::*;
pub use three_way_match::*;
