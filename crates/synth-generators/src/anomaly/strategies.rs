//! Injection strategies for anomaly generation.
//!
//! Strategies determine how anomalies are applied to existing data.

use rand::Rng;
use rust_decimal::Decimal;

use synth_core::models::{
    AnomalyType, ErrorType, FraudType, JournalEntry, ProcessIssueType, StatisticalAnomalyType,
};

/// Base trait for injection strategies.
pub trait InjectionStrategy {
    /// Name of the strategy.
    fn name(&self) -> &'static str;

    /// Whether this strategy can be applied to the given entry.
    fn can_apply(&self, entry: &JournalEntry) -> bool;

    /// Applies the strategy to modify an entry.
    fn apply<R: Rng>(
        &self,
        entry: &mut JournalEntry,
        anomaly_type: &AnomalyType,
        rng: &mut R,
    ) -> InjectionResult;
}

/// Result of an injection attempt.
#[derive(Debug, Clone)]
pub struct InjectionResult {
    /// Whether the injection was successful.
    pub success: bool,
    /// Description of what was modified.
    pub description: String,
    /// Monetary impact of the anomaly.
    pub monetary_impact: Option<Decimal>,
    /// Related entity IDs.
    pub related_entities: Vec<String>,
    /// Additional metadata.
    pub metadata: Vec<(String, String)>,
}

impl InjectionResult {
    /// Creates a successful result.
    pub fn success(description: &str) -> Self {
        Self {
            success: true,
            description: description.to_string(),
            monetary_impact: None,
            related_entities: Vec::new(),
            metadata: Vec::new(),
        }
    }

    /// Creates a failed result.
    pub fn failure(reason: &str) -> Self {
        Self {
            success: false,
            description: reason.to_string(),
            monetary_impact: None,
            related_entities: Vec::new(),
            metadata: Vec::new(),
        }
    }

    /// Adds monetary impact.
    pub fn with_impact(mut self, impact: Decimal) -> Self {
        self.monetary_impact = Some(impact);
        self
    }

    /// Adds a related entity.
    pub fn with_entity(mut self, entity: &str) -> Self {
        self.related_entities.push(entity.to_string());
        self
    }

    /// Adds metadata.
    pub fn with_metadata(mut self, key: &str, value: &str) -> Self {
        self.metadata.push((key.to_string(), value.to_string()));
        self
    }
}

/// Strategy for modifying amounts.
pub struct AmountModificationStrategy {
    /// Minimum multiplier for amount changes.
    pub min_multiplier: f64,
    /// Maximum multiplier for amount changes.
    pub max_multiplier: f64,
    /// Whether to use round numbers.
    pub prefer_round_numbers: bool,
}

impl Default for AmountModificationStrategy {
    fn default() -> Self {
        Self {
            min_multiplier: 2.0,
            max_multiplier: 10.0,
            prefer_round_numbers: false,
        }
    }
}

impl InjectionStrategy for AmountModificationStrategy {
    fn name(&self) -> &'static str {
        "AmountModification"
    }

    fn can_apply(&self, entry: &JournalEntry) -> bool {
        !entry.lines.is_empty()
    }

    fn apply<R: Rng>(
        &self,
        entry: &mut JournalEntry,
        anomaly_type: &AnomalyType,
        rng: &mut R,
    ) -> InjectionResult {
        if entry.lines.is_empty() {
            return InjectionResult::failure("No lines to modify");
        }

        let line_idx = rng.gen_range(0..entry.lines.len());
        let line = &mut entry.lines[line_idx];

        let original_amount = if line.debit_amount > Decimal::ZERO {
            line.debit_amount
        } else {
            line.credit_amount
        };

        let multiplier = rng.gen_range(self.min_multiplier..self.max_multiplier);
        let mut new_amount =
            original_amount * Decimal::from_f64_retain(multiplier).unwrap_or(Decimal::ONE);

        // Round to nice number if preferred
        if self.prefer_round_numbers {
            let magnitude = new_amount.to_string().len() as i32 - 2;
            let round_factor = Decimal::new(10_i64.pow(magnitude.max(0) as u32), 0);
            new_amount = (new_amount / round_factor).round() * round_factor;
        }

        let impact = new_amount - original_amount;

        if line.debit_amount > Decimal::ZERO {
            line.debit_amount = new_amount;
        } else {
            line.credit_amount = new_amount;
        }

        // Entry is now unbalanced - this is intentional for some anomaly types
        match anomaly_type {
            AnomalyType::Fraud(FraudType::RoundDollarManipulation) => {
                InjectionResult::success(&format!(
                    "Modified amount from {} to {} (round dollar)",
                    original_amount, new_amount
                ))
                .with_impact(impact)
                .with_entity(&line.account_code)
            }
            AnomalyType::Statistical(StatisticalAnomalyType::UnusuallyHighAmount) => {
                InjectionResult::success(&format!(
                    "Inflated amount by {:.1}x to {}",
                    multiplier, new_amount
                ))
                .with_impact(impact)
                .with_metadata("multiplier", &format!("{:.2}", multiplier))
            }
            _ => InjectionResult::success(&format!("Modified amount to {}", new_amount))
                .with_impact(impact),
        }
    }
}

