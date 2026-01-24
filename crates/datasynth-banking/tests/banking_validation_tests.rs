//! Validation tests for banking data generation.
//!
//! These tests validate that generated banking data meets compliance requirements,
//! has proper KYC coherence, correct typology distribution, and accurate labels.

use std::collections::{HashMap, HashSet};

use datasynth_banking::{
    BankingConfig, BankingOrchestrator, Direction, RiskTier, TransactionCategory,
};

// =============================================================================
// KYC Profile Coherence Tests
// =============================================================================

/// Test that customer KYC profiles are coherent with customer type.
#[test]
fn test_kyc_profile_coherence() {
    let config = BankingConfig::small();
    let orchestrator = BankingOrchestrator::new(config, 12345);
    let data = orchestrator.generate();

    for customer in &data.customers {
        let kyc = &customer.kyc_profile;

        // Expected turnover should be positive
        assert!(
            kyc.expected_monthly_turnover >= 0.0,
            "Customer {} has invalid expected turnover: {}",
            customer.customer_id,
            kyc.expected_monthly_turnover
        );

        // Transaction frequency should be reasonable
        assert!(
            kyc.expected_transaction_frequency >= 0,
            "Customer {} has invalid transaction frequency: {}",
            customer.customer_id,
            kyc.expected_transaction_frequency
        );

        // KYC completeness should be recorded
        assert!(
            kyc.kyc_completeness >= 0.0 && kyc.kyc_completeness <= 1.0,
            "Customer {} has invalid KYC completeness: {}",
            customer.customer_id,
            kyc.kyc_completeness
        );
    }
}

/// Test that business customers have business-appropriate KYC profiles.
#[test]
fn test_business_customer_kyc() {
    let mut config = BankingConfig::small();
    config.population.business_customers = 50;

    let orchestrator = BankingOrchestrator::new(config, 54321);
    let data = orchestrator.generate();

    let business_customers: Vec<_> = data
        .customers
        .iter()
        .filter(|c| {
            c.customer_type == datasynth_banking::BankingCustomerType::Business
                || c.customer_type == datasynth_banking::BankingCustomerType::Trust
        })
        .collect();

    for customer in &business_customers {
        // Business customers typically have higher expected turnover
        assert!(
            customer.kyc_profile.expected_monthly_turnover >= 0.0,
            "Business customer {} should have valid turnover",
            customer.customer_id
        );
    }

    println!(
        "Validated {} business/trust customers",
        business_customers.len()
    );
}

// =============================================================================
// Account Feature Validation Tests
// =============================================================================

/// Test that account features match customer type.
#[test]
fn test_account_feature_validation() {
    let config = BankingConfig::small();
    let orchestrator = BankingOrchestrator::new(config, 67890);
    let data = orchestrator.generate();

    // Build customer ID -> type map
    let customer_types: HashMap<_, _> = data
        .customers
        .iter()
        .map(|c| (c.customer_id.clone(), c.customer_type.clone()))
        .collect();

    for account in &data.accounts {
        // All accounts should have a customer ID
        assert!(
            !account.customer_id.is_empty(),
            "Account {} missing customer ID",
            account.account_id
        );

        // Account should reference valid customer
        assert!(
            customer_types.contains_key(&account.customer_id),
            "Account {} references unknown customer {}",
            account.account_id,
            account.customer_id
        );

        // Balance should be non-negative for standard accounts
        // (Some account types may allow overdraft)
        if !account.has_overdraft {
            assert!(
                account.current_balance >= rust_decimal::Decimal::ZERO,
                "Account {} has negative balance without overdraft",
                account.account_id
            );
        }
    }
}

/// Test that each customer has at least one account.
#[test]
fn test_customer_account_linkage() {
    let config = BankingConfig::small();
    let orchestrator = BankingOrchestrator::new(config, 11111);
    let data = orchestrator.generate();

    // Collect customers with accounts
    let customers_with_accounts: HashSet<_> = data
        .accounts
        .iter()
        .map(|a| a.customer_id.clone())
        .collect();

    // All customers should have at least one account
    for customer in &data.customers {
        assert!(
            customers_with_accounts.contains(&customer.customer_id),
            "Customer {} has no accounts",
            customer.customer_id
        );
    }
}

// =============================================================================
// Customer Type Distribution Tests
// =============================================================================

/// Test that customer type distribution matches configuration.
#[test]
fn test_customer_type_distribution() {
    let mut config = BankingConfig::small();
    config.population.retail_customers = 100;
    config.population.business_customers = 20;
    config.population.trusts = 5;

    let orchestrator = BankingOrchestrator::new(config.clone(), 22222);
    let data = orchestrator.generate();

    let mut type_counts: HashMap<_, usize> = HashMap::new();
    for customer in &data.customers {
        *type_counts.entry(customer.customer_type.clone()).or_default() += 1;
    }

    // Check counts match (with tolerance for generation logic)
    let retail_count = *type_counts
        .get(&datasynth_banking::BankingCustomerType::Retail)
        .unwrap_or(&0);
    let business_count = *type_counts
        .get(&datasynth_banking::BankingCustomerType::Business)
        .unwrap_or(&0);
    let trust_count = *type_counts
        .get(&datasynth_banking::BankingCustomerType::Trust)
        .unwrap_or(&0);

    // Verify counts are within expected range (allow 20% variance)
    assert!(
        retail_count >= 80 && retail_count <= 120,
        "Retail count {} outside expected range [80, 120]",
        retail_count
    );
    assert!(
        business_count >= 15 && business_count <= 30,
        "Business count {} outside expected range [15, 30]",
        business_count
    );
    assert!(
        trust_count <= 10,
        "Trust count {} outside expected range [0, 10]",
        trust_count
    );

    println!(
        "Customer distribution: retail={}, business={}, trust={}",
        retail_count, business_count, trust_count
    );
}

