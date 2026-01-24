//! Main anomaly injection engine.
//!
//! The injector coordinates anomaly generation across all data types,
//! managing rates, patterns, clustering, and label generation.

use chrono::NaiveDate;
use rand::Rng;
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;
use rust_decimal::Decimal;
use std::collections::HashMap;

use datasynth_core::models::{
    AnomalyCausalReason, AnomalyRateConfig, AnomalySummary, AnomalyType, ErrorType, FraudType,
    JournalEntry, LabeledAnomaly, RelationalAnomalyType,
};

use super::patterns::{
    should_inject_anomaly, AnomalyPatternConfig, ClusterManager, EntityTargetingManager,
    TemporalPattern,
};
use super::strategies::{DuplicationStrategy, StrategyCollection};
use super::types::AnomalyTypeSelector;

/// Configuration for the anomaly injector.
#[derive(Debug, Clone)]
pub struct AnomalyInjectorConfig {
    /// Rate configuration.
    pub rates: AnomalyRateConfig,
    /// Pattern configuration.
    pub patterns: AnomalyPatternConfig,
    /// Random seed for reproducibility.
    pub seed: u64,
    /// Whether to generate labels.
    pub generate_labels: bool,
    /// Whether to allow duplicate injection.
    pub allow_duplicates: bool,
    /// Maximum anomalies per document.
    pub max_anomalies_per_document: usize,
    /// Company codes to target (empty = all).
    pub target_companies: Vec<String>,
    /// Date range for injection.
    pub date_range: Option<(NaiveDate, NaiveDate)>,
}

impl Default for AnomalyInjectorConfig {
    fn default() -> Self {
        Self {
            rates: AnomalyRateConfig::default(),
            patterns: AnomalyPatternConfig::default(),
            seed: 42,
            generate_labels: true,
            allow_duplicates: true,
            max_anomalies_per_document: 2,
            target_companies: Vec::new(),
            date_range: None,
        }
    }
}

/// Result of an injection batch.
#[derive(Debug, Clone)]
pub struct InjectionBatchResult {
    /// Number of entries processed.
    pub entries_processed: usize,
    /// Number of anomalies injected.
    pub anomalies_injected: usize,
    /// Number of duplicates created.
    pub duplicates_created: usize,
    /// Labels generated.
    pub labels: Vec<LabeledAnomaly>,
    /// Summary of anomalies.
    pub summary: AnomalySummary,
    /// Entries that were modified (document numbers).
    pub modified_documents: Vec<String>,
}

/// Main anomaly injection engine.
#[allow(dead_code)]
pub struct AnomalyInjector {
    config: AnomalyInjectorConfig,
    rng: ChaCha8Rng,
    type_selector: AnomalyTypeSelector,
    strategies: StrategyCollection,
    cluster_manager: ClusterManager,
    entity_targeting: EntityTargetingManager,
    /// Tracking which documents already have anomalies.
    document_anomaly_counts: HashMap<String, usize>,
    /// All generated labels.
    labels: Vec<LabeledAnomaly>,
    /// Statistics.
    stats: InjectorStats,
}

/// Internal statistics tracking.
#[derive(Debug, Clone, Default)]
#[allow(dead_code)]
pub struct InjectorStats {
    total_processed: usize,
    total_injected: usize,
    by_category: HashMap<String, usize>,
    by_type: HashMap<String, usize>,
    by_company: HashMap<String, usize>,
    skipped_rate: usize,
    skipped_date: usize,
    skipped_company: usize,
    skipped_max_per_doc: usize,
}

impl AnomalyInjector {
    /// Creates a new anomaly injector.
    pub fn new(config: AnomalyInjectorConfig) -> Self {
        let rng = ChaCha8Rng::seed_from_u64(config.seed);
        let cluster_manager = ClusterManager::new(config.patterns.clustering.clone());
        let entity_targeting =
            EntityTargetingManager::new(config.patterns.entity_targeting.clone());

        Self {
            config,
            rng,
            type_selector: AnomalyTypeSelector::new(),
            strategies: StrategyCollection::default(),
            cluster_manager,
            entity_targeting,
            document_anomaly_counts: HashMap::new(),
            labels: Vec::new(),
            stats: InjectorStats::default(),
        }
    }

