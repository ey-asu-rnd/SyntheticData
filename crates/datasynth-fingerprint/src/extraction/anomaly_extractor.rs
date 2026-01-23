//! Anomaly pattern extractor.

use crate::error::FingerprintResult;
use crate::models::{AnomalyFingerprint, AnomalyOverview};
use crate::privacy::PrivacyEngine;

use super::{DataSource, ExtractedComponent, ExtractionConfig, Extractor};

/// Extractor for anomaly patterns.
pub struct AnomalyExtractor;

impl Extractor for AnomalyExtractor {
    fn name(&self) -> &'static str {
        "anomalies"
    }

    fn extract(
        &self,
        _data: &DataSource,
        _config: &ExtractionConfig,
        _privacy: &mut PrivacyEngine,
    ) -> FingerprintResult<ExtractedComponent> {
        // Anomaly extraction requires labeled data or statistical detection
        // For now, return empty anomaly fingerprint
        let overview = AnomalyOverview::new(0, 0);
        let anomalies = AnomalyFingerprint::new(overview);
        Ok(ExtractedComponent::Anomalies(anomalies))
    }
}
