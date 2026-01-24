//! Validation tests for audit data generation.
//!
//! These tests validate engagement structure, workpaper references,
//! evidence linkage, risk assessment completeness, and finding severity distribution.

use std::collections::HashSet;

use chrono::NaiveDate;
use rust_decimal::Decimal;

use datasynth_core::models::audit::{
    AuditEngagement, EngagementPhase, EngagementType, FindingSeverity,
    FindingType, RiskLevel, WorkpaperSection,
};
use datasynth_generators::audit::{
    AuditEngagementConfig, AuditEngagementGenerator, FindingGenerator, FindingGeneratorConfig,
    WorkpaperGenerator,
};

// =============================================================================
// Engagement Structure Validation Tests
// =============================================================================

/// Test that engagement has required fields.
#[test]
fn test_engagement_required_fields() {
    let mut generator = AuditEngagementGenerator::new(42);
    let period_end = NaiveDate::from_ymd_opt(2025, 12, 31).unwrap();
    let revenue = Decimal::new(100_000_000, 0);

    let engagement =
        generator.generate_engagement("ENTITY001", "Test Company Inc.", 2025, period_end, revenue, None);

    // Required identification fields
    assert!(!engagement.engagement_ref.is_empty(), "Engagement ref required");
    assert_eq!(engagement.fiscal_year, 2025, "Fiscal year should match");
    assert_eq!(
        engagement.client_entity_id, "ENTITY001",
        "Client entity ID should match"
    );
    assert!(!engagement.client_name.is_empty(), "Client name required");

    // Materiality fields
    assert!(
        engagement.materiality > Decimal::ZERO,
        "Materiality should be positive"
    );
    assert!(
        engagement.performance_materiality <= engagement.materiality,
        "Performance materiality should be <= materiality"
    );

    // Timeline fields
    assert!(
        engagement.planning_start < engagement.planning_end,
        "Planning dates should be sequential"
    );
    assert!(
        engagement.fieldwork_start < engagement.fieldwork_end,
        "Fieldwork dates should be sequential"
    );

    // Team fields
    assert!(
        !engagement.engagement_partner_id.is_empty(),
        "Partner ID required"
    );
    assert!(
        !engagement.engagement_manager_id.is_empty(),
        "Manager ID required"
    );
}

/// Test that engagement timeline is coherent.
#[test]
fn test_engagement_timeline_coherence() {
    let mut generator = AuditEngagementGenerator::new(42);
    let period_end = NaiveDate::from_ymd_opt(2025, 12, 31).unwrap();
    let revenue = Decimal::new(50_000_000, 0);

    let engagement = generator.generate_engagement(
        "ENTITY002",
        "Timeline Test Corp",
        2025,
        period_end,
        revenue,
        None,
    );

    // Planning should start before period end
    assert!(
        engagement.planning_start <= period_end,
        "Planning should start before period end"
    );

    // Planning should end before or at fieldwork start
    assert!(
        engagement.planning_end <= engagement.fieldwork_start,
        "Planning should end before fieldwork starts"
    );

    // Fieldwork should end before or at completion start
    assert!(
        engagement.fieldwork_end <= engagement.completion_start,
        "Fieldwork should end before completion starts"
    );

    // Completion should end with or before report date
    assert!(
        engagement.completion_start <= engagement.report_date,
        "Completion should be before report date"
    );

    // Report date should be after period end
    assert!(
        engagement.report_date > period_end,
        "Report date should be after period end"
    );
}

/// Test that materiality is calculated correctly.
#[test]
fn test_materiality_calculation() {
    let mut generator = AuditEngagementGenerator::new(42);
    let period_end = NaiveDate::from_ymd_opt(2025, 12, 31).unwrap();
    let revenue = Decimal::new(100_000_000, 0);

    let engagement = generator.generate_engagement(
        "ENTITY003",
        "Materiality Test Corp",
        2025,
        period_end,
        revenue,
        None,
    );

    // Materiality should be within expected range (0.3% - 1% of revenue)
    let min_materiality = revenue * Decimal::try_from(0.003).unwrap();
    let max_materiality = revenue * Decimal::try_from(0.010).unwrap();

    assert!(
        engagement.materiality >= min_materiality,
        "Materiality {} should be >= {:.0}",
        engagement.materiality,
        min_materiality
    );
    assert!(
        engagement.materiality <= max_materiality,
        "Materiality {} should be <= {:.0}",
        engagement.materiality,
        max_materiality
    );

    // Performance materiality should be 50-75% of materiality
    let min_perf = engagement.materiality * Decimal::try_from(0.50).unwrap();
    let max_perf = engagement.materiality * Decimal::try_from(0.75).unwrap();

    assert!(
        engagement.performance_materiality >= min_perf,
        "Performance materiality should be >= 50% of materiality"
    );
    assert!(
        engagement.performance_materiality <= max_perf,
        "Performance materiality should be <= 75% of materiality"
    );
}