    /// Processes a batch of journal entries, potentially injecting anomalies.
    pub fn process_entries(&mut self, entries: &mut [JournalEntry]) -> InjectionBatchResult {
        let mut modified_documents = Vec::new();
        let mut duplicates = Vec::new();

        for entry in entries.iter_mut() {
            self.stats.total_processed += 1;

            // Check if we should process this entry
            if !self.should_process(entry) {
                continue;
            }

            // Determine if we inject an anomaly
            if should_inject_anomaly(
                self.config.rates.total_rate,
                entry.posting_date(),
                &self.config.patterns.temporal_pattern,
                &mut self.rng,
            ) {
                // Select anomaly category based on rates
                let anomaly_type = self.select_anomaly_category();

                // Apply the anomaly
                if let Some(label) = self.inject_anomaly(entry, anomaly_type) {
                    modified_documents.push(entry.document_number().clone());
                    self.labels.push(label);
                    self.stats.total_injected += 1;
                }

                // Check for duplicate injection
                if self.config.allow_duplicates
                    && matches!(
                        self.labels.last().map(|l| &l.anomaly_type),
                        Some(AnomalyType::Error(ErrorType::DuplicateEntry))
                            | Some(AnomalyType::Fraud(FraudType::DuplicatePayment))
                    )
                {
                    let dup_strategy = DuplicationStrategy::default();
                    let duplicate = dup_strategy.duplicate(entry, &mut self.rng);
                    duplicates.push(duplicate);
                }
            }
        }

        // Count duplicates
        let duplicates_created = duplicates.len();

        // Build summary
        let summary = AnomalySummary::from_anomalies(&self.labels);

        InjectionBatchResult {
            entries_processed: self.stats.total_processed,
            anomalies_injected: self.stats.total_injected,
            duplicates_created,
            labels: self.labels.clone(),
            summary,
            modified_documents,
        }
    }

    /// Checks if an entry should be processed.
    fn should_process(&mut self, entry: &JournalEntry) -> bool {
        // Check company filter
        if !self.config.target_companies.is_empty()
            && !self
                .config
                .target_companies
                .iter()
                .any(|c| c == entry.company_code())
        {
            self.stats.skipped_company += 1;
            return false;
        }

        // Check date range
        if let Some((start, end)) = self.config.date_range {
            if entry.posting_date() < start || entry.posting_date() > end {
                self.stats.skipped_date += 1;
                return false;
            }
        }

        // Check max anomalies per document
        let current_count = self
            .document_anomaly_counts
            .get(&entry.document_number())
            .copied()
            .unwrap_or(0);
        if current_count >= self.config.max_anomalies_per_document {
            self.stats.skipped_max_per_doc += 1;
            return false;
        }

        true
    }

    /// Selects an anomaly category based on configured rates.
    fn select_anomaly_category(&mut self) -> AnomalyType {
        let r = self.rng.gen::<f64>();
        let rates = &self.config.rates;

        let mut cumulative = 0.0;

        cumulative += rates.fraud_rate;
        if r < cumulative {
            return self.type_selector.select_fraud(&mut self.rng);
        }

        cumulative += rates.error_rate;
        if r < cumulative {
            return self.type_selector.select_error(&mut self.rng);
        }

        cumulative += rates.process_issue_rate;
        if r < cumulative {
            return self.type_selector.select_process_issue(&mut self.rng);
        }

        cumulative += rates.statistical_rate;
        if r < cumulative {
            return self.type_selector.select_statistical(&mut self.rng);
        }

        self.type_selector.select_relational(&mut self.rng)
    }