/// Strategy for modifying dates.
pub struct DateModificationStrategy {
    /// Maximum days to backdate.
    pub max_backdate_days: i64,
    /// Maximum days to future-date.
    pub max_future_days: i64,
    /// Whether to cross period boundaries.
    pub cross_period_boundary: bool,
}

impl Default for DateModificationStrategy {
    fn default() -> Self {
        Self {
            max_backdate_days: 30,
            max_future_days: 7,
            cross_period_boundary: true,
        }
    }
}

impl InjectionStrategy for DateModificationStrategy {
    fn name(&self) -> &'static str {
        "DateModification"
    }

    fn can_apply(&self, _entry: &JournalEntry) -> bool {
        true
    }

    fn apply<R: Rng>(
        &self,
        entry: &mut JournalEntry,
        anomaly_type: &AnomalyType,
        rng: &mut R,
    ) -> InjectionResult {
        let original_date = entry.header.posting_date;

        let (days_offset, description) = match anomaly_type {
            AnomalyType::Error(ErrorType::BackdatedEntry) => {
                let days = rng.gen_range(1..=self.max_backdate_days);
                (-days, format!("Backdated by {} days", days))
            }
            AnomalyType::Error(ErrorType::FutureDatedEntry) => {
                let days = rng.gen_range(1..=self.max_future_days);
                (days, format!("Future-dated by {} days", days))
            }
            AnomalyType::Error(ErrorType::WrongPeriod) => {
                // Move to previous or next month
                let direction: i64 = if rng.gen_bool(0.5) { -1 } else { 1 };
                let days = direction * 32; // Ensure crossing month boundary
                (days, "Posted to wrong period".to_string())
            }
            AnomalyType::ProcessIssue(ProcessIssueType::LatePosting) => {
                let days = rng.gen_range(5..=15);
                entry.header.document_date = entry.header.posting_date; // Document date stays same
                entry.header.posting_date = original_date + chrono::Duration::days(days);
                return InjectionResult::success(&format!(
                    "Late posting: {} days after transaction",
                    days
                ))
                .with_metadata("delay_days", &days.to_string());
            }
            _ => (0, "Date unchanged".to_string()),
        };

        if days_offset != 0 {
            entry.header.posting_date = original_date + chrono::Duration::days(days_offset);
        }

        InjectionResult::success(&description)
            .with_metadata("original_date", &original_date.to_string())
            .with_metadata("new_date", &entry.header.posting_date.to_string())
    }
}

/// Strategy for document duplication.
pub struct DuplicationStrategy {
    /// Whether to modify amounts slightly.
    pub vary_amounts: bool,
    /// Amount variance factor.
    pub amount_variance: f64,
    /// Whether to change document numbers.
    pub change_doc_number: bool,
}

impl Default for DuplicationStrategy {
    fn default() -> Self {
        Self {
            vary_amounts: false,
            amount_variance: 0.01,
            change_doc_number: true,
        }
    }
}