/// Test engagement phase progression.
#[test]
fn test_engagement_phase_progression() {
    let mut generator = AuditEngagementGenerator::new(42);
    let period_end = NaiveDate::from_ymd_opt(2025, 12, 31).unwrap();
    let revenue = Decimal::new(75_000_000, 0);

    let mut engagement = generator.generate_engagement(
        "ENTITY004",
        "Phase Test Corp",
        2025,
        period_end,
        revenue,
        None,
    );

    // Initial phase should be planning
    assert_eq!(
        engagement.current_phase,
        EngagementPhase::Planning,
        "Initial phase should be Planning"
    );

    // Copy dates to avoid borrowing issues
    let planning_end = engagement.planning_end;
    let fieldwork_start = engagement.fieldwork_start;
    let completion_start = engagement.completion_start;

    // Advance through phases
    generator.advance_engagement_phase(&mut engagement, planning_end);
    assert!(
        engagement.current_phase != EngagementPhase::Planning,
        "Should advance past planning after planning_end"
    );

    generator.advance_engagement_phase(&mut engagement, fieldwork_start);
    assert!(
        matches!(
            engagement.current_phase,
            EngagementPhase::ControlTesting | EngagementPhase::SubstantiveTesting | EngagementPhase::RiskAssessment
        ),
        "Should be in fieldwork-related phase during fieldwork"
    );

    generator.advance_engagement_phase(&mut engagement, completion_start);
    assert!(
        matches!(
            engagement.current_phase,
            EngagementPhase::Completion | EngagementPhase::SubstantiveTesting
        ),
        "Should be in completion or late fieldwork phase"
    );
}

// =============================================================================
// Workpaper Reference Validation Tests
// =============================================================================

/// Test that workpapers are properly generated for all phases.
#[test]
fn test_workpaper_phase_coverage() {
    let mut eng_generator = AuditEngagementGenerator::new(42);
    let mut wp_generator = WorkpaperGenerator::new(42);

    let period_end = NaiveDate::from_ymd_opt(2025, 12, 31).unwrap();
    let engagement = eng_generator.generate_engagement(
        "ENTITY005",
        "Workpaper Test Corp",
        2025,
        period_end,
        Decimal::new(80_000_000, 0),
        None,
    );

    let team = vec![
        "STAFF001".into(),
        "STAFF002".into(),
        "SENIOR001".into(),
        "MANAGER001".into(),
    ];

    let workpapers = wp_generator.generate_complete_workpaper_set(&engagement, &team);

    // Should have workpapers
    assert!(!workpapers.is_empty(), "Should generate workpapers");

    // Check section coverage
    let sections: HashSet<_> = workpapers.iter().map(|w| w.section).collect();

    assert!(
        sections.contains(&WorkpaperSection::Planning),
        "Should have planning workpapers"
    );
    assert!(
        sections.contains(&WorkpaperSection::RiskAssessment),
        "Should have risk assessment workpapers"
    );
    assert!(
        sections.contains(&WorkpaperSection::ControlTesting),
        "Should have control testing workpapers"
    );
    assert!(
        sections.contains(&WorkpaperSection::SubstantiveTesting),
        "Should have substantive testing workpapers"
    );
    assert!(
        sections.contains(&WorkpaperSection::Completion),
        "Should have completion workpapers"
    );

    println!(
        "Generated {} workpapers across {} sections",
        workpapers.len(),
        sections.len()
    );
}

/// Test that workpaper references are unique.
#[test]
fn test_workpaper_unique_references() {
    let mut wp_generator = WorkpaperGenerator::new(42);
    let engagement = create_test_engagement();
    let team = vec!["STAFF001".into(), "SENIOR001".into()];

    let workpapers = wp_generator.generate_complete_workpaper_set(&engagement, &team);

    let refs: HashSet<_> = workpapers.iter().map(|w| w.workpaper_ref.clone()).collect();

    assert_eq!(
        refs.len(),
        workpapers.len(),
        "Workpaper references should be unique"
    );
}

