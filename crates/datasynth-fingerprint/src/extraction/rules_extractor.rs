//! Business rules extractor.

use crate::error::FingerprintResult;
use crate::models::RulesFingerprint;
use crate::privacy::PrivacyEngine;

use super::{DataSource, ExtractionConfig, ExtractedComponent, Extractor};

/// Extractor for business rules.
pub struct RulesExtractor;

impl Extractor for RulesExtractor {
    fn name(&self) -> &'static str {
        "rules"
    }

    fn extract(
        &self,
        _data: &DataSource,
        _config: &ExtractionConfig,
        _privacy: &mut PrivacyEngine,
    ) -> FingerprintResult<ExtractedComponent> {
        // Rules extraction requires domain knowledge
        // For now, return empty rules
        let rules = RulesFingerprint::new();
        Ok(ExtractedComponent::Rules(rules))
    }
}
