//! Configuration validation.

use crate::schema::GeneratorConfig;
use synth_core::error::{SynthError, SynthResult};

/// Validate a generator configuration.
pub fn validate_config(config: &GeneratorConfig) -> SynthResult<()> {
    // Validate global settings
    if config.global.period_months == 0 {
        return Err(SynthError::validation("period_months must be greater than 0"));
    }

    // Validate companies
    if config.companies.is_empty() {
        return Err(SynthError::validation("At least one company must be configured"));
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
    if config.fraud.enabled {
        if config.fraud.fraud_rate < 0.0 || config.fraud.fraud_rate > 1.0 {
            return Err(SynthError::validation(
                "fraud_rate must be between 0.0 and 1.0",
            ));
        }
    }

    Ok(())
}