    /// Injects an anomaly into an entry.
    fn inject_anomaly(
        &mut self,
        entry: &mut JournalEntry,
        anomaly_type: AnomalyType,
    ) -> Option<LabeledAnomaly> {
        // Check if strategy can be applied
        if !self.strategies.can_apply(entry, &anomaly_type) {
            return None;
        }

        // Apply the strategy
        let result = self
            .strategies
            .apply_strategy(entry, &anomaly_type, &mut self.rng);

        if !result.success {
            return None;
        }

        // Update document anomaly count
        *self
            .document_anomaly_counts
            .entry(entry.document_number().clone())
            .or_insert(0) += 1;

        // Update statistics
        let category = anomaly_type.category().to_string();
        let type_name = anomaly_type.type_name();

        *self.stats.by_category.entry(category).or_insert(0) += 1;
        *self.stats.by_type.entry(type_name.clone()).or_insert(0) += 1;
        *self
            .stats
            .by_company
            .entry(entry.company_code().to_string())
            .or_insert(0) += 1;

        // Generate label
        if self.config.generate_labels {
            let anomaly_id = format!("ANO{:08}", self.labels.len() + 1);

            // Update entry header with anomaly tracking fields
            entry.header.is_anomaly = true;
            entry.header.anomaly_id = Some(anomaly_id.clone());
            entry.header.anomaly_type = Some(type_name.clone());

            // Also set fraud flag if this is a fraud anomaly
            if matches!(anomaly_type, AnomalyType::Fraud(_)) {
                entry.header.is_fraud = true;
                if let AnomalyType::Fraud(ref ft) = anomaly_type {
                    entry.header.fraud_type = Some(*ft);
                }
            }

            let mut label = LabeledAnomaly::new(
                anomaly_id,
                anomaly_type.clone(),
                entry.document_number().clone(),
                "JE".to_string(),
                entry.company_code().to_string(),
                entry.posting_date(),
            )
            .with_description(&result.description)
            .with_injection_strategy(&type_name);

            // Add causal reason with injection context (provenance tracking)
            let causal_reason = AnomalyCausalReason::RandomRate {
                base_rate: self.config.rates.total_rate,
            };
            label = label.with_causal_reason(causal_reason);

            // Add monetary impact
            if let Some(impact) = result.monetary_impact {
                label = label.with_monetary_impact(impact);
            }

            // Add related entities
            for entity in &result.related_entities {
                label = label.with_related_entity(entity);
            }

            // Add metadata
            for (key, value) in &result.metadata {
                label = label.with_metadata(key, value);
            }

            // Assign cluster and update causal reason if in cluster
            if let Some(cluster_id) =
                self.cluster_manager
                    .assign_cluster(entry.posting_date(), &type_name, &mut self.rng)
            {
                label = label.with_cluster(&cluster_id);
                // Update causal reason to reflect cluster membership
                label = label.with_causal_reason(AnomalyCausalReason::ClusterMembership {
                    cluster_id: cluster_id.clone(),
                });
            }

            return Some(label);
        }

        None
    }

    /// Injects a specific anomaly type into an entry.
    pub fn inject_specific(
        &mut self,
        entry: &mut JournalEntry,
        anomaly_type: AnomalyType,
    ) -> Option<LabeledAnomaly> {
        self.inject_anomaly(entry, anomaly_type)
    }

    /// Creates a self-approval anomaly.
    pub fn create_self_approval(
        &mut self,
        entry: &mut JournalEntry,
        user_id: &str,
    ) -> Option<LabeledAnomaly> {
        let anomaly_type = AnomalyType::Fraud(FraudType::SelfApproval);

        let label = LabeledAnomaly::new(
            format!("ANO{:08}", self.labels.len() + 1),
            anomaly_type,
            entry.document_number().clone(),
            "JE".to_string(),
            entry.company_code().to_string(),
            entry.posting_date(),
        )
        .with_description(&format!("User {} approved their own transaction", user_id))
        .with_related_entity(user_id)
        .with_injection_strategy("ManualSelfApproval")
        .with_causal_reason(AnomalyCausalReason::EntityTargeting {
            target_type: "User".to_string(),
            target_id: user_id.to_string(),
        });

        // Set entry header anomaly tracking fields
        entry.header.is_anomaly = true;
        entry.header.is_fraud = true;
        entry.header.anomaly_id = Some(label.anomaly_id.clone());
        entry.header.anomaly_type = Some("SelfApproval".to_string());
        entry.header.fraud_type = Some(FraudType::SelfApproval);

        // Set approver = requester
        entry.header.created_by = user_id.to_string();

        self.labels.push(label.clone());
        Some(label)
    }

