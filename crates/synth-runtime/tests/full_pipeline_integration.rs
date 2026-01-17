//! Full pipeline integration tests.
//!
//! These tests exercise the complete generation pipeline with all phases enabled,
//! testing the interaction between master data, document flows, journal entries,
//! and anomaly injection.

use rust_decimal::Decimal;
use std::collections::HashSet;
use synth_config::schema::TransactionVolume;
use synth_runtime::{EnhancedOrchestrator, PhaseConfig};
use synth_test_utils::fixtures::{minimal_config, multi_company_config};

/// Test full pipeline with all phases enabled.
#[test]
fn test_full_pipeline_all_phases() {
    let mut config = minimal_config();
    config.global.seed = Some(12345);
    config.global.period_months = 3;
    config.companies[0].annual_transaction_volume = TransactionVolume::TenK;

    // Enable master data generation
    config.master_data.vendors.count = 10;
    config.master_data.customers.count = 10;
    config.master_data.materials.count = 20;
    config.master_data.fixed_assets.count = 5;
    config.master_data.employees.count = 5;

    // Enable document flows
    config.document_flows.p2p.enabled = true;
    config.document_flows.p2p.three_way_match_rate = 0.9;
    config.document_flows.o2c.enabled = true;
    config.document_flows.generate_document_references = true;

    let phase_config = PhaseConfig {
        generate_master_data: true,
        generate_document_flows: true,
        generate_journal_entries: true,
        inject_anomalies: false,
        show_progress: false,
        ..Default::default()
    };

    let mut orchestrator =
        EnhancedOrchestrator::new(config, phase_config).expect("Failed to create orchestrator");

    let result = orchestrator.generate().expect("Generation failed");

    // Verify journal entries were generated
    assert!(
        !result.journal_entries.is_empty(),
        "Should generate journal entries"
    );

    // Verify entries are balanced (excluding human errors)
    for entry in &result.journal_entries {
        if entry
            .header
            .header_text
            .as_ref()
            .map(|t| !t.contains("[HUMAN_ERROR:"))
            .unwrap_or(true)
        {
            let total_debits: Decimal = entry.lines.iter().map(|l| l.debit_amount).sum();
            let total_credits: Decimal = entry.lines.iter().map(|l| l.credit_amount).sum();
            assert_eq!(
                total_debits, total_credits,
                "Entry {} should be balanced",
                entry.header.document_id
            );
        }
    }

    println!(
        "Full pipeline test generated {} journal entries",
        result.journal_entries.len()
    );
}

/// Test that master data is consistent across the pipeline.
#[test]
fn test_master_data_consistency() {
    let mut config = minimal_config();
    config.global.seed = Some(54321);
    config.global.period_months = 1;

    config.master_data.vendors.count = 5;
    config.master_data.customers.count = 5;

    let phase_config = PhaseConfig {
        generate_master_data: true,
        generate_document_flows: false,
        generate_journal_entries: true,
        inject_anomalies: false,
        show_progress: false,
        ..Default::default()
    };

    let mut orchestrator =
        EnhancedOrchestrator::new(config, phase_config).expect("Failed to create orchestrator");

    let result = orchestrator.generate().expect("Generation failed");

    // Verify journal entries exist
    assert!(
        !result.journal_entries.is_empty(),
        "Should generate journal entries"
    );

    // All entries should have valid company codes
    for entry in &result.journal_entries {
        assert!(
            !entry.header.company_code.is_empty(),
            "Entry should have a company code"
        );
    }
}