impl DuplicationStrategy {
    /// Creates a duplicate of the entry.
    pub fn duplicate<R: Rng>(&self, entry: &JournalEntry, rng: &mut R) -> JournalEntry {
        let mut duplicate = entry.clone();

        if self.change_doc_number {
            // Generate a new UUID for the duplicate
            duplicate.header.document_id = uuid::Uuid::new_v4();
            // Update line items to reference the new document ID
            for line in &mut duplicate.lines {
                line.document_id = duplicate.header.document_id;
            }
        }

        if self.vary_amounts {
            for line in &mut duplicate.lines {
                let variance = 1.0 + rng.gen_range(-self.amount_variance..self.amount_variance);
                let variance_dec = Decimal::from_f64_retain(variance).unwrap_or(Decimal::ONE);

                if line.debit_amount > Decimal::ZERO {
                    line.debit_amount = (line.debit_amount * variance_dec).round_dp(2);
                }
                if line.credit_amount > Decimal::ZERO {
                    line.credit_amount = (line.credit_amount * variance_dec).round_dp(2);
                }
            }
        }

        duplicate
    }
}

/// Strategy for approval-related anomalies.
pub struct ApprovalAnomalyStrategy {
    /// Approval threshold to target.
    pub approval_threshold: Decimal,
    /// Buffer below threshold.
    pub threshold_buffer: Decimal,
}

impl Default for ApprovalAnomalyStrategy {
    fn default() -> Self {
        Self {
            approval_threshold: Decimal::new(10000, 0),
            threshold_buffer: Decimal::new(100, 0),
        }
    }
}

impl InjectionStrategy for ApprovalAnomalyStrategy {
    fn name(&self) -> &'static str {
        "ApprovalAnomaly"
    }

    fn can_apply(&self, entry: &JournalEntry) -> bool {
        entry.total_debit() > Decimal::ZERO
    }

    fn apply<R: Rng>(
        &self,
        entry: &mut JournalEntry,
        anomaly_type: &AnomalyType,
        rng: &mut R,
    ) -> InjectionResult {
        match anomaly_type {
            AnomalyType::Fraud(FraudType::JustBelowThreshold) => {
                // Set total to just below threshold
                let target = self.approval_threshold
                    - self.threshold_buffer
                    - Decimal::new(rng.gen_range(1..50), 0);

                let current_total = entry.total_debit();
                if current_total == Decimal::ZERO {
                    return InjectionResult::failure("Cannot scale zero amount");
                }

                let scale = target / current_total;
                for line in &mut entry.lines {
                    line.debit_amount = (line.debit_amount * scale).round_dp(2);
                    line.credit_amount = (line.credit_amount * scale).round_dp(2);
                }

                InjectionResult::success(&format!(
                    "Adjusted total to {} (just below threshold {})",
                    entry.total_debit(),
                    self.approval_threshold
                ))
                .with_metadata("threshold", &self.approval_threshold.to_string())
            }
            AnomalyType::Fraud(FraudType::ExceededApprovalLimit) => {
                // Set total to exceed threshold
                let target = self.approval_threshold * Decimal::new(15, 1); // 1.5x threshold

                let current_total = entry.total_debit();
                if current_total == Decimal::ZERO {
                    return InjectionResult::failure("Cannot scale zero amount");
                }

                let scale = target / current_total;
                for line in &mut entry.lines {
                    line.debit_amount = (line.debit_amount * scale).round_dp(2);
                    line.credit_amount = (line.credit_amount * scale).round_dp(2);
                }

                InjectionResult::success(&format!(
                    "Exceeded approval limit: {} vs limit {}",
                    entry.total_debit(),
                    self.approval_threshold
                ))
                .with_impact(entry.total_debit() - self.approval_threshold)
            }
            _ => InjectionResult::failure("Unsupported anomaly type for this strategy"),
        }
    }
}

/// Strategy for description/text anomalies.
pub struct DescriptionAnomalyStrategy {
    /// Vague descriptions to use.
    pub vague_descriptions: Vec<String>,
}

impl Default for DescriptionAnomalyStrategy {
    fn default() -> Self {
        Self {
            vague_descriptions: vec![
                "Misc".to_string(),
                "Adjustment".to_string(),
                "Correction".to_string(),
                "Various".to_string(),
                "Other".to_string(),
                "TBD".to_string(),
                "See attachment".to_string(),
                "As discussed".to_string(),
                "Per management".to_string(),
                ".".to_string(),
                "xxx".to_string(),
                "test".to_string(),
            ],
        }
    }
}

