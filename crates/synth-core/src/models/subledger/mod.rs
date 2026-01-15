//! Subledger models for detailed transaction tracking.
//!
//! This module provides models for the four main subledgers:
//! - **AR (Accounts Receivable)**: Customer invoices, receipts, credit memos, aging
//! - **AP (Accounts Payable)**: Vendor invoices, payments, debit memos, schedules
//! - **FA (Fixed Assets)**: Asset register, depreciation, disposals
//! - **Inventory**: Positions, movements, valuations
//!
//! Each subledger maintains detailed records that:
//! 1. Track individual transactions at document level
//! 2. Generate corresponding GL journal entries
//! 3. Reconcile to GL control accounts

mod common;
pub mod ar;
pub mod ap;
pub mod fa;
pub mod inventory;

pub use common::*;