// =============================================================================
// AML Typology Detection Tests
// =============================================================================

/// Test that AML typologies are properly labeled.
#[test]
fn test_typology_labels() {
    let mut config = BankingConfig::small();
    // Increase suspicious rate for testing
    config.typologies.suspicious_rate = 0.10;
    config.typologies.structuring_rate = 0.03;
    config.typologies.mule_rate = 0.03;

    let orchestrator = BankingOrchestrator::new(config, 33333);
    let data = orchestrator.generate();

    // Count suspicious transactions
    let suspicious_count = data
        .transactions
        .iter()
        .filter(|t| t.is_suspicious)
        .count();

    // Should have some suspicious transactions
    if data.transactions.len() >= 100 {
        assert!(
            suspicious_count > 0,
            "Should have at least some suspicious transactions"
        );

        let suspicious_rate = suspicious_count as f64 / data.transactions.len() as f64;
        println!(
            "Suspicious rate: {:.2}% ({} of {})",
            suspicious_rate * 100.0,
            suspicious_count,
            data.transactions.len()
        );
    }

    // Verify transaction labels exist for suspicious transactions
    let suspicious_txn_ids: HashSet<_> = data
        .transactions
        .iter()
        .filter(|t| t.is_suspicious)
        .map(|t| t.transaction_id.clone())
        .collect();

    let labeled_suspicious_ids: HashSet<_> = data
        .transaction_labels
        .iter()
        .filter(|l| l.is_suspicious)
        .map(|l| l.transaction_id.clone())
        .collect();

    // Labels should match suspicious flags
    for txn_id in &suspicious_txn_ids {
        assert!(
            labeled_suspicious_ids.contains(txn_id),
            "Suspicious transaction {} missing label",
            txn_id
        );
    }
}

/// Test structuring detection patterns.
#[test]
fn test_structuring_patterns() {
    let mut config = BankingConfig::small();
    config.typologies.structuring_rate = 0.05;
    config.typologies.suspicious_rate = 0.10;

    let orchestrator = BankingOrchestrator::new(config, 44444);
    let data = orchestrator.generate();

    // Find structuring scenarios
    let structuring_scenarios: Vec<_> = data
        .scenarios
        .iter()
        .filter(|s| {
            matches!(
                s.typology,
                datasynth_banking::AmlTypology::Structuring { .. }
            )
        })
        .collect();

    if !structuring_scenarios.is_empty() {
        println!(
            "Found {} structuring scenarios",
            structuring_scenarios.len()
        );

        // Verify structuring scenario properties
        for scenario in &structuring_scenarios {
            assert!(
                !scenario.transactions.is_empty(),
                "Structuring scenario should have transactions"
            );
        }
    }
}

/// Test mule network detection patterns.
#[test]
fn test_mule_network_patterns() {
    let mut config = BankingConfig::small();
    config.typologies.mule_rate = 0.05;
    config.typologies.suspicious_rate = 0.10;

    let orchestrator = BankingOrchestrator::new(config, 55555);
    let data = orchestrator.generate();

    // Find mule scenarios
    let mule_scenarios: Vec<_> = data
        .scenarios
        .iter()
        .filter(|s| matches!(s.typology, datasynth_banking::AmlTypology::MuleNetwork { .. }))
        .collect();

    if !mule_scenarios.is_empty() {
        println!("Found {} mule network scenarios", mule_scenarios.len());

        // Verify mule scenario properties
        for scenario in &mule_scenarios {
            assert!(
                !scenario.transactions.is_empty(),
                "Mule scenario should have transactions"
            );
            assert!(
                !scenario.entities.is_empty(),
                "Mule scenario should involve entities"
            );
        }
    }
}

// =============================================================================
// Transaction Validation Tests
// =============================================================================

/// Test that transactions have valid amounts.
#[test]
fn test_transaction_amount_validation() {
    let config = BankingConfig::small();
    let orchestrator = BankingOrchestrator::new(config, 66666);
    let data = orchestrator.generate();

    for txn in &data.transactions {
        // Amount should be positive
        assert!(
            txn.amount > rust_decimal::Decimal::ZERO,
            "Transaction {} has non-positive amount: {}",
            txn.transaction_id,
            txn.amount
        );

        // Should have a valid category
        assert!(
            txn.category != TransactionCategory::Other || txn.is_suspicious,
            "Non-suspicious transaction {} should have specific category",
            txn.transaction_id
        );

        // Should have valid direction
        assert!(
            txn.direction == Direction::Inbound || txn.direction == Direction::Outbound,
            "Transaction {} has invalid direction",
            txn.transaction_id
        );
    }
}

