//! Accounts Receivable (AR) subledger models.
//!
//! This module provides models for:
//! - Customer invoices
//! - Customer receipts/payments
//! - Credit memos
//! - AR aging analysis
//! - Dunning (Mahnungen) process
//! - Payment corrections (NSF, chargebacks)
//! - Short payments and on-account payments

mod aging;
mod credit_memo;
mod dunning;
mod invoice;
mod payment_correction;
mod receipt;

pub use aging::*;
pub use credit_memo::*;
pub use dunning::*;
pub use invoice::*;
pub use payment_correction::*;
pub use receipt::*;
