//! Template system for realistic data generation.
//!
//! This module provides templates for generating realistic:
//! - Person names (multi-cultural)
//! - Journal entry descriptions (header and line text)
//! - Reference numbers (invoices, POs, etc.)

pub mod descriptions;
pub mod names;
pub mod references;

pub use descriptions::{DescriptionGenerator, HeaderTextPattern, LineTextPattern};
pub use names::{MultiCultureNameGenerator, NameCulture, NamePool, PersonName};
pub use references::{ReferenceGenerator, ReferenceType};