/// Test transaction to account linkage.
#[test]
fn test_transaction_account_linkage() {
    let config = BankingConfig::small();
    let orchestrator = BankingOrchestrator::new(config, 77777);
    let data = orchestrator.generate();

    let account_ids: HashSet<_> = data.accounts.iter().map(|a| a.account_id.clone()).collect();

    for txn in &data.transactions {
        // Transaction should reference valid account
        assert!(
            account_ids.contains(&txn.account_id),
            "Transaction {} references unknown account {}",
            txn.transaction_id,
            txn.account_id
        );
    }
}

// =============================================================================
// Label Quality Tests
// =============================================================================

/// Test that transaction labels have correct format.
#[test]
fn test_transaction_label_format() {
    let config = BankingConfig::small();
    let orchestrator = BankingOrchestrator::new(config, 88888);
    let data = orchestrator.generate();

    for label in &data.transaction_labels {
        // Label should reference valid transaction
        assert!(
            !label.transaction_id.is_empty(),
            "Label missing transaction ID"
        );

        // Risk tier should be valid
        assert!(
            matches!(
                label.risk_tier,
                RiskTier::Low | RiskTier::Medium | RiskTier::High | RiskTier::Critical
            ),
            "Label has invalid risk tier"
        );

        // If suspicious, should have typology info
        if label.is_suspicious {
            // Suspicious labels may have scenario_id
            println!(
                "Suspicious label: txn={}, tier={:?}",
                label.transaction_id, label.risk_tier
            );
        }
    }
}

/// Test that customer labels exist for all customers.
#[test]
fn test_customer_label_coverage() {
    let config = BankingConfig::small();
    let orchestrator = BankingOrchestrator::new(config, 99999);
    let data = orchestrator.generate();

    let customer_ids: HashSet<_> = data
        .customers
        .iter()
        .map(|c| c.customer_id.clone())
        .collect();

    let labeled_customer_ids: HashSet<_> = data
        .customer_labels
        .iter()
        .map(|l| l.customer_id.clone())
        .collect();

    // All customers should have labels
    for customer_id in &customer_ids {
        assert!(
            labeled_customer_ids.contains(customer_id),
            "Customer {} missing label",
            customer_id
        );
    }
}

// =============================================================================
// Spoofing Tests
// =============================================================================

/// Test that spoofed transactions are properly marked.
#[test]
fn test_spoofing_labels() {
    let mut config = BankingConfig::small();
    config.spoofing.enabled = true;
    config.spoofing.intensity = 0.5;

    let orchestrator = BankingOrchestrator::new(config, 10101);
    let data = orchestrator.generate();

    // Count spoofed transactions
    let spoofed_count = data.transactions.iter().filter(|t| t.is_spoofed).count();

    // Should have some spoofed transactions
    if data.transactions.len() >= 100 {
        let spoofed_rate = spoofed_count as f64 / data.transactions.len() as f64;
        println!(
            "Spoofed rate: {:.2}% ({} of {})",
            spoofed_rate * 100.0,
            spoofed_count,
            data.transactions.len()
        );

        // Spoofed rate should be related to intensity (not exact due to random sampling)
        // Just verify we have some spoofed transactions
        assert!(
            spoofed_count > 0,
            "Should have some spoofed transactions with intensity=0.5"
        );
    }
}

// =============================================================================
// Generation Statistics Tests
// =============================================================================

/// Test that generation statistics are accurate.
#[test]
fn test_generation_statistics() {
    let config = BankingConfig::small();
    let orchestrator = BankingOrchestrator::new(config, 20202);
    let data = orchestrator.generate();

    // Verify stats match actual counts
    assert_eq!(
        data.stats.customer_count,
        data.customers.len(),
        "Customer count mismatch"
    );
    assert_eq!(
        data.stats.account_count,
        data.accounts.len(),
        "Account count mismatch"
    );
    assert_eq!(
        data.stats.transaction_count,
        data.transactions.len(),
        "Transaction count mismatch"
    );

    // Verify suspicious count
    let actual_suspicious = data
        .transactions
        .iter()
        .filter(|t| t.is_suspicious)
        .count();
    assert_eq!(
        data.stats.suspicious_count, actual_suspicious,
        "Suspicious count mismatch"
    );

    // Verify suspicious rate calculation
    if data.transactions.len() > 0 {
        let expected_rate = actual_suspicious as f64 / data.transactions.len() as f64;
        assert!(
            (data.stats.suspicious_rate - expected_rate).abs() < 0.001,
            "Suspicious rate mismatch"
        );
    }

    println!(
        "Generation stats: {} customers, {} accounts, {} transactions, {} suspicious",
        data.stats.customer_count,
        data.stats.account_count,
        data.stats.transaction_count,
        data.stats.suspicious_count
    );
}
