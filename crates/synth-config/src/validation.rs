//! Configuration validation.

use crate::schema::GeneratorConfig;
use synth_core::error::{SynthError, SynthResult};

/// Validate a generator configuration.
pub fn validate_config(config: &GeneratorConfig) -> SynthResult<()> {
    // Validate global settings
    if config.global.period_months == 0 {
        return Err(SynthError::validation(
            "period_months must be greater than 0",
        ));
    }

    // Validate companies
    if config.companies.is_empty() {
        return Err(SynthError::validation(
            "At least one company must be configured",
        ));
    }

    for company in &config.companies {
        if company.code.is_empty() {
            return Err(SynthError::validation("Company code cannot be empty"));
        }
        if company.currency.len() != 3 {
            return Err(SynthError::validation(format!(
                "Invalid currency code '{}' for company '{}'",
                company.currency, company.code
            )));
        }
    }

    // Validate transaction distribution
    let line_dist = &config.transactions.line_item_distribution;
    if let Err(e) = line_dist.validate() {
        return Err(SynthError::validation(e));
    }

    // Validate source distribution sums to ~1.0
    let source_sum = config.transactions.source_distribution.manual
        + config.transactions.source_distribution.automated
        + config.transactions.source_distribution.recurring
        + config.transactions.source_distribution.adjustment;
    if (source_sum - 1.0).abs() > 0.01 {
        return Err(SynthError::validation(format!(
            "Source distribution must sum to 1.0, got {}",
            source_sum
        )));
    }

    // Validate business process weights
    let bp_sum = config.business_processes.o2c_weight
        + config.business_processes.p2p_weight
        + config.business_processes.r2r_weight
        + config.business_processes.h2r_weight
        + config.business_processes.a2r_weight;
    if (bp_sum - 1.0).abs() > 0.01 {
        return Err(SynthError::validation(format!(
            "Business process weights must sum to 1.0, got {}",
            bp_sum
        )));
    }

    // Validate fraud config if enabled
    if config.fraud.enabled && (config.fraud.fraud_rate < 0.0 || config.fraud.fraud_rate > 1.0) {
        return Err(SynthError::validation(
            "fraud_rate must be between 0.0 and 1.0",
        ));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::presets::{create_preset, demo_preset, stress_test_preset};
    use crate::schema::*;
    use synth_core::models::{CoAComplexity, IndustrySector};

    /// Helper to create a minimal valid config for testing.
    fn minimal_valid_config() -> GeneratorConfig {
        GeneratorConfig {
            global: GlobalConfig {
                seed: Some(42),
                industry: IndustrySector::Manufacturing,
                start_date: "2024-01-01".to_string(),
                period_months: 3,
                group_currency: "USD".to_string(),
                parallel: true,
                worker_threads: 0,
                memory_limit_mb: 0,
            },
            companies: vec![CompanyConfig {
                code: "TEST".to_string(),
                name: "Test Company".to_string(),
                currency: "USD".to_string(),
                country: "US".to_string(),
                fiscal_year_variant: "K4".to_string(),
                annual_transaction_volume: TransactionVolume::TenK,
                volume_weight: 1.0,
            }],
            chart_of_accounts: ChartOfAccountsConfig {
                complexity: CoAComplexity::Small,
                industry_specific: true,
                custom_accounts: None,
                min_hierarchy_depth: 2,
                max_hierarchy_depth: 5,
            },
            transactions: TransactionConfig::default(),
            output: OutputConfig::default(),
            fraud: FraudConfig::default(),
            internal_controls: InternalControlsConfig::default(),
            business_processes: BusinessProcessConfig::default(),
            user_personas: UserPersonaConfig::default(),
            templates: TemplateConfig::default(),
            approval: ApprovalConfig::default(),
            departments: DepartmentConfig::default(),
            master_data: MasterDataConfig::default(),
            document_flows: DocumentFlowConfig::default(),
            intercompany: IntercompanyConfig::default(),
            balance: BalanceConfig::default(),
        }
    }

    // ==========================================================================
    // Period Months Validation Tests
    // ==========================================================================

    #[test]
    fn test_valid_period_months() {
        let config = minimal_valid_config();
        assert!(validate_config(&config).is_ok());
    }

    #[test]
    fn test_zero_period_months_rejected() {
        let mut config = minimal_valid_config();
        config.global.period_months = 0;
        let result = validate_config(&config);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("period_months"));
    }

    #[test]
    fn test_large_period_months_accepted() {
        let mut config = minimal_valid_config();
        config.global.period_months = 120; // 10 years
        assert!(validate_config(&config).is_ok());
    }

    // ==========================================================================
    // Company Validation Tests
    // ==========================================================================

    #[test]
    fn test_empty_companies_rejected() {
        let mut config = minimal_valid_config();
        config.companies.clear();
        let result = validate_config(&config);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("company"));
    }

    #[test]
    fn test_empty_company_code_rejected() {
        let mut config = minimal_valid_config();
        config.companies[0].code = "".to_string();
        let result = validate_config(&config);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Company code"));
    }

    #[test]
    fn test_invalid_currency_code_rejected() {
        let mut config = minimal_valid_config();
        config.companies[0].currency = "US".to_string(); // 2 chars, not 3
        let result = validate_config(&config);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("currency"));
    }

    #[test]
    fn test_long_currency_code_rejected() {
        let mut config = minimal_valid_config();
        config.companies[0].currency = "USDD".to_string(); // 4 chars
        let result = validate_config(&config);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("currency"));
    }

    #[test]
    fn test_multiple_companies_validated() {
        let mut config = minimal_valid_config();
        config.companies.push(CompanyConfig {
            code: "SUB1".to_string(),
            name: "Subsidiary 1".to_string(),
            currency: "EUR".to_string(),
            country: "DE".to_string(),
            fiscal_year_variant: "K4".to_string(),
            annual_transaction_volume: TransactionVolume::TenK,
            volume_weight: 0.5,
        });
        assert!(validate_config(&config).is_ok());
    }

    #[test]
    fn test_second_company_invalid_currency_rejected() {
        let mut config = minimal_valid_config();
        config.companies.push(CompanyConfig {
            code: "SUB1".to_string(),
            name: "Subsidiary 1".to_string(),
            currency: "EU".to_string(), // Invalid
            country: "DE".to_string(),
            fiscal_year_variant: "K4".to_string(),
            annual_transaction_volume: TransactionVolume::TenK,
            volume_weight: 0.5,
        });
        let result = validate_config(&config);
        assert!(result.is_err());
    }

    // ==========================================================================
    // Source Distribution Validation Tests
    // ==========================================================================

    #[test]
    fn test_valid_source_distribution() {
        let config = minimal_valid_config();
        // Default source distribution sums to 1.0
        assert!(validate_config(&config).is_ok());
    }

    #[test]
    fn test_source_distribution_not_summing_to_one_rejected() {
        let mut config = minimal_valid_config();
        config.transactions.source_distribution = SourceDistribution {
            manual: 0.5,
            automated: 0.5,
            recurring: 0.5, // Sum = 1.6
            adjustment: 0.1,
        };
        let result = validate_config(&config);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Source distribution"));
    }

    #[test]
    fn test_source_distribution_slightly_off_accepted() {
        let mut config = minimal_valid_config();
        // Within 0.01 tolerance
        config.transactions.source_distribution = SourceDistribution {
            manual: 0.20,
            automated: 0.70,
            recurring: 0.07,
            adjustment: 0.025, // Sum = 0.995, within tolerance
        };
        assert!(validate_config(&config).is_ok());
    }

    // ==========================================================================
    // Business Process Weights Validation Tests
    // ==========================================================================

    #[test]
    fn test_valid_business_process_weights() {
        let config = minimal_valid_config();
        // Default weights sum to 1.0
        assert!(validate_config(&config).is_ok());
    }

    #[test]
    fn test_business_process_weights_not_summing_to_one_rejected() {
        let mut config = minimal_valid_config();
        config.business_processes = BusinessProcessConfig {
            o2c_weight: 0.5,
            p2p_weight: 0.5,
            r2r_weight: 0.5, // Sum > 1
            h2r_weight: 0.1,
            a2r_weight: 0.1,
        };
        let result = validate_config(&config);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Business process weights"));
    }

    // ==========================================================================
    // Fraud Configuration Validation Tests
    // ==========================================================================

    #[test]
    fn test_fraud_disabled_invalid_rate_accepted() {
        let mut config = minimal_valid_config();
        config.fraud.enabled = false;
        config.fraud.fraud_rate = 5.0; // Invalid but ignored since disabled
        assert!(validate_config(&config).is_ok());
    }

    #[test]
    fn test_fraud_enabled_valid_rate_accepted() {
        let mut config = minimal_valid_config();
        config.fraud.enabled = true;
        config.fraud.fraud_rate = 0.05;
        assert!(validate_config(&config).is_ok());
    }

    #[test]
    fn test_fraud_enabled_negative_rate_rejected() {
        let mut config = minimal_valid_config();
        config.fraud.enabled = true;
        config.fraud.fraud_rate = -0.1;
        let result = validate_config(&config);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("fraud_rate"));
    }

    #[test]
    fn test_fraud_enabled_rate_above_one_rejected() {
        let mut config = minimal_valid_config();
        config.fraud.enabled = true;
        config.fraud.fraud_rate = 1.5;
        let result = validate_config(&config);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("fraud_rate"));
    }

    #[test]
    fn test_fraud_rate_zero_accepted() {
        let mut config = minimal_valid_config();
        config.fraud.enabled = true;
        config.fraud.fraud_rate = 0.0;
        assert!(validate_config(&config).is_ok());
    }

    #[test]
    fn test_fraud_rate_one_accepted() {
        let mut config = minimal_valid_config();
        config.fraud.enabled = true;
        config.fraud.fraud_rate = 1.0;
        assert!(validate_config(&config).is_ok());
    }

    // ==========================================================================
    // Preset Validation Tests
    // ==========================================================================

    #[test]
    fn test_demo_preset_valid() {
        let config = demo_preset();
        assert!(validate_config(&config).is_ok());
    }

    #[test]
    fn test_stress_test_preset_valid() {
        let config = stress_test_preset();
        assert!(validate_config(&config).is_ok());
    }

    #[test]
    fn test_manufacturing_preset_valid() {
        let config = create_preset(
            IndustrySector::Manufacturing,
            2,
            12,
            CoAComplexity::Medium,
            TransactionVolume::HundredK,
        );
        assert!(validate_config(&config).is_ok());
    }

    #[test]
    fn test_retail_preset_valid() {
        let config = create_preset(
            IndustrySector::Retail,
            3,
            6,
            CoAComplexity::Large,
            TransactionVolume::OneM,
        );
        assert!(validate_config(&config).is_ok());
    }

    #[test]
    fn test_financial_services_preset_valid() {
        let config = create_preset(
            IndustrySector::FinancialServices,
            2,
            12,
            CoAComplexity::Large,
            TransactionVolume::TenM,
        );
        assert!(validate_config(&config).is_ok());
    }

    #[test]
    fn test_healthcare_preset_valid() {
        let config = create_preset(
            IndustrySector::Healthcare,
            2,
            6,
            CoAComplexity::Medium,
            TransactionVolume::HundredK,
        );
        assert!(validate_config(&config).is_ok());
    }

    #[test]
    fn test_technology_preset_valid() {
        let config = create_preset(
            IndustrySector::Technology,
            3,
            12,
            CoAComplexity::Medium,
            TransactionVolume::HundredK,
        );
        assert!(validate_config(&config).is_ok());
    }

    // ==========================================================================
    // Transaction Volume Tests
    // ==========================================================================

    #[test]
    fn test_transaction_volume_counts() {
        assert_eq!(TransactionVolume::TenK.count(), 10_000);
        assert_eq!(TransactionVolume::HundredK.count(), 100_000);
        assert_eq!(TransactionVolume::OneM.count(), 1_000_000);
        assert_eq!(TransactionVolume::TenM.count(), 10_000_000);
        assert_eq!(TransactionVolume::HundredM.count(), 100_000_000);
        assert_eq!(TransactionVolume::Custom(50_000).count(), 50_000);
    }

    // ==========================================================================
    // Default Value Tests
    // ==========================================================================

    #[test]
    fn test_source_distribution_default_sums_to_one() {
        let dist = SourceDistribution::default();
        let sum = dist.manual + dist.automated + dist.recurring + dist.adjustment;
        assert!((sum - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_business_process_default_sums_to_one() {
        let bp = BusinessProcessConfig::default();
        let sum = bp.o2c_weight + bp.p2p_weight + bp.r2r_weight + bp.h2r_weight + bp.a2r_weight;
        assert!((sum - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_fraud_type_distribution_default_sums_to_one() {
        let dist = FraudTypeDistribution::default();
        let sum = dist.suspense_account_abuse
            + dist.fictitious_transaction
            + dist.revenue_manipulation
            + dist.expense_capitalization
            + dist.split_transaction
            + dist.timing_anomaly
            + dist.unauthorized_access
            + dist.duplicate_payment;
        assert!((sum - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_persona_distribution_default_sums_to_one() {
        let dist = PersonaDistribution::default();
        let sum = dist.junior_accountant
            + dist.senior_accountant
            + dist.controller
            + dist.manager
            + dist.automated_system;
        assert!((sum - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_payment_terms_distribution_default_sums_to_one() {
        let dist = PaymentTermsDistribution::default();
        let sum = dist.net_30
            + dist.net_60
            + dist.net_90
            + dist.two_ten_net_30
            + dist.due_on_receipt
            + dist.end_of_month;
        assert!((sum - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_vendor_behavior_distribution_default_sums_to_one() {
        let dist = VendorBehaviorDistribution::default();
        let sum = dist.reliable
            + dist.sometimes_late
            + dist.inconsistent_quality
            + dist.premium
            + dist.budget;
        assert!((sum - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_credit_rating_distribution_default_sums_to_one() {
        let dist = CreditRatingDistribution::default();
        let sum = dist.aaa + dist.aa + dist.a + dist.bbb + dist.bb + dist.b + dist.below_b;
        assert!((sum - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_payment_behavior_distribution_default_sums_to_one() {
        let dist = PaymentBehaviorDistribution::default();
        let sum = dist.early_payer
            + dist.on_time
            + dist.occasional_late
            + dist.frequent_late
            + dist.discount_taker;
        assert!((sum - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_material_type_distribution_default_sums_to_one() {
        let dist = MaterialTypeDistribution::default();
        let sum = dist.raw_material
            + dist.semi_finished
            + dist.finished_good
            + dist.trading_good
            + dist.operating_supply
            + dist.service;
        assert!((sum - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_valuation_method_distribution_default_sums_to_one() {
        let dist = ValuationMethodDistribution::default();
        let sum = dist.standard_cost + dist.moving_average + dist.fifo + dist.lifo;
        assert!((sum - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_asset_class_distribution_default_sums_to_one() {
        let dist = AssetClassDistribution::default();
        let sum = dist.buildings
            + dist.machinery
            + dist.vehicles
            + dist.it_equipment
            + dist.furniture
            + dist.land
            + dist.leasehold;
        assert!((sum - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_depreciation_method_distribution_default_sums_to_one() {
        let dist = DepreciationMethodDistribution::default();
        let sum = dist.straight_line
            + dist.declining_balance
            + dist.double_declining
            + dist.sum_of_years
            + dist.units_of_production;
        assert!((sum - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_employee_department_distribution_default_sums_to_one() {
        let dist = EmployeeDepartmentDistribution::default();
        let sum = dist.finance
            + dist.procurement
            + dist.sales
            + dist.warehouse
            + dist.it
            + dist.hr
            + dist.operations
            + dist.executive;
        assert!((sum - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_ic_transaction_type_distribution_default_sums_to_one() {
        let dist = ICTransactionTypeDistribution::default();
        let sum = dist.goods_sale
            + dist.service_provided
            + dist.loan
            + dist.dividend
            + dist.management_fee
            + dist.royalty
            + dist.cost_sharing;
        assert!((sum - 1.0).abs() < 0.001);
    }
}
