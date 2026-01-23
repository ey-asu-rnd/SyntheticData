//! Extraction engine for fingerprinting.
//!
//! This module provides extractors that analyze data and produce
//! fingerprint components while applying privacy mechanisms.

mod anomaly_extractor;
mod correlation_extractor;
mod integrity_extractor;
mod rules_extractor;
mod schema_extractor;
mod stats_extractor;

pub use anomaly_extractor::*;
pub use correlation_extractor::*;
pub use integrity_extractor::*;
pub use rules_extractor::*;
pub use schema_extractor::*;
pub use stats_extractor::*;

use std::path::Path;

use crate::error::{FingerprintError, FingerprintResult};
use crate::models::{
    Fingerprint, Manifest, PrivacyLevel, PrivacyMetadata, SchemaFingerprint,
    SourceMetadata, StatisticsFingerprint,
};
use crate::privacy::{PrivacyConfig, PrivacyEngine};

/// Configuration for fingerprint extraction.
#[derive(Debug, Clone)]
pub struct ExtractionConfig {
    /// Privacy configuration.
    pub privacy: PrivacyConfig,
    /// Whether to extract correlations.
    pub extract_correlations: bool,
    /// Whether to extract integrity constraints.
    pub extract_integrity: bool,
    /// Whether to extract business rules.
    pub extract_rules: bool,
    /// Whether to extract anomaly patterns.
    pub extract_anomalies: bool,
    /// Maximum sample size for large datasets.
    pub max_sample_size: Option<usize>,
    /// Minimum rows required for extraction.
    pub min_rows: usize,
}

impl Default for ExtractionConfig {
    fn default() -> Self {
        Self {
            privacy: PrivacyConfig::from_level(PrivacyLevel::Standard),
            extract_correlations: true,
            extract_integrity: true,
            extract_rules: true,
            extract_anomalies: true,
            max_sample_size: None,
            min_rows: 10,
        }
    }
}

impl ExtractionConfig {
    /// Create with a specific privacy level.
    pub fn with_privacy_level(level: PrivacyLevel) -> Self {
        Self {
            privacy: PrivacyConfig::from_level(level),
            ..Default::default()
        }
    }
}

/// Trait for data extractors.
pub trait Extractor: Send + Sync {
    /// Name of this extractor.
    fn name(&self) -> &'static str;

    /// Extract component from data.
    fn extract(
        &self,
        data: &DataSource,
        config: &ExtractionConfig,
        privacy: &mut PrivacyEngine,
    ) -> FingerprintResult<ExtractedComponent>;
}

/// Source of data for extraction.
#[derive(Debug)]
pub enum DataSource {
    /// CSV file.
    Csv(CsvDataSource),
    /// In-memory data.
    Memory(MemoryDataSource),
}

/// CSV data source.
#[derive(Debug)]
pub struct CsvDataSource {
    /// Path to the CSV file.
    pub path: std::path::PathBuf,
    /// Whether the CSV has headers.
    pub has_headers: bool,
    /// Delimiter character.
    pub delimiter: u8,
}

impl CsvDataSource {
    /// Create from a path.
    pub fn new(path: impl AsRef<Path>) -> Self {
        Self {
            path: path.as_ref().to_path_buf(),
            has_headers: true,
            delimiter: b',',
        }
    }
}

/// In-memory data source.
#[derive(Debug)]
pub struct MemoryDataSource {
    /// Column names.
    pub columns: Vec<String>,
    /// Row data (each inner Vec is a row).
    pub rows: Vec<Vec<String>>,
}

impl MemoryDataSource {
    /// Create from columns and rows.
    pub fn new(columns: Vec<String>, rows: Vec<Vec<String>>) -> Self {
        Self { columns, rows }
    }

    /// Get row count.
    pub fn row_count(&self) -> usize {
        self.rows.len()
    }

    /// Get column count.
    pub fn column_count(&self) -> usize {
        self.columns.len()
    }
}

/// Result of extraction from a single extractor.
#[derive(Debug)]
pub enum ExtractedComponent {
    Schema(SchemaFingerprint),
    Statistics(StatisticsFingerprint),
    Correlations(crate::models::CorrelationFingerprint),
    Integrity(crate::models::IntegrityFingerprint),
    Rules(crate::models::RulesFingerprint),
    Anomalies(crate::models::AnomalyFingerprint),
}

/// Main fingerprint extractor that coordinates all extraction.
pub struct FingerprintExtractor {
    config: ExtractionConfig,
}

impl FingerprintExtractor {
    /// Create a new extractor with default configuration.
    pub fn new() -> Self {
        Self {
            config: ExtractionConfig::default(),
        }
    }