impl InjectionStrategy for DescriptionAnomalyStrategy {
    fn name(&self) -> &'static str {
        "DescriptionAnomaly"
    }

    fn can_apply(&self, _entry: &JournalEntry) -> bool {
        true
    }

    fn apply<R: Rng>(
        &self,
        entry: &mut JournalEntry,
        _anomaly_type: &AnomalyType,
        rng: &mut R,
    ) -> InjectionResult {
        let original = entry.description().unwrap_or("").to_string();
        let vague = &self.vague_descriptions[rng.gen_range(0..self.vague_descriptions.len())];
        entry.set_description(vague.clone());

        InjectionResult::success(&format!(
            "Changed description from '{}' to '{}'",
            original, vague
        ))
        .with_metadata("original_description", &original)
    }
}

/// Strategy for Benford's Law violations.
pub struct BenfordViolationStrategy {
    /// Target first digits (rarely occurring).
    pub target_digits: Vec<u32>,
}

impl Default for BenfordViolationStrategy {
    fn default() -> Self {
        Self {
            target_digits: vec![5, 6, 7, 8, 9], // Less common first digits
        }
    }
}

impl InjectionStrategy for BenfordViolationStrategy {
    fn name(&self) -> &'static str {
        "BenfordViolation"
    }

    fn can_apply(&self, entry: &JournalEntry) -> bool {
        !entry.lines.is_empty()
    }

    fn apply<R: Rng>(
        &self,
        entry: &mut JournalEntry,
        _anomaly_type: &AnomalyType,
        rng: &mut R,
    ) -> InjectionResult {
        if entry.lines.is_empty() {
            return InjectionResult::failure("No lines to modify");
        }

        let line_idx = rng.gen_range(0..entry.lines.len());
        let line = &mut entry.lines[line_idx];

        let original_amount = if line.debit_amount > Decimal::ZERO {
            line.debit_amount
        } else {
            line.credit_amount
        };

        // Get target first digit
        let target_digit = self.target_digits[rng.gen_range(0..self.target_digits.len())];

        // Calculate new amount with target first digit
        let original_str = original_amount.to_string();
        let magnitude = original_str.replace('.', "").trim_start_matches('0').len() as i32 - 1;

        let base = Decimal::new(10_i64.pow(magnitude.max(0) as u32), 0);
        let new_amount = base * Decimal::new(target_digit as i64, 0)
            + Decimal::new(rng.gen_range(0..10_i64.pow(magnitude.max(0) as u32)), 0);

        if line.debit_amount > Decimal::ZERO {
            line.debit_amount = new_amount;
        } else {
            line.credit_amount = new_amount;
        }

        let first_digit = target_digit;
        let benford_prob = (1.0 + 1.0 / first_digit as f64).log10();

        InjectionResult::success(&format!(
            "Created Benford violation: first digit {} (expected probability {:.1}%)",
            first_digit,
            benford_prob * 100.0
        ))
        .with_impact(new_amount - original_amount)
        .with_metadata("first_digit", &first_digit.to_string())
        .with_metadata("benford_probability", &format!("{:.4}", benford_prob))
    }
}

/// Collection of all available strategies.
#[derive(Default)]
pub struct StrategyCollection {
    pub amount_modification: AmountModificationStrategy,
    pub date_modification: DateModificationStrategy,
    pub duplication: DuplicationStrategy,
    pub approval_anomaly: ApprovalAnomalyStrategy,
    pub description_anomaly: DescriptionAnomalyStrategy,
    pub benford_violation: BenfordViolationStrategy,
}