/// Test multi-company intercompany transaction generation.
#[test]
fn test_multi_company_with_intercompany() {
    let mut config = multi_company_config();
    config.global.seed = Some(67890);
    config.global.period_months = 3;

    // Enable intercompany transactions
    config.intercompany.enabled = true;
    config.intercompany.ic_transaction_rate = 0.1;
    config.intercompany.generate_eliminations = true;
    config.intercompany.generate_matched_pairs = true;

    let phase_config = PhaseConfig {
        generate_master_data: false,
        generate_document_flows: false,
        generate_journal_entries: true,
        inject_anomalies: false,
        show_progress: false,
        ..Default::default()
    };

    let mut orchestrator = EnhancedOrchestrator::new(config.clone(), phase_config)
        .expect("Failed to create orchestrator");

    let result = orchestrator.generate().expect("Generation failed");

    // Collect company codes
    let company_codes: HashSet<_> = result
        .journal_entries
        .iter()
        .map(|e| e.header.company_code.as_str())
        .collect();

    println!(
        "Generated entries for {} companies: {:?}",
        company_codes.len(),
        company_codes
    );

    // Should generate entries (may be from one or more companies depending on random selection)
    assert!(
        !result.journal_entries.is_empty(),
        "Should generate entries for at least one company"
    );
}

/// Test document flow chain integrity.
#[test]
fn test_document_flow_chain_integrity() {
    let mut config = minimal_config();
    config.global.seed = Some(11111);
    config.global.period_months = 2;
    config.companies[0].annual_transaction_volume = TransactionVolume::TenK;

    config.document_flows.p2p.enabled = true;
    config.document_flows.p2p.three_way_match_rate = 0.95;
    config.document_flows.o2c.enabled = true;
    config.document_flows.generate_document_references = true;

    let phase_config = PhaseConfig {
        generate_master_data: true,
        generate_document_flows: true,
        generate_journal_entries: true,
        inject_anomalies: false,
        show_progress: false,
        ..Default::default()
    };

    let mut orchestrator =
        EnhancedOrchestrator::new(config, phase_config).expect("Failed to create orchestrator");

    let result = orchestrator.generate().expect("Generation failed");

    // Check that some entries have document type assignments
    let document_types: HashSet<_> = result
        .journal_entries
        .iter()
        .map(|e| e.header.document_type.as_str())
        .collect();

    println!(
        "Generated {} entries with document types: {:?}",
        result.journal_entries.len(),
        document_types
    );

    assert!(
        !result.journal_entries.is_empty(),
        "Should generate journal entries"
    );
}

/// Test anomaly injection integration.
#[test]
fn test_anomaly_injection_integration() {
    let mut config = minimal_config();
    config.global.seed = Some(99999);
    config.global.period_months = 2;
    config.companies[0].annual_transaction_volume = TransactionVolume::TenK;

    // Enable fraud detection scenarios
    config.fraud.enabled = true;
    config.fraud.fraud_rate = 0.05; // 5% fraud rate

    let phase_config = PhaseConfig {
        generate_master_data: false,
        generate_document_flows: false,
        generate_journal_entries: true,
        inject_anomalies: true,
        show_progress: false,
        ..Default::default()
    };

    let mut orchestrator =
        EnhancedOrchestrator::new(config, phase_config).expect("Failed to create orchestrator");

    let result = orchestrator.generate().expect("Generation failed");

    // Verify entries were generated
    assert!(
        !result.journal_entries.is_empty(),
        "Should generate journal entries"
    );

    // Count fraud entries
    let fraud_count = result
        .journal_entries
        .iter()
        .filter(|e| e.header.is_fraud)
        .count();

    println!(
        "Generated {} entries: {} fraud, {} labels",
        result.journal_entries.len(),
        fraud_count,
        result.anomaly_labels.labels.len()
    );
}

/// Test balance coherence across companies.
#[test]
fn test_balance_coherence() {
    let mut config = minimal_config();
    config.global.seed = Some(77777);
    config.global.period_months = 1;

    // Enable balance tracking
    config.balance.generate_opening_balances = true;
    config.balance.generate_trial_balances = true;

    let phase_config = PhaseConfig {
        generate_master_data: false,
        generate_document_flows: false,
        generate_journal_entries: true,
        inject_anomalies: false,
        show_progress: false,
        ..Default::default()
    };

    let mut orchestrator =
        EnhancedOrchestrator::new(config, phase_config).expect("Failed to create orchestrator");

    let result = orchestrator.generate().expect("Generation failed");

    // Verify all entries are balanced
    for entry in &result.journal_entries {
        // Skip entries marked as human errors
        if entry
            .header
            .header_text
            .as_ref()
            .map(|t| t.contains("[HUMAN_ERROR:"))
            .unwrap_or(false)
        {
            continue;
        }

        let total_debits: Decimal = entry.lines.iter().map(|l| l.debit_amount).sum();
        let total_credits: Decimal = entry.lines.iter().map(|l| l.credit_amount).sum();
        assert_eq!(
            total_debits, total_credits,
            "Entry {} should be balanced: debits={}, credits={}",
            entry.header.document_id, total_debits, total_credits
        );
    }
}

