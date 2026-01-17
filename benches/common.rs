//! Shared fixtures and utilities for benchmarks.

use std::sync::Arc;

use chrono::NaiveDate;
use synth_config::presets;
use synth_config::schema::{GeneratorConfig, TransactionConfig};
use synth_core::models::{ChartOfAccounts, CoAComplexity, IndustrySector, JournalEntry};
use synth_generators::{ChartOfAccountsGenerator, JournalEntryGenerator};

/// Default seed for reproducible benchmarks.
pub const BENCHMARK_SEED: u64 = 12345;

/// Standard start date for benchmark periods.
pub fn start_date() -> NaiveDate {
    NaiveDate::from_ymd_opt(2024, 1, 1).unwrap()
}

/// Standard end date for benchmark periods (1 year).
pub fn end_date() -> NaiveDate {
    NaiveDate::from_ymd_opt(2024, 12, 31).unwrap()
}

/// Create a small chart of accounts for benchmarks.
pub fn small_coa() -> Arc<ChartOfAccounts> {
    let mut gen = ChartOfAccountsGenerator::new(
        CoAComplexity::Small,
        IndustrySector::Manufacturing,
        BENCHMARK_SEED,
    );
    Arc::new(gen.generate())
}

/// Create a medium chart of accounts for benchmarks.
pub fn medium_coa() -> Arc<ChartOfAccounts> {
    let mut gen = ChartOfAccountsGenerator::new(
        CoAComplexity::Medium,
        IndustrySector::Manufacturing,
        BENCHMARK_SEED,
    );
    Arc::new(gen.generate())
}

/// Create a large chart of accounts for benchmarks.
pub fn large_coa() -> Arc<ChartOfAccounts> {
    let mut gen = ChartOfAccountsGenerator::new(
        CoAComplexity::Large,
        IndustrySector::Manufacturing,
        BENCHMARK_SEED,
    );
    Arc::new(gen.generate())
}

/// Create a journal entry generator with default settings.
pub fn create_je_generator(coa: Arc<ChartOfAccounts>) -> JournalEntryGenerator {
    JournalEntryGenerator::new_with_params(
        TransactionConfig::default(),
        coa,
        vec!["1000".to_string()],
        start_date(),
        end_date(),
        BENCHMARK_SEED,
    )
    .with_persona_errors(false) // Disable for consistent benchmarks
    .with_approval(false) // Disable for consistent benchmarks
}

/// Create a journal entry generator with approval workflow enabled.
pub fn create_je_generator_with_approval(coa: Arc<ChartOfAccounts>) -> JournalEntryGenerator {
    JournalEntryGenerator::new_with_params(
        TransactionConfig::default(),
        coa,
        vec!["1000".to_string()],
        start_date(),
        end_date(),
        BENCHMARK_SEED,
    )
    .with_persona_errors(false)
    .with_approval(true)
}

/// Generate a batch of journal entries for output benchmarks.
pub fn generate_entries(count: usize) -> Vec<JournalEntry> {
    let coa = small_coa();
    let mut gen = create_je_generator(coa);
    (0..count).map(|_| gen.generate()).collect()
}

/// Get a demo generator config.
pub fn demo_config() -> GeneratorConfig {
    presets::demo_preset()
}

/// Get a stress test generator config.
pub fn stress_config() -> GeneratorConfig {
    presets::stress_test_preset()
}

/// Companies for multi-company benchmarks.
pub fn multi_company_codes() -> Vec<String> {
    vec![
        "1000".to_string(),
        "2000".to_string(),
        "3000".to_string(),
    ]
}