impl StrategyCollection {
    /// Checks if the strategy can be applied to an entry.
    pub fn can_apply(&self, entry: &JournalEntry, anomaly_type: &AnomalyType) -> bool {
        match anomaly_type {
            AnomalyType::Fraud(FraudType::RoundDollarManipulation)
            | AnomalyType::Statistical(StatisticalAnomalyType::UnusuallyHighAmount)
            | AnomalyType::Statistical(StatisticalAnomalyType::UnusuallyLowAmount) => {
                self.amount_modification.can_apply(entry)
            }
            AnomalyType::Error(ErrorType::BackdatedEntry)
            | AnomalyType::Error(ErrorType::FutureDatedEntry)
            | AnomalyType::Error(ErrorType::WrongPeriod)
            | AnomalyType::ProcessIssue(ProcessIssueType::LatePosting) => {
                self.date_modification.can_apply(entry)
            }
            AnomalyType::Fraud(FraudType::JustBelowThreshold)
            | AnomalyType::Fraud(FraudType::ExceededApprovalLimit) => {
                self.approval_anomaly.can_apply(entry)
            }
            AnomalyType::ProcessIssue(ProcessIssueType::VagueDescription) => {
                self.description_anomaly.can_apply(entry)
            }
            AnomalyType::Statistical(StatisticalAnomalyType::BenfordViolation) => {
                self.benford_violation.can_apply(entry)
            }
            _ => self.amount_modification.can_apply(entry),
        }
    }

    /// Applies the appropriate strategy for an anomaly type.
    pub fn apply_strategy<R: Rng>(
        &self,
        entry: &mut JournalEntry,
        anomaly_type: &AnomalyType,
        rng: &mut R,
    ) -> InjectionResult {
        match anomaly_type {
            AnomalyType::Fraud(FraudType::RoundDollarManipulation)
            | AnomalyType::Statistical(StatisticalAnomalyType::UnusuallyHighAmount)
            | AnomalyType::Statistical(StatisticalAnomalyType::UnusuallyLowAmount) => {
                self.amount_modification.apply(entry, anomaly_type, rng)
            }
            AnomalyType::Error(ErrorType::BackdatedEntry)
            | AnomalyType::Error(ErrorType::FutureDatedEntry)
            | AnomalyType::Error(ErrorType::WrongPeriod)
            | AnomalyType::ProcessIssue(ProcessIssueType::LatePosting) => {
                self.date_modification.apply(entry, anomaly_type, rng)
            }
            AnomalyType::Fraud(FraudType::JustBelowThreshold)
            | AnomalyType::Fraud(FraudType::ExceededApprovalLimit) => {
                self.approval_anomaly.apply(entry, anomaly_type, rng)
            }
            AnomalyType::ProcessIssue(ProcessIssueType::VagueDescription) => {
                self.description_anomaly.apply(entry, anomaly_type, rng)
            }
            AnomalyType::Statistical(StatisticalAnomalyType::BenfordViolation) => {
                self.benford_violation.apply(entry, anomaly_type, rng)
            }
            _ => self.amount_modification.apply(entry, anomaly_type, rng), // Default fallback
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;
    use rand::SeedableRng;
    use rand_chacha::ChaCha8Rng;
    use rust_decimal_macros::dec;
    use synth_core::models::JournalEntryLine;

    fn create_test_entry() -> JournalEntry {
        let mut entry = JournalEntry::new_simple(
            "JE001".to_string(),
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
    fn test_amount_modification() {
        let strategy = AmountModificationStrategy::default();
        let mut entry = create_test_entry();
        let mut rng = ChaCha8Rng::seed_from_u64(42);

        let result = strategy.apply(
            &mut entry,
            &AnomalyType::Statistical(StatisticalAnomalyType::UnusuallyHighAmount),
            &mut rng,
        );

        assert!(result.success);
        assert!(result.monetary_impact.is_some());
    }

    #[test]
    fn test_date_modification() {
        let strategy = DateModificationStrategy::default();
        let mut entry = create_test_entry();
        let original_date = entry.header.posting_date;
        let mut rng = ChaCha8Rng::seed_from_u64(42);

        let result = strategy.apply(
            &mut entry,
            &AnomalyType::Error(ErrorType::BackdatedEntry),
            &mut rng,
        );

        assert!(result.success);
        assert!(entry.header.posting_date < original_date);
    }

    #[test]
    fn test_description_anomaly() {
        let strategy = DescriptionAnomalyStrategy::default();
        let mut entry = create_test_entry();
        let mut rng = ChaCha8Rng::seed_from_u64(42);

        let result = strategy.apply(
            &mut entry,
            &AnomalyType::ProcessIssue(ProcessIssueType::VagueDescription),
            &mut rng,
        );

        assert!(result.success);
        let desc = entry.description().unwrap_or("").to_string();
        assert!(strategy.vague_descriptions.contains(&desc));
    }
}
