//! Journal Entry generator with statistical distributions.

use chrono::NaiveDate;
use rand::prelude::*;
use rand_chacha::ChaCha8Rng;
use std::sync::Arc;
use uuid::Uuid;

use synth_config::schema::TransactionConfig;
use synth_core::distributions::*;
use synth_core::models::*;
use synth_core::traits::Generator;

/// Generator for realistic journal entries.
pub struct JournalEntryGenerator {
    rng: ChaCha8Rng,
    seed: u64,
    config: TransactionConfig,
    coa: Arc<ChartOfAccounts>,
    companies: Vec<String>,
    line_sampler: LineItemSampler,
    amount_sampler: AmountSampler,
    temporal_sampler: TemporalSampler,
    start_date: NaiveDate,
    end_date: NaiveDate,
    count: u64,
    doc_counter: u64,
}

impl JournalEntryGenerator {
    /// Create a new journal entry generator.
    pub fn new_with_params(
        config: TransactionConfig,
        coa: Arc<ChartOfAccounts>,
        companies: Vec<String>,
        start_date: NaiveDate,
        end_date: NaiveDate,
        seed: u64,
    ) -> Self {
        Self {
            rng: ChaCha8Rng::seed_from_u64(seed),
            seed,
            config: config.clone(),
            coa,
            companies,
            line_sampler: LineItemSampler::with_config(
                seed + 1,
                config.line_item_distribution.clone(),
                config.even_odd_distribution.clone(),
                config.debit_credit_distribution.clone(),
            ),
            amount_sampler: AmountSampler::with_config(seed + 2, config.amounts.clone()),
            temporal_sampler: TemporalSampler::with_config(
                seed + 3,
                config.seasonality.clone(),
                WorkingHoursConfig::default(),
                Vec::new(),
            ),
            start_date,
            end_date,
            count: 0,
            doc_counter: 0,
        }
    }

    /// Generate a deterministic UUID from seed and counter.
    fn generate_deterministic_uuid(&self) -> Uuid {
        // Create a deterministic UUID v4-format from seed and counter
        // using the seed and counter to generate the UUID bytes
        let mut bytes = [0u8; 16];
        let seed_bytes = self.seed.to_le_bytes();
        let counter_bytes = self.doc_counter.to_le_bytes();

        // Mix seed and counter into the UUID bytes
        bytes[0..8].copy_from_slice(&seed_bytes);
        bytes[8..16].copy_from_slice(&counter_bytes);

        // Set version to 4 and variant bits properly
        bytes[6] = (bytes[6] & 0x0f) | 0x40; // Version 4
        bytes[8] = (bytes[8] & 0x3f) | 0x80; // Variant 1

        Uuid::from_bytes(bytes)
    }

    /// Generate a single journal entry.
    pub fn generate(&mut self) -> JournalEntry {
        self.count += 1;
        self.doc_counter += 1;

        // Generate deterministic document ID
        let document_id = self.generate_deterministic_uuid();

        // Sample posting date
        let posting_date = self.temporal_sampler.sample_date(self.start_date, self.end_date);

        // Select company
        let company_code = self.companies
            .choose(&mut self.rng)
            .cloned()
            .unwrap_or_else(|| "1000".to_string());

        // Sample line item specification
        let line_spec = self.line_sampler.sample();

        // Determine source type
        let is_automated = self.rng.gen::<f64>() < self.config.source_distribution.automated;
        let source = if is_automated {
            TransactionSource::Automated
        } else {
            TransactionSource::Manual
        };

        // Sample time based on source
        let time = self.temporal_sampler.sample_time(!is_automated);
        let created_at = posting_date.and_time(time).and_utc();

        // Create header with deterministic UUID
        let mut header = JournalEntryHeader::with_deterministic_id(company_code, posting_date, document_id);
        header.created_at = created_at;
        header.source = source;
        header.created_by = if is_automated {
            format!("BATCH{:04}", self.rng.gen_range(1..=20))
        } else {
            format!("USER{:04}", self.rng.gen_range(1..=40))
        };
        header.user_persona = if is_automated {
            "automated_system".to_string()
        } else {
            "senior_accountant".to_string()
        };

        // Generate line items
        let mut entry = JournalEntry::new(header);
        let total_amount = self.amount_sampler.sample();

        // Generate debit lines
        let debit_amounts = self.amount_sampler.sample_summing_to(line_spec.debit_count, total_amount);
        for (i, amount) in debit_amounts.into_iter().enumerate() {
            let account = self.select_debit_account();
            entry.add_line(JournalEntryLine::debit(
                entry.header.document_id,
                (i + 1) as u32,
                account.account_number.clone(),
                amount,
            ));
        }

        // Generate credit lines - use the SAME amounts to ensure balance
        let credit_amounts = self.amount_sampler.sample_summing_to(line_spec.credit_count, total_amount);
        for (i, amount) in credit_amounts.into_iter().enumerate() {
            let account = self.select_credit_account();
            entry.add_line(JournalEntryLine::credit(
                entry.header.document_id,
                (line_spec.debit_count + i + 1) as u32,
                account.account_number.clone(),
                amount,
            ));
        }

        entry
    }