/// Test that workpapers have proper review chain.
#[test]
fn test_workpaper_review_chain() {
    let mut wp_generator = WorkpaperGenerator::new(42);
    let engagement = create_test_engagement();
    let team = vec![
        "STAFF001".into(),
        "SENIOR001".into(),
        "MANAGER001".into(),
    ];

    let workpapers = wp_generator.generate_complete_workpaper_set(&engagement, &team);

    for wp in &workpapers {
        // All workpapers should have a preparer
        assert!(
            !wp.preparer_id.is_empty(),
            "Workpaper {} should have preparer",
            wp.workpaper_ref
        );

        // All workpapers should have a first reviewer
        assert!(
            wp.reviewer_id.is_some(),
            "Workpaper {} should have first reviewer",
            wp.workpaper_ref
        );

        // Review date should be after preparer date
        if let Some(review_date) = wp.reviewer_date {
            assert!(
                review_date >= wp.preparer_date,
                "Workpaper {} first review should be after preparation",
                wp.workpaper_ref
            );
        }

        // If second review exists, it should be after first
        if let (Some(first), Some(second)) = (wp.reviewer_date, wp.second_reviewer_date) {
            assert!(
                second >= first,
                "Workpaper {} second review should be after first",
                wp.workpaper_ref
            );
        }
    }
}

/// Test that testing workpapers have sampling information.
#[test]
fn test_workpaper_sampling_for_testing() {
    let mut wp_generator = WorkpaperGenerator::new(42);
    let engagement = create_test_engagement();
    let team = vec!["STAFF001".into(), "SENIOR001".into()];

    let workpapers = wp_generator.generate_complete_workpaper_set(&engagement, &team);

    // Filter testing workpapers
    let testing_workpapers: Vec<_> = workpapers
        .iter()
        .filter(|w| {
            matches!(
                w.section,
                WorkpaperSection::ControlTesting | WorkpaperSection::SubstantiveTesting
            )
        })
        .collect();

    // Most testing workpapers should have sampling
    let with_sampling = testing_workpapers
        .iter()
        .filter(|w| w.population_size > 0)
        .count();

    assert!(
        with_sampling > testing_workpapers.len() / 2,
        "Most testing workpapers should have sampling: {}/{}",
        with_sampling,
        testing_workpapers.len()
    );

    // Verify sample size is reasonable
    for wp in testing_workpapers.iter().filter(|w| w.population_size > 0) {
        assert!(
            wp.sample_size <= wp.population_size as u32,
            "Sample size should be <= population for {}",
            wp.workpaper_ref
        );
        assert!(
            wp.sample_size >= 25 || wp.population_size < 100,
            "Sample size should be >= 25 for significant populations for {}",
            wp.workpaper_ref
        );
    }
}

// =============================================================================
// Finding Severity Distribution Tests
// =============================================================================

/// Test that finding severity distribution is reasonable.
#[test]
fn test_finding_severity_distribution() {
    let config = FindingGeneratorConfig {
        findings_per_engagement: (50, 50),
        ..Default::default()
    };
    let mut generator = FindingGenerator::with_config(42, config);

    let engagement = create_test_engagement();
    let team = vec!["STAFF001".into(), "SENIOR001".into(), "MANAGER001".into()];

    let findings = generator.generate_findings_for_engagement(&engagement, &[], &team);

    // Count by severity
    let mut severity_counts: std::collections::HashMap<FindingSeverity, usize> =
        std::collections::HashMap::new();
    for finding in &findings {
        *severity_counts.entry(finding.severity).or_default() += 1;
    }

    // Critical findings should be rare
    let critical_count = severity_counts.get(&FindingSeverity::Critical).unwrap_or(&0);
    let total = findings.len();

    assert!(
        *critical_count < total / 4,
        "Critical findings should be rare: {}/{}",
        critical_count,
        total
    );

    // Should have a mix of severities
    assert!(
        severity_counts.len() >= 3,
        "Should have diverse severity levels: {}",
        severity_counts.len()
    );

    println!(
        "Finding severity distribution: {:?}",
        severity_counts
    );
}

