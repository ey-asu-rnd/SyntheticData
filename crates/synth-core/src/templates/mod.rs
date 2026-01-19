//! Template system for realistic data generation.
//!
//! This module provides templates for generating realistic:
//! - Person names (multi-cultural)
//! - Journal entry descriptions (header and line text)
//! - Reference numbers (invoices, POs, etc.)
//!
//! The template system supports both embedded templates and file-based customization
//! through YAML/JSON files for regional and sector-specific variations.

pub mod descriptions;
pub mod loader;
pub mod names;
pub mod provider;
pub mod references;

pub use descriptions::{DescriptionGenerator, HeaderTextPattern, LineTextPattern};
pub use loader::{
    AssetDescriptionTemplates, CultureNames, CustomerNameTemplates, HeaderTextTemplates,
    LineItemDescriptionTemplates, MaterialDescriptionTemplates, MergeStrategy, PersonNameTemplates,
    TemplateData, TemplateError, TemplateLoader, TemplateMetadata, VendorNameTemplates,
};
pub use names::{MultiCultureNameGenerator, NameCulture, NamePool, PersonName};
pub use provider::{
    default_provider, provider_from_file, DefaultTemplateProvider, SharedTemplateProvider,
    TemplateProvider,
};
pub use references::{ReferenceGenerator, ReferenceType};