/// Test generation with internal controls.
#[test]
fn test_internal_controls_integration() {
    let mut config = minimal_config();
    config.global.seed = Some(33333);
    config.global.period_months = 1;

    // Enable internal controls
    config.internal_controls.enabled = true;
    config.internal_controls.exception_rate = 0.02;
    config.internal_controls.sod_violation_rate = 0.01;

    let phase_config = PhaseConfig {
        generate_master_data: false,
        generate_document_flows: false,
        generate_journal_entries: true,
        inject_anomalies: true,
        show_progress: false,
        ..Default::default()
    };

    let mut orchestrator =
        EnhancedOrchestrator::new(config, phase_config).expect("Failed to create orchestrator");

    let result = orchestrator.generate().expect("Generation failed");

    // Verify entries were generated
    assert!(
        !result.journal_entries.is_empty(),
        "Should generate journal entries"
    );

    // Count entries with control IDs
    let entries_with_controls = result
        .journal_entries
        .iter()
        .filter(|e| !e.header.control_ids.is_empty())
        .count();

    // Count SOD violations
    let sod_violations = result
        .journal_entries
        .iter()
        .filter(|e| e.header.sod_violation)
        .count();

    println!(
        "Generated {} entries: {} with controls, {} SOD violations",
        result.journal_entries.len(),
        entries_with_controls,
        sod_violations
    );
}

/// Test business process distribution.
#[test]
fn test_business_process_distribution() {
    let mut config = minimal_config();
    config.global.seed = Some(44444);
    config.global.period_months = 3;
    config.companies[0].annual_transaction_volume = TransactionVolume::TenK;

    let phase_config = PhaseConfig {
        generate_master_data: false,
        generate_document_flows: false,
        generate_journal_entries: true,
        inject_anomalies: false,
        show_progress: false,
        ..Default::default()
    };

    let mut orchestrator =
        EnhancedOrchestrator::new(config, phase_config).expect("Failed to create orchestrator");

    let result = orchestrator.generate().expect("Generation failed");

    // Count entries by business process
    let mut process_counts = std::collections::HashMap::new();
    for entry in &result.journal_entries {
        if let Some(process) = &entry.header.business_process {
            *process_counts.entry(format!("{:?}", process)).or_insert(0) += 1;
        }
    }

    println!("Business process distribution: {:?}", process_counts);

    // Should have some variety in business processes
    assert!(
        !process_counts.is_empty(),
        "Should have at least one business process"
    );
}

/// Test transaction source distribution.
#[test]
fn test_transaction_source_distribution() {
    let mut config = minimal_config();
    config.global.seed = Some(55555);
    config.global.period_months = 2;
    config.companies[0].annual_transaction_volume = TransactionVolume::TenK;

    let phase_config = PhaseConfig {
        generate_master_data: false,
        generate_document_flows: false,
        generate_journal_entries: true,
        inject_anomalies: false,
        show_progress: false,
        ..Default::default()
    };

    let mut orchestrator =
        EnhancedOrchestrator::new(config, phase_config).expect("Failed to create orchestrator");

    let result = orchestrator.generate().expect("Generation failed");

    // Count entries by source
    let mut source_counts = std::collections::HashMap::new();
    for entry in &result.journal_entries {
        *source_counts
            .entry(format!("{:?}", entry.header.source))
            .or_insert(0) += 1;
    }

    println!("Transaction source distribution: {:?}", source_counts);

    // Should have entries generated
    assert!(
        !result.journal_entries.is_empty(),
        "Should generate entries"
    );
}