/// Test that finding types match severity expectations.
#[test]
fn test_finding_type_severity_consistency() {
    let config = FindingGeneratorConfig {
        findings_per_engagement: (30, 30),
        ..Default::default()
    };
    let mut generator = FindingGenerator::with_config(42, config);

    let engagement = create_test_engagement();
    let team = vec!["STAFF001".into()];

    let findings = generator.generate_findings_for_engagement(&engagement, &[], &team);

    for finding in &findings {
        // Material weaknesses should generally be high severity
        if finding.finding_type == FindingType::MaterialWeakness {
            assert!(
                matches!(
                    finding.severity,
                    FindingSeverity::Critical | FindingSeverity::High
                ),
                "Material weakness should be high severity: {:?}",
                finding.severity
            );
        }

        // Process improvements should be low severity
        if finding.finding_type == FindingType::ProcessImprovement {
            assert!(
                matches!(
                    finding.severity,
                    FindingSeverity::Low | FindingSeverity::Informational | FindingSeverity::Medium
                ),
                "Process improvement should be lower severity: {:?}",
                finding.severity
            );
        }
    }
}

/// Test that findings have required CCCE structure.
#[test]
fn test_finding_ccce_structure() {
    let mut generator = FindingGenerator::new(42);
    let engagement = create_test_engagement();
    let team = vec!["STAFF001".into(), "SENIOR001".into()];

    let findings = generator.generate_findings_for_engagement(&engagement, &[], &team);

    for finding in &findings {
        // All findings should have CCCE
        assert!(
            !finding.condition.is_empty(),
            "Finding {} should have condition",
            finding.finding_ref
        );
        assert!(
            !finding.criteria.is_empty(),
            "Finding {} should have criteria",
            finding.finding_ref
        );
        assert!(
            !finding.cause.is_empty(),
            "Finding {} should have cause",
            finding.finding_ref
        );
        assert!(
            !finding.effect.is_empty(),
            "Finding {} should have effect",
            finding.finding_ref
        );

        // All findings should have recommendations
        assert!(
            !finding.recommendation.is_empty(),
            "Finding {} should have recommendation",
            finding.finding_ref
        );
    }
}

/// Test that misstatement findings have monetary values.
#[test]
fn test_misstatement_findings_have_amounts() {
    let config = FindingGeneratorConfig {
        misstatement_probability: 1.0,
        material_weakness_probability: 0.0,
        significant_deficiency_probability: 0.0,
        findings_per_engagement: (10, 10),
        ..Default::default()
    };
    let mut generator = FindingGenerator::with_config(42, config);

    let engagement = create_test_engagement();
    let team = vec!["STAFF001".into()];

    let findings = generator.generate_findings_for_engagement(&engagement, &[], &team);

    // Misstatement findings should have amounts
    let misstatement_findings: Vec<_> = findings
        .iter()
        .filter(|f| {
            matches!(
                f.finding_type,
                FindingType::MaterialMisstatement | FindingType::ImmaterialMisstatement
            )
        })
        .collect();

    assert!(
        !misstatement_findings.is_empty(),
        "Should have misstatement findings with probability 1.0"
    );

    for finding in &misstatement_findings {
        assert!(
            finding.factual_misstatement.is_some()
                || finding.projected_misstatement.is_some()
                || finding.judgmental_misstatement.is_some(),
            "Misstatement finding {} should have amount",
            finding.finding_ref
        );
    }
}

/// Test that findings requiring governance communication are flagged.
#[test]
fn test_governance_communication_flagging() {
    let config = FindingGeneratorConfig {
        material_weakness_probability: 0.5,
        significant_deficiency_probability: 0.5,
        findings_per_engagement: (20, 20),
        ..Default::default()
    };
    let mut generator = FindingGenerator::with_config(42, config);

    let engagement = create_test_engagement();
    let team = vec!["STAFF001".into(), "SENIOR001".into()];

    let findings = generator.generate_findings_for_engagement(&engagement, &[], &team);

    // Material weaknesses and significant deficiencies should be flagged for governance
    let critical_findings: Vec<_> = findings
        .iter()
        .filter(|f| {
            matches!(
                f.finding_type,
                FindingType::MaterialWeakness | FindingType::SignificantDeficiency
            )
        })
        .collect();

    let flagged_for_governance = critical_findings
        .iter()
        .filter(|f| f.report_to_governance || f.include_in_management_letter)
        .count();

    assert!(
        flagged_for_governance >= critical_findings.len() / 2,
        "Critical findings should be flagged for governance communication: {}/{}",
        flagged_for_governance,
        critical_findings.len()
    );
}

// =============================================================================
// Risk Assessment Completeness Tests
// =============================================================================