    fn select_debit_account(&mut self) -> &GLAccount {
        let accounts = self.coa.get_accounts_by_type(AccountType::Asset);
        let expense_accounts = self.coa.get_accounts_by_type(AccountType::Expense);

        // 60% asset, 40% expense for debits
        let all: Vec<_> = if self.rng.gen::<f64>() < 0.6 {
            accounts
        } else {
            expense_accounts
        };

        all.choose(&mut self.rng)
            .copied()
            .unwrap_or_else(|| &self.coa.accounts[0])
    }

    fn select_credit_account(&mut self) -> &GLAccount {
        let liability_accounts = self.coa.get_accounts_by_type(AccountType::Liability);
        let revenue_accounts = self.coa.get_accounts_by_type(AccountType::Revenue);

        // 60% liability, 40% revenue for credits
        let all: Vec<_> = if self.rng.gen::<f64>() < 0.6 {
            liability_accounts
        } else {
            revenue_accounts
        };

        all.choose(&mut self.rng)
            .copied()
            .unwrap_or_else(|| &self.coa.accounts[0])
    }
}

impl Generator for JournalEntryGenerator {
    type Item = JournalEntry;
    type Config = (TransactionConfig, Arc<ChartOfAccounts>, Vec<String>, NaiveDate, NaiveDate);

    fn new(config: Self::Config, seed: u64) -> Self {
        Self::new_with_params(config.0, config.1, config.2, config.3, config.4, seed)
    }

    fn generate_one(&mut self) -> Self::Item {
        self.generate()
    }

    fn reset(&mut self) {
        self.rng = ChaCha8Rng::seed_from_u64(self.seed);
        self.line_sampler.reset(self.seed + 1);
        self.amount_sampler.reset(self.seed + 2);
        self.temporal_sampler.reset(self.seed + 3);
        self.count = 0;
        self.doc_counter = 0;
    }

    fn count(&self) -> u64 {
        self.count
    }

    fn seed(&self) -> u64 {
        self.seed
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ChartOfAccountsGenerator;

    #[test]
    fn test_generate_balanced_entries() {
        let mut coa_gen = ChartOfAccountsGenerator::new(
            CoAComplexity::Small,
            IndustrySector::Manufacturing,
            42,
        );
        let coa = Arc::new(coa_gen.generate());

        let mut je_gen = JournalEntryGenerator::new_with_params(
            TransactionConfig::default(),
            coa,
            vec!["1000".to_string()],
            NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            NaiveDate::from_ymd_opt(2024, 12, 31).unwrap(),
            42,
        );

        for _ in 0..100 {
            let entry = je_gen.generate();
            assert!(entry.is_balanced(), "Entry {:?} is not balanced", entry.header.document_id);
            assert!(entry.line_count() >= 2, "Entry has fewer than 2 lines");
        }
    }

    #[test]
    fn test_deterministic_generation() {
        let mut coa_gen = ChartOfAccountsGenerator::new(
            CoAComplexity::Small,
            IndustrySector::Manufacturing,
            42,
        );
        let coa = Arc::new(coa_gen.generate());

        let mut gen1 = JournalEntryGenerator::new_with_params(
            TransactionConfig::default(),
            Arc::clone(&coa),
            vec!["1000".to_string()],
            NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            NaiveDate::from_ymd_opt(2024, 12, 31).unwrap(),
            42,
        );

        let mut gen2 = JournalEntryGenerator::new_with_params(
            TransactionConfig::default(),
            coa,
            vec!["1000".to_string()],
            NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            NaiveDate::from_ymd_opt(2024, 12, 31).unwrap(),
            42,
        );

        for _ in 0..50 {
            let e1 = gen1.generate();
            let e2 = gen2.generate();
            assert_eq!(e1.header.document_id, e2.header.document_id);
            assert_eq!(e1.total_debit(), e2.total_debit());
        }
    }
}