/// Test large volume generation doesn't panic.
#[test]
fn test_large_volume_generation() {
    let mut config = minimal_config();
    config.global.seed = Some(66666);
    config.global.period_months = 6;
    config.companies[0].annual_transaction_volume = TransactionVolume::HundredK;

    let phase_config = PhaseConfig {
        generate_master_data: false,
        generate_document_flows: false,
        generate_journal_entries: true,
        inject_anomalies: false,
        show_progress: false,
        ..Default::default()
    };

    let mut orchestrator =
        EnhancedOrchestrator::new(config, phase_config).expect("Failed to create orchestrator");

    let result = orchestrator.generate().expect("Generation failed");

    // Should generate a significant number of entries
    assert!(
        result.journal_entries.len() >= 100,
        "Should generate at least 100 entries for 100K volume over 6 months, got {}",
        result.journal_entries.len()
    );

    println!(
        "Large volume test generated {} entries",
        result.journal_entries.len()
    );
}

/// Test currency handling across entries.
#[test]
fn test_currency_handling() {
    let mut config = multi_company_config();
    config.global.seed = Some(88888);
    config.global.period_months = 1;

    let phase_config = PhaseConfig {
        generate_master_data: false,
        generate_document_flows: false,
        generate_journal_entries: true,
        inject_anomalies: false,
        show_progress: false,
        ..Default::default()
    };

    let mut orchestrator = EnhancedOrchestrator::new(config.clone(), phase_config)
        .expect("Failed to create orchestrator");

    let result = orchestrator.generate().expect("Generation failed");

    // Collect currencies used
    let currencies: HashSet<_> = result
        .journal_entries
        .iter()
        .map(|e| e.header.currency.as_str())
        .collect();

    println!("Currencies used: {:?}", currencies);

    // All entries should have valid currency codes
    for entry in &result.journal_entries {
        assert!(
            entry.header.currency.len() == 3,
            "Currency code should be 3 characters: {}",
            entry.header.currency
        );
    }
}

/// Test fiscal year and period assignment.
#[test]
fn test_fiscal_year_assignment() {
    let mut config = minimal_config();
    config.global.seed = Some(11112);
    config.global.start_date = "2024-01-01".to_string();
    config.global.period_months = 6;

    let phase_config = PhaseConfig {
        generate_master_data: false,
        generate_document_flows: false,
        generate_journal_entries: true,
        inject_anomalies: false,
        show_progress: false,
        ..Default::default()
    };

    let mut orchestrator =
        EnhancedOrchestrator::new(config, phase_config).expect("Failed to create orchestrator");

    let result = orchestrator.generate().expect("Generation failed");

    // Verify entries were generated
    assert!(
        !result.journal_entries.is_empty(),
        "Should generate journal entries"
    );

    // Fiscal periods should be valid (1-12)
    let fiscal_periods: HashSet<_> = result
        .journal_entries
        .iter()
        .map(|e| e.header.fiscal_period)
        .collect();

    println!("Fiscal periods used: {:?}", fiscal_periods);

    for entry in &result.journal_entries {
        // All periods should be valid (1-12)
        assert!(
            entry.header.fiscal_period >= 1 && entry.header.fiscal_period <= 12,
            "Fiscal period {} should be between 1 and 12",
            entry.header.fiscal_period
        );

        // Fiscal year should be reasonable (2024 or 2025 depending on period)
        assert!(
            entry.header.fiscal_year >= 2024 && entry.header.fiscal_year <= 2025,
            "Fiscal year {} should be 2024 or 2025",
            entry.header.fiscal_year
        );
    }
}