    /// Creates a segregation of duties violation.
    pub fn create_sod_violation(
        &mut self,
        entry: &mut JournalEntry,
        user_id: &str,
        conflicting_duties: (&str, &str),
    ) -> Option<LabeledAnomaly> {
        let anomaly_type = AnomalyType::Fraud(FraudType::SegregationOfDutiesViolation);

        let label = LabeledAnomaly::new(
            format!("ANO{:08}", self.labels.len() + 1),
            anomaly_type,
            entry.document_number().clone(),
            "JE".to_string(),
            entry.company_code().to_string(),
            entry.posting_date(),
        )
        .with_description(&format!(
            "User {} performed conflicting duties: {} and {}",
            user_id, conflicting_duties.0, conflicting_duties.1
        ))
        .with_related_entity(user_id)
        .with_metadata("duty1", conflicting_duties.0)
        .with_metadata("duty2", conflicting_duties.1)
        .with_injection_strategy("ManualSoDViolation")
        .with_causal_reason(AnomalyCausalReason::EntityTargeting {
            target_type: "User".to_string(),
            target_id: user_id.to_string(),
        });

        // Set entry header anomaly tracking fields
        entry.header.is_anomaly = true;
        entry.header.is_fraud = true;
        entry.header.anomaly_id = Some(label.anomaly_id.clone());
        entry.header.anomaly_type = Some("SegregationOfDutiesViolation".to_string());
        entry.header.fraud_type = Some(FraudType::SegregationOfDutiesViolation);

        self.labels.push(label.clone());
        Some(label)
    }

    /// Creates an intercompany mismatch anomaly.
    pub fn create_ic_mismatch(
        &mut self,
        entry: &mut JournalEntry,
        matching_company: &str,
        expected_amount: Decimal,
        actual_amount: Decimal,
    ) -> Option<LabeledAnomaly> {
        let anomaly_type = AnomalyType::Relational(RelationalAnomalyType::UnmatchedIntercompany);

        let label = LabeledAnomaly::new(
            format!("ANO{:08}", self.labels.len() + 1),
            anomaly_type,
            entry.document_number().clone(),
            "JE".to_string(),
            entry.company_code().to_string(),
            entry.posting_date(),
        )
        .with_description(&format!(
            "Intercompany mismatch with {}: expected {} but got {}",
            matching_company, expected_amount, actual_amount
        ))
        .with_related_entity(matching_company)
        .with_monetary_impact(actual_amount - expected_amount)
        .with_metadata("expected_amount", &expected_amount.to_string())
        .with_metadata("actual_amount", &actual_amount.to_string())
        .with_injection_strategy("ManualICMismatch")
        .with_causal_reason(AnomalyCausalReason::EntityTargeting {
            target_type: "Intercompany".to_string(),
            target_id: matching_company.to_string(),
        });

        // Set entry header anomaly tracking fields
        entry.header.is_anomaly = true;
        entry.header.anomaly_id = Some(label.anomaly_id.clone());
        entry.header.anomaly_type = Some("UnmatchedIntercompany".to_string());

        self.labels.push(label.clone());
        Some(label)
    }

    /// Returns all generated labels.
    pub fn get_labels(&self) -> &[LabeledAnomaly] {
        &self.labels
    }

    /// Returns the anomaly summary.
    pub fn get_summary(&self) -> AnomalySummary {
        AnomalySummary::from_anomalies(&self.labels)
    }

    /// Returns injection statistics.
    pub fn get_stats(&self) -> &InjectorStats {
        &self.stats
    }

    /// Clears all labels and resets statistics.
    pub fn reset(&mut self) {
        self.labels.clear();
        self.document_anomaly_counts.clear();
        self.stats = InjectorStats::default();
        self.cluster_manager = ClusterManager::new(self.config.patterns.clustering.clone());
    }

    /// Returns the number of clusters created.
    pub fn cluster_count(&self) -> usize {
        self.cluster_manager.cluster_count()
    }
}

/// Builder for AnomalyInjectorConfig.
pub struct AnomalyInjectorConfigBuilder {
    config: AnomalyInjectorConfig,
}

impl AnomalyInjectorConfigBuilder {
    /// Creates a new builder with default configuration.
    pub fn new() -> Self {
        Self {
            config: AnomalyInjectorConfig::default(),
        }
    }

    /// Sets the total anomaly rate.
    pub fn with_total_rate(mut self, rate: f64) -> Self {
        self.config.rates.total_rate = rate;
        self
    }

    /// Sets the fraud rate (proportion of anomalies).
    pub fn with_fraud_rate(mut self, rate: f64) -> Self {
        self.config.rates.fraud_rate = rate;
        self
    }

