//! Document models for enterprise transaction flow simulation.
//!
//! This module provides document models representing the various
//! business documents in P2P (Procure-to-Pay) and O2C (Order-to-Cash) flows:
//!
//! - Purchase Orders (PO)
//! - Goods Receipts (GR)
//! - Vendor Invoices
//! - AP Payments
//! - Sales Orders (SO)
//! - Deliveries
//! - Customer Invoices
//! - Customer Receipts

mod document_chain;
mod purchase_order;
mod goods_receipt;
mod vendor_invoice;
mod payment;
mod sales_order;
mod delivery;
mod customer_invoice;

pub use document_chain::*;
pub use purchase_order::*;
pub use goods_receipt::*;
pub use vendor_invoice::*;
pub use payment::*;
pub use sales_order::*;
pub use delivery::*;
pub use customer_invoice::*;