    /// Create with a specific privacy level.
    pub fn with_privacy_level(level: PrivacyLevel) -> Self {
        Self {
            config: ExtractionConfig::with_privacy_level(level),
        }
    }

    /// Create with custom configuration.
    pub fn with_config(config: ExtractionConfig) -> Self {
        Self { config }
    }

    /// Extract fingerprint from a CSV file.
    pub fn extract_from_csv(&self, path: impl AsRef<Path>) -> FingerprintResult<Fingerprint> {
        let source = DataSource::Csv(CsvDataSource::new(path));
        self.extract(&source)
    }

    /// Extract fingerprint from in-memory data.
    pub fn extract_from_memory(
        &self,
        columns: Vec<String>,
        rows: Vec<Vec<String>>,
    ) -> FingerprintResult<Fingerprint> {
        let source = DataSource::Memory(MemoryDataSource::new(columns, rows));
        self.extract(&source)
    }

    /// Extract fingerprint from a data source.
    pub fn extract(&self, source: &DataSource) -> FingerprintResult<Fingerprint> {
        let mut privacy = PrivacyEngine::new(self.config.privacy.clone());

        // Extract schema
        let schema_extractor = SchemaExtractor;
        let schema = match schema_extractor.extract(source, &self.config, &mut privacy)? {
            ExtractedComponent::Schema(s) => s,
            _ => return Err(FingerprintError::extraction("schema", "Unexpected component type")),
        };

        // Extract statistics
        let stats_extractor = StatsExtractor;
        let statistics = match stats_extractor.extract(source, &self.config, &mut privacy)? {
            ExtractedComponent::Statistics(s) => s,
            _ => return Err(FingerprintError::extraction("statistics", "Unexpected component type")),
        };

        // Extract optional components
        let correlations = if self.config.extract_correlations {
            let extractor = CorrelationExtractor;
            match extractor.extract(source, &self.config, &mut privacy) {
                Ok(ExtractedComponent::Correlations(c)) => Some(c),
                Ok(_) => None,
                Err(_) => None, // Optional, ignore errors
            }
        } else {
            None
        };

        let integrity = if self.config.extract_integrity {
            let extractor = IntegrityExtractor;
            match extractor.extract(source, &self.config, &mut privacy) {
                Ok(ExtractedComponent::Integrity(i)) => Some(i),
                Ok(_) => None,
                Err(_) => None,
            }
        } else {
            None
        };

        let rules = if self.config.extract_rules {
            let extractor = RulesExtractor;
            match extractor.extract(source, &self.config, &mut privacy) {
                Ok(ExtractedComponent::Rules(r)) => Some(r),
                Ok(_) => None,
                Err(_) => None,
            }
        } else {
            None
        };

        let anomalies = if self.config.extract_anomalies {
            let extractor = AnomalyExtractor;
            match extractor.extract(source, &self.config, &mut privacy) {
                Ok(ExtractedComponent::Anomalies(a)) => Some(a),
                Ok(_) => None,
                Err(_) => None,
            }
        } else {
            None
        };

        // Build manifest
        let source_meta = build_source_metadata(source, &schema);
        let privacy_meta = PrivacyMetadata::from_level(PrivacyLevel::Standard);
        let manifest = Manifest::new(source_meta, privacy_meta);

        // Get privacy audit
        let privacy_audit = privacy.into_audit();

        // Build fingerprint
        let mut fingerprint = Fingerprint::new(manifest, schema, statistics, privacy_audit);

        if let Some(c) = correlations {
            fingerprint = fingerprint.with_correlations(c);
        }
        if let Some(i) = integrity {
            fingerprint = fingerprint.with_integrity(i);
        }
        if let Some(r) = rules {
            fingerprint = fingerprint.with_rules(r);
        }
        if let Some(a) = anomalies {
            fingerprint = fingerprint.with_anomalies(a);
        }

        Ok(fingerprint)
    }
}

impl Default for FingerprintExtractor {
    fn default() -> Self {
        Self::new()
    }
}

/// Build source metadata from data source and schema.
fn build_source_metadata(source: &DataSource, schema: &SchemaFingerprint) -> SourceMetadata {
    let (description, tables, total_rows) = match source {
        DataSource::Csv(csv) => {
            let name = csv.path.file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("unknown")
                .to_string();
            let rows = schema.tables.values().map(|t| t.row_count).sum();
            (format!("CSV file: {}", name), vec![name], rows)
        }
        DataSource::Memory(mem) => {
            let rows = mem.row_count() as u64;
            ("In-memory data".to_string(), vec!["memory".to_string()], rows)
        }
    };

    SourceMetadata::new(description, tables, total_rows)
}