/// Test posting date range.
#[test]
fn test_posting_date_range() {
    use chrono::{Datelike, NaiveDate};

    let mut config = minimal_config();
    config.global.seed = Some(22222);
    config.global.start_date = "2024-03-01".to_string();
    config.global.period_months = 2; // Shorter period to ensure we stay in range

    let phase_config = PhaseConfig {
        generate_master_data: false,
        generate_document_flows: false,
        generate_journal_entries: true,
        inject_anomalies: false,
        show_progress: false,
        ..Default::default()
    };

    let mut orchestrator =
        EnhancedOrchestrator::new(config, phase_config).expect("Failed to create orchestrator");

    let result = orchestrator.generate().expect("Generation failed");

    let start_date = NaiveDate::from_ymd_opt(2024, 3, 1).unwrap();

    // All entries should have dates starting from the configured start date
    for entry in &result.journal_entries {
        assert!(
            entry.header.posting_date >= start_date,
            "Posting date {} should be >= start date {}",
            entry.header.posting_date,
            start_date
        );

        // Posting date year should be 2024
        assert_eq!(
            entry.header.posting_date.year(),
            2024,
            "Posting date should be in 2024"
        );
    }

    // Verify dates span the expected period
    let posting_months: HashSet<_> = result
        .journal_entries
        .iter()
        .map(|e| e.header.posting_date.month())
        .collect();

    println!("Posting months: {:?}", posting_months);
}

/// Test with all master data types enabled.
#[test]
fn test_all_master_data_types() {
    let mut config = minimal_config();
    config.global.seed = Some(13579);
    config.global.period_months = 2;

    // Configure all master data types
    config.master_data.vendors.count = 10;
    config.master_data.customers.count = 10;
    config.master_data.materials.count = 20;
    config.master_data.fixed_assets.count = 5;
    config.master_data.employees.count = 10;

    let phase_config = PhaseConfig {
        generate_master_data: true,
        generate_document_flows: true,
        generate_journal_entries: true,
        inject_anomalies: false,
        show_progress: false,
        ..Default::default()
    };

    let mut orchestrator =
        EnhancedOrchestrator::new(config, phase_config).expect("Failed to create orchestrator");

    let result = orchestrator.generate().expect("Generation failed");

    // Verify journal entries exist
    assert!(
        !result.journal_entries.is_empty(),
        "Should generate journal entries"
    );

    println!(
        "All master data test generated {} journal entries",
        result.journal_entries.len()
    );
}

/// Test multi-currency generation.
#[test]
fn test_multi_currency_generation() {
    let mut config = multi_company_config();
    config.global.seed = Some(24680);
    config.global.period_months = 1;

    let phase_config = PhaseConfig {
        generate_master_data: false,
        generate_document_flows: false,
        generate_journal_entries: true,
        inject_anomalies: false,
        show_progress: false,
        ..Default::default()
    };

    let mut orchestrator =
        EnhancedOrchestrator::new(config, phase_config).expect("Failed to create orchestrator");

    let result = orchestrator.generate().expect("Generation failed");

    // Should generate entries
    assert!(
        !result.journal_entries.is_empty(),
        "Should generate entries"
    );

    // Check exchange rates are set
    for entry in &result.journal_entries {
        assert!(
            entry.header.exchange_rate > Decimal::ZERO,
            "Exchange rate should be positive"
        );
    }
}

/// Test balance configuration features.
#[test]
fn test_balance_configuration() {
    let mut config = minimal_config();
    config.global.seed = Some(36912);
    config.global.period_months = 3;

    // Enable balance features
    config.balance.generate_opening_balances = true;
    config.balance.generate_trial_balances = true;

    let phase_config = PhaseConfig {
        generate_master_data: true,
        generate_document_flows: false,
        generate_journal_entries: true,
        inject_anomalies: false,
        show_progress: false,
        ..Default::default()
    };

    let mut orchestrator =
        EnhancedOrchestrator::new(config, phase_config).expect("Failed to create orchestrator");

    let result = orchestrator.generate().expect("Generation failed");

    // Verify entries exist
    assert!(
        !result.journal_entries.is_empty(),
        "Should generate journal entries"
    );

    println!(
        "Balance configuration test generated {} journal entries",
        result.journal_entries.len()
    );
}
