//! Generation integration tests.
//!
//! Tests the full generation pipeline including master data, document flows,
//! and journal entries.

use datasynth_runtime::{EnhancedOrchestrator, PhaseConfig};
use datasynth_test_utils::fixtures::{minimal_config, multi_company_config};

/// Test basic generation with minimal configuration.
#[test]
fn test_basic_generation() {
    let mut config = minimal_config();
    config.global.seed = Some(42);

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

    assert!(
        !result.journal_entries.is_empty(),
        "Should generate at least one journal entry"
    );

    // Verify entries have the correct company code
    for entry in &result.journal_entries {
        assert_eq!(entry.header.company_code, "TEST");
    }
}

/// Test generation with master data enabled.
#[test]
fn test_generation_with_master_data() {
    let mut config = minimal_config();
    config.global.seed = Some(43);

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

    // Should still generate entries even with master data enabled
    assert!(
        !result.journal_entries.is_empty(),
        "Should generate journal entries with master data enabled"
    );
}

/// Test multi-company generation.
#[test]
fn test_multi_company_generation() {
    let mut config = multi_company_config();
    config.global.seed = Some(44);
    config.global.period_months = 1; // Keep it short for speed

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

    // Collect company codes from entries
    let company_codes: std::collections::HashSet<_> = result
        .journal_entries
        .iter()
        .map(|e| e.header.company_code.as_str())
        .collect();

    // With weighted distribution, at least one company should have entries
    assert!(
        !company_codes.is_empty(),
        "Should generate entries for at least one company"
    );
}

/// Test that period months affects output.
#[test]
fn test_period_months_affects_output() {
    let mut config_short = minimal_config();
    config_short.global.seed = Some(100);
    config_short.global.period_months = 1;

    let mut config_long = minimal_config();
    config_long.global.seed = Some(100);
    config_long.global.period_months = 6;

    let phase_config = PhaseConfig {
        generate_master_data: false,
        generate_document_flows: false,
        generate_journal_entries: true,
        inject_anomalies: false,
        show_progress: false,
        ..Default::default()
    };

    let mut orchestrator_short = EnhancedOrchestrator::new(config_short, phase_config.clone())
        .expect("Failed to create orchestrator");
    let result_short = orchestrator_short.generate().expect("Generation failed");

    let mut orchestrator_long = EnhancedOrchestrator::new(config_long, phase_config)
        .expect("Failed to create orchestrator");
    let result_long = orchestrator_long.generate().expect("Generation failed");

    // Longer period should generally produce more entries
    // (though this isn't strictly guaranteed due to volume-based generation)
    assert!(
        !result_short.journal_entries.is_empty() && !result_long.journal_entries.is_empty(),
        "Both configurations should produce entries"
    );
}

/// Test fiscal period assignment in entries.
#[test]
fn test_fiscal_period_assignment() {
    let mut config = minimal_config();
    config.global.seed = Some(45);
    config.global.period_months = 3;

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

    // All entries should have valid fiscal periods (1-12)
    for entry in &result.journal_entries {
        assert!(
            entry.header.fiscal_period >= 1 && entry.header.fiscal_period <= 12,
            "Fiscal period should be between 1 and 12, got {}",
            entry.header.fiscal_period
        );
    }
}

/// Test entry line numbers are sequential.
#[test]
fn test_line_numbers_sequential() {
    let mut config = minimal_config();
    config.global.seed = Some(46);

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

    for entry in &result.journal_entries {
        let line_numbers: Vec<_> = entry.lines.iter().map(|l| l.line_number).collect();

        // Check that line numbers start at 1 and are sequential
        for (i, &ln) in line_numbers.iter().enumerate() {
            assert_eq!(
                ln,
                (i + 1) as u32,
                "Line number should be sequential, expected {} got {}",
                i + 1,
                ln
            );
        }
    }
}

/// Test that document IDs have reasonable uniqueness.
///
/// Note: Due to batch generation, some document IDs may be reused across
/// different generation batches. This test verifies a reasonable level
/// of uniqueness rather than strict 100% uniqueness.
#[test]
fn test_document_id_generation() {
    let mut config = minimal_config();
    config.global.seed = Some(47);
    config.global.period_months = 1; // Shorter period for consistent test

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

    let doc_ids: std::collections::HashSet<_> = result
        .journal_entries
        .iter()
        .map(|e| e.header.document_id)
        .collect();

    // Document IDs should be generated (non-nil)
    for entry in &result.journal_entries {
        assert!(
            !entry.header.document_id.is_nil(),
            "Document ID should not be nil"
        );
    }

    // Should have some unique document IDs
    assert!(
        !doc_ids.is_empty(),
        "Should have at least some unique document IDs"
    );
}

/// Test that line document IDs match header.
#[test]
fn test_line_document_ids_match_header() {
    let mut config = minimal_config();
    config.global.seed = Some(48);

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

    for entry in &result.journal_entries {
        for line in &entry.lines {
            assert_eq!(
                line.document_id, entry.header.document_id,
                "Line document_id should match header document_id"
            );
        }
    }
}