    /// Sets the error rate (proportion of anomalies).
    pub fn with_error_rate(mut self, rate: f64) -> Self {
        self.config.rates.error_rate = rate;
        self
    }

    /// Sets the random seed.
    pub fn with_seed(mut self, seed: u64) -> Self {
        self.config.seed = seed;
        self
    }

    /// Sets the temporal pattern.
    pub fn with_temporal_pattern(mut self, pattern: TemporalPattern) -> Self {
        self.config.patterns.temporal_pattern = pattern;
        self
    }

    /// Enables or disables label generation.
    pub fn with_labels(mut self, generate: bool) -> Self {
        self.config.generate_labels = generate;
        self
    }

    /// Sets target companies.
    pub fn with_target_companies(mut self, companies: Vec<String>) -> Self {
        self.config.target_companies = companies;
        self
    }

    /// Sets the date range.
    pub fn with_date_range(mut self, start: NaiveDate, end: NaiveDate) -> Self {
        self.config.date_range = Some((start, end));
        self
    }

    /// Builds the configuration.
    pub fn build(self) -> AnomalyInjectorConfig {
        self.config
    }
}

impl Default for AnomalyInjectorConfigBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;
    use datasynth_core::models::{JournalEntryLine, StatisticalAnomalyType};
    use rust_decimal_macros::dec;

    fn create_test_entry(doc_num: &str) -> JournalEntry {
        let mut entry = JournalEntry::new_simple(
            doc_num.to_string(),
            "1000".to_string(),
            NaiveDate::from_ymd_opt(2024, 6, 15).unwrap(),
            "Test Entry".to_string(),
        );

        entry.add_line(JournalEntryLine {
            line_number: 1,
            gl_account: "5000".to_string(),
            debit_amount: dec!(1000),
            ..Default::default()
        });

        entry.add_line(JournalEntryLine {
            line_number: 2,
            gl_account: "1000".to_string(),
            credit_amount: dec!(1000),
            ..Default::default()
        });

        entry
    }

    #[test]
    fn test_anomaly_injector_basic() {
        let config = AnomalyInjectorConfigBuilder::new()
            .with_total_rate(0.5) // High rate for testing
            .with_seed(42)
            .build();

        let mut injector = AnomalyInjector::new(config);

        let mut entries: Vec<_> = (0..100)
            .map(|i| create_test_entry(&format!("JE{:04}", i)))
            .collect();

        let result = injector.process_entries(&mut entries);

        // With 50% rate, we should have some anomalies
        assert!(result.anomalies_injected > 0);
        assert!(!result.labels.is_empty());
        assert_eq!(result.labels.len(), result.anomalies_injected);
    }

    #[test]
    fn test_specific_injection() {
        let config = AnomalyInjectorConfig::default();
        let mut injector = AnomalyInjector::new(config);

        let mut entry = create_test_entry("JE001");
        let anomaly_type = AnomalyType::Statistical(StatisticalAnomalyType::UnusuallyHighAmount);

        let label = injector.inject_specific(&mut entry, anomaly_type);

        assert!(label.is_some());
        let label = label.unwrap();
        // document_id is the UUID string from the journal entry header
        assert!(!label.document_id.is_empty());
        assert_eq!(label.document_id, entry.document_number());
    }

    #[test]
    fn test_self_approval_injection() {
        let config = AnomalyInjectorConfig::default();
        let mut injector = AnomalyInjector::new(config);

        let mut entry = create_test_entry("JE001");
        let label = injector.create_self_approval(&mut entry, "USER001");

        assert!(label.is_some());
        let label = label.unwrap();
        assert!(matches!(
            label.anomaly_type,
            AnomalyType::Fraud(FraudType::SelfApproval)
        ));
        assert!(label.related_entities.contains(&"USER001".to_string()));
    }

    #[test]
    fn test_company_filtering() {
        let config = AnomalyInjectorConfigBuilder::new()
            .with_total_rate(1.0) // Inject all
            .with_target_companies(vec!["2000".to_string()])
            .build();

        let mut injector = AnomalyInjector::new(config);

        let mut entries = vec![
            create_test_entry("JE001"), // company 1000
            create_test_entry("JE002"), // company 1000
        ];

        let result = injector.process_entries(&mut entries);

        // No anomalies because entries are in company 1000, not 2000
        assert_eq!(result.anomalies_injected, 0);
    }
}