/// Test that engagements have risk assessment.
#[test]
fn test_engagement_risk_assessment() {
    let mut generator = AuditEngagementGenerator::new(42);

    // Generate multiple engagements
    for i in 0..10 {
        let period_end = NaiveDate::from_ymd_opt(2025, 12, 31).unwrap();
        let engagement = generator.generate_engagement(
            &format!("ENTITY{:03}", i),
            &format!("Risk Test Corp {}", i),
            2025,
            period_end,
            Decimal::new(50_000_000 + i as i64 * 10_000_000, 0),
            None,
        );

        // All engagements should have risk level assigned
        assert!(
            matches!(
                engagement.overall_audit_risk,
                RiskLevel::Low | RiskLevel::Medium | RiskLevel::High | RiskLevel::Significant
            ),
            "Engagement should have overall audit risk"
        );

        assert!(
            matches!(
                engagement.fraud_risk_level,
                RiskLevel::Low | RiskLevel::Medium | RiskLevel::High | RiskLevel::Significant
            ),
            "Engagement should have fraud risk level"
        );
    }
}

/// Test that risk levels influence findings.
#[test]
fn test_risk_influenced_findings() {
    // High risk engagement
    let high_risk_config = AuditEngagementConfig {
        high_fraud_risk_probability: 1.0,
        significant_risk_probability: 1.0,
        ..Default::default()
    };
    let mut eng_generator = AuditEngagementGenerator::with_config(42, high_risk_config);

    let period_end = NaiveDate::from_ymd_opt(2025, 12, 31).unwrap();
    let high_risk_engagement = eng_generator.generate_engagement(
        "ENTITY_HR",
        "High Risk Corp",
        2025,
        period_end,
        Decimal::new(100_000_000, 0),
        None,
    );

    assert_eq!(
        high_risk_engagement.fraud_risk_level,
        RiskLevel::High,
        "Should have high fraud risk"
    );
    assert!(
        high_risk_engagement.significant_risk_count > 2,
        "Should have significant risks"
    );
}

// =============================================================================
// Cross-Reference Validation Tests
// =============================================================================

/// Test that findings can reference workpapers.
#[test]
fn test_finding_workpaper_references() {
    let mut eng_generator = AuditEngagementGenerator::new(42);
    let mut wp_generator = WorkpaperGenerator::new(42);
    let mut finding_generator = FindingGenerator::new(42);

    let period_end = NaiveDate::from_ymd_opt(2025, 12, 31).unwrap();
    let engagement = eng_generator.generate_engagement(
        "ENTITY_REF",
        "Reference Test Corp",
        2025,
        period_end,
        Decimal::new(75_000_000, 0),
        None,
    );

    let team = vec!["STAFF001".into(), "SENIOR001".into(), "MANAGER001".into()];

    let workpapers = wp_generator.generate_complete_workpaper_set(&engagement, &team);
    let findings = finding_generator.generate_findings_for_engagement(&engagement, &workpapers, &team);

    // Findings should reference workpapers
    let findings_with_refs = findings.iter().filter(|f| !f.workpaper_refs.is_empty()).count();

    assert!(
        findings_with_refs > findings.len() / 2,
        "Most findings should reference workpapers: {}/{}",
        findings_with_refs,
        findings.len()
    );

    // Referenced workpapers should exist
    let workpaper_ids: HashSet<_> = workpapers.iter().map(|w| w.workpaper_id).collect();

    for finding in &findings {
        for wp_ref in &finding.workpaper_refs {
            assert!(
                workpaper_ids.contains(wp_ref),
                "Finding {} references unknown workpaper {:?}",
                finding.finding_ref,
                wp_ref
            );
        }
    }
}

// =============================================================================
// Helper Functions
// =============================================================================

fn create_test_engagement() -> AuditEngagement {
    AuditEngagement::new(
        "ENTITY_TEST",
        "Test Company Inc.",
        EngagementType::AnnualAudit,
        2025,
        NaiveDate::from_ymd_opt(2025, 12, 31).unwrap(),
    )
    .with_materiality(
        Decimal::new(1_000_000, 0),
        0.75,
        0.05,
        "Total Revenue",
        0.005,
    )
    .with_timeline(
        NaiveDate::from_ymd_opt(2025, 10, 1).unwrap(),
        NaiveDate::from_ymd_opt(2025, 10, 31).unwrap(),
        NaiveDate::from_ymd_opt(2026, 1, 5).unwrap(),
        NaiveDate::from_ymd_opt(2026, 2, 15).unwrap(),
        NaiveDate::from_ymd_opt(2026, 2, 16).unwrap(),
        NaiveDate::from_ymd_opt(2026, 3, 15).unwrap(),
    )
}
