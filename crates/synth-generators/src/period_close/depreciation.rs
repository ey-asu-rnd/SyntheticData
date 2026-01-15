//! Depreciation run generator for period close.

use chrono::NaiveDate;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;

use synth_core::models::subledger::fa::{
    AssetStatus, DepreciationEntry, DepreciationMethod, DepreciationRun, DepreciationRunStatus,
    FixedAssetRecord,
};
use synth_core::models::{FiscalPeriod, JournalEntry, JournalEntryLine};

/// Configuration for depreciation run.
#[derive(Debug, Clone)]
pub struct DepreciationRunConfig {
    /// Default depreciation expense account.
    pub default_expense_account: String,
    /// Default accumulated depreciation account.
    pub default_accum_depr_account: String,
    /// Whether to post zero depreciation entries.
    pub post_zero_entries: bool,
    /// Minimum depreciation amount to post.
    pub minimum_amount: Decimal,
}

impl Default for DepreciationRunConfig {
    fn default() -> Self {
        Self {
            default_expense_account: "6100".to_string(),
            default_accum_depr_account: "1510".to_string(),
            post_zero_entries: false,
            minimum_amount: dec!(0.01),
        }
    }
}

/// Generator for depreciation runs.
pub struct DepreciationRunGenerator {
    config: DepreciationRunConfig,
    run_counter: u64,
}

impl DepreciationRunGenerator {
    /// Creates a new depreciation run generator.
    pub fn new(config: DepreciationRunConfig) -> Self {
        Self {
            config,
            run_counter: 0,
        }
    }

    /// Executes a depreciation run for a company.
    pub fn execute_run(
        &mut self,
        company_code: &str,
        assets: &mut [FixedAssetRecord],
        fiscal_period: &FiscalPeriod,
    ) -> DepreciationRunResult {
        self.run_counter += 1;
        let run_id = format!("DEPR-{}-{:08}", company_code, self.run_counter);

        let mut run = DepreciationRun {
            run_id: run_id.clone(),
            company_code: company_code.to_string(),
            fiscal_year: fiscal_period.year,
            fiscal_period: fiscal_period.period,
            run_date: fiscal_period.end_date,
            status: DepreciationRunStatus::Processing,
            assets_processed: 0,
            assets_with_errors: 0,
            total_depreciation: Decimal::ZERO,
            posted_by: Some("SYSTEM".to_string()),
            posted_at: Some(fiscal_period.end_date),
        };

        let mut entries = Vec::new();
        let mut journal_entries = Vec::new();
        let mut errors = Vec::new();

        for asset in assets.iter_mut() {
            // Skip non-active assets
            if asset.status != AssetStatus::Active {
                continue;
            }

            // Skip if company doesn't match
            if asset.company_code != company_code {
                continue;
            }

            // Skip if already depreciated this period
            if let Some(last_date) = asset.last_depreciation_date {
                if last_date >= fiscal_period.end_date {
                    continue;
                }
            }

            // Calculate depreciation
            match self.calculate_depreciation(asset, fiscal_period) {
                Ok(amount) => {
                    if amount < self.config.minimum_amount && !self.config.post_zero_entries {
                        continue;
                    }

                    let entry = DepreciationEntry {
                        entry_id: format!("{}-{}", run_id, asset.asset_id),
                        run_id: run_id.clone(),
                        asset_id: asset.asset_id.clone(),
                        fiscal_year: fiscal_period.year,
                        fiscal_period: fiscal_period.period,
                        depreciation_amount: amount,
                        accumulated_before: asset.accumulated_depreciation,
                        accumulated_after: asset.accumulated_depreciation + amount,
                        net_book_value_before: asset.net_book_value,
                        net_book_value_after: asset.net_book_value - amount,
                        depreciation_method: asset.depreciation_method.clone(),
                        posting_date: fiscal_period.end_date,
                        document_number: None,
                    };

                    // Generate journal entry
                    let je = self.generate_depreciation_je(asset, &entry, fiscal_period);

                    // Update asset
                    asset.accumulated_depreciation += amount;
                    asset.net_book_value -= amount;
                    asset.ytd_depreciation += amount;
                    asset.last_depreciation_date = Some(fiscal_period.end_date);

                    // Check if fully depreciated
                    if asset.net_book_value <= asset.salvage_value {
                        asset.status = AssetStatus::FullyDepreciated;
                    }

                    run.total_depreciation += amount;
                    run.assets_processed += 1;

                    entries.push(entry);
                    journal_entries.push(je);
                }
                Err(e) => {
                    run.assets_with_errors += 1;
                    errors.push(DepreciationError {
                        asset_id: asset.asset_id.clone(),
                        error: e,
                    });
                }
            }
        }

        run.status = if errors.is_empty() {
            DepreciationRunStatus::Completed
        } else if run.assets_processed > 0 {
            DepreciationRunStatus::CompletedWithErrors
        } else {
            DepreciationRunStatus::Failed
        };

        DepreciationRunResult {
            run,
            entries,
            journal_entries,
            errors,
        }
    }

    /// Calculates depreciation for a single asset.
    fn calculate_depreciation(
        &self,
        asset: &FixedAssetRecord,
        period: &FiscalPeriod,
    ) -> Result<Decimal, String> {
        let depreciable_amount = asset.current_acquisition_cost - asset.salvage_value;

        if depreciable_amount <= Decimal::ZERO {
            return Ok(Decimal::ZERO);
        }

        if asset.net_book_value <= asset.salvage_value {
            return Ok(Decimal::ZERO);
        }

        let remaining_value = asset.net_book_value - asset.salvage_value;

        match &asset.depreciation_method {
            DepreciationMethod::StraightLine => {
                let monthly = depreciable_amount / Decimal::from(asset.useful_life_months);
                Ok(monthly.min(remaining_value).round_dp(2))
            }
            DepreciationMethod::DecliningBalance { rate } => {
                let annual = asset.net_book_value * rate;
                let monthly = annual / dec!(12);
                Ok(monthly.min(remaining_value).round_dp(2))
            }
            DepreciationMethod::DoubleDecliningBalance => {
                let rate = dec!(2) / Decimal::from(asset.useful_life_months) * dec!(12);
                let annual = asset.net_book_value * rate;
                let monthly = annual / dec!(12);
                Ok(monthly.min(remaining_value).round_dp(2))
            }
            DepreciationMethod::SumOfYearsDigits => {
                let total_months = asset.useful_life_months as i64;
                let sum_of_digits = total_months * (total_months + 1) / 2;

                let months_elapsed = asset
                    .last_depreciation_date
                    .map(|d| {
                        let days = (d - asset.depreciation_start_date).num_days();
                        (days / 30) as i64
                    })
                    .unwrap_or(0);

                let remaining_months = (total_months - months_elapsed).max(1);
                let factor = Decimal::from(remaining_months) / Decimal::from(sum_of_digits);
                let annual = depreciable_amount * factor;
                let monthly = annual / dec!(12);

                Ok(monthly.min(remaining_value).round_dp(2))
            }
            DepreciationMethod::UnitsOfProduction {
                total_units,
                units_this_period,
            } => {
                if *total_units <= Decimal::ZERO {
                    return Err("Total units must be positive".to_string());
                }
                let per_unit = depreciable_amount / *total_units;
                let depreciation = per_unit * *units_this_period;
                Ok(depreciation.min(remaining_value).round_dp(2))
            }
            DepreciationMethod::NoDepreciation => Ok(Decimal::ZERO),
        }
    }

    /// Generates the journal entry for a depreciation entry.
    fn generate_depreciation_je(
        &self,
        asset: &FixedAssetRecord,
        entry: &DepreciationEntry,
        period: &FiscalPeriod,
    ) -> JournalEntry {
        let expense_account = if asset.depreciation_expense_account.is_empty() {
            &self.config.default_expense_account
        } else {
            &asset.depreciation_expense_account
        };

        let accum_account = if asset.accumulated_depreciation_account.is_empty() {
            &self.config.default_accum_depr_account
        } else {
            &asset.accumulated_depreciation_account
        };

        let mut je = JournalEntry::new_simple(
            format!("DEPR-{}-{}", entry.run_id, asset.asset_id),
            asset.company_code.clone(),
            period.end_date,
            format!(
                "Depreciation {} P{}/{}",
                asset.asset_id, period.year, period.period
            ),
        );

        // Debit Depreciation Expense
        je.add_line(JournalEntryLine {
            line_number: 1,
            gl_account: expense_account.to_string(),
            debit_amount: entry.depreciation_amount,
            cost_center: asset.cost_center.clone(),
            profit_center: asset.profit_center.clone(),
            reference: Some(asset.asset_id.clone()),
            assignment: Some(asset.asset_class.clone()),
            text: Some(asset.description.clone()),
            ..Default::default()
        });

        // Credit Accumulated Depreciation
        je.add_line(JournalEntryLine {
            line_number: 2,
            gl_account: accum_account.to_string(),
            credit_amount: entry.depreciation_amount,
            reference: Some(asset.asset_id.clone()),
            assignment: Some(asset.asset_class.clone()),
            ..Default::default()
        });

        je
    }

    /// Generates a depreciation forecast for planning purposes.
    pub fn forecast_depreciation(
        &self,
        assets: &[FixedAssetRecord],
        start_period: &FiscalPeriod,
        months: u32,
    ) -> Vec<DepreciationForecastEntry> {
        let mut forecast = Vec::new();

        // Create simulated asset states
        let mut simulated_assets: Vec<SimulatedAsset> = assets
            .iter()
            .filter(|a| a.status == AssetStatus::Active)
            .map(|a| SimulatedAsset {
                asset_id: a.asset_id.clone(),
                net_book_value: a.net_book_value,
                salvage_value: a.salvage_value,
                useful_life_months: a.useful_life_months,
                depreciation_method: a.depreciation_method.clone(),
                monthly_depreciation: self.calculate_monthly_straight_line(a),
            })
            .collect();

        let mut current_year = start_period.year;
        let mut current_month = start_period.period;

        for _ in 0..months {
            let period_key = format!("{}-{:02}", current_year, current_month);
            let mut period_total = Decimal::ZERO;

            for sim_asset in &mut simulated_assets {
                let remaining = sim_asset.net_book_value - sim_asset.salvage_value;
                if remaining > Decimal::ZERO {
                    let depr = sim_asset.monthly_depreciation.min(remaining);
                    sim_asset.net_book_value -= depr;
                    period_total += depr;
                }
            }

            forecast.push(DepreciationForecastEntry {
                period_key,
                fiscal_year: current_year,
                fiscal_period: current_month,
                forecasted_depreciation: period_total,
            });

            // Advance to next month
            if current_month == 12 {
                current_month = 1;
                current_year += 1;
            } else {
                current_month += 1;
            }
        }

        forecast
    }

    fn calculate_monthly_straight_line(&self, asset: &FixedAssetRecord) -> Decimal {
        let depreciable = asset.current_acquisition_cost - asset.salvage_value;
        if depreciable <= Decimal::ZERO || asset.useful_life_months == 0 {
            return Decimal::ZERO;
        }
        (depreciable / Decimal::from(asset.useful_life_months)).round_dp(2)
    }
}

/// Simulated asset state for forecasting.
struct SimulatedAsset {
    asset_id: String,
    net_book_value: Decimal,
    salvage_value: Decimal,
    useful_life_months: u32,
    depreciation_method: DepreciationMethod,
    monthly_depreciation: Decimal,
}

/// Result of a depreciation run.
#[derive(Debug, Clone)]
pub struct DepreciationRunResult {
    /// The depreciation run record.
    pub run: DepreciationRun,
    /// Individual depreciation entries.
    pub entries: Vec<DepreciationEntry>,
    /// Generated journal entries.
    pub journal_entries: Vec<JournalEntry>,
    /// Errors encountered.
    pub errors: Vec<DepreciationError>,
}

impl DepreciationRunResult {
    /// Returns true if the run completed successfully.
    pub fn is_success(&self) -> bool {
        matches!(
            self.run.status,
            DepreciationRunStatus::Completed | DepreciationRunStatus::CompletedWithErrors
        )
    }

    /// Returns the total depreciation amount.
    pub fn total_depreciation(&self) -> Decimal {
        self.run.total_depreciation
    }
}

/// Error during depreciation processing.
#[derive(Debug, Clone)]
pub struct DepreciationError {
    /// Asset ID.
    pub asset_id: String,
    /// Error message.
    pub error: String,
}

/// Depreciation forecast entry.
#[derive(Debug, Clone)]
pub struct DepreciationForecastEntry {
    /// Period key (YYYY-MM).
    pub period_key: String,
    /// Fiscal year.
    pub fiscal_year: i32,
    /// Fiscal period.
    pub fiscal_period: u8,
    /// Forecasted depreciation amount.
    pub forecasted_depreciation: Decimal,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_asset() -> FixedAssetRecord {
        FixedAssetRecord {
            asset_id: "FA00001".to_string(),
            company_code: "1000".to_string(),
            asset_class: "MACHINERY".to_string(),
            description: "Test Machine".to_string(),
            serial_number: None,
            inventory_number: None,
            acquisition_date: NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            capitalization_date: NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            original_acquisition_cost: dec!(120000),
            current_acquisition_cost: dec!(120000),
            salvage_value: dec!(12000),
            useful_life_months: 60,
            depreciation_method: DepreciationMethod::StraightLine,
            depreciation_start_date: NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            accumulated_depreciation: Decimal::ZERO,
            net_book_value: dec!(120000),
            ytd_depreciation: Decimal::ZERO,
            currency: "USD".to_string(),
            cost_center: Some("CC100".to_string()),
            profit_center: None,
            location: None,
            responsible_person: None,
            status: AssetStatus::Active,
            last_depreciation_date: None,
            disposal_date: None,
            disposal_proceeds: None,
            asset_account: "1500".to_string(),
            accumulated_depreciation_account: "1510".to_string(),
            depreciation_expense_account: "6100".to_string(),
        }
    }

    #[test]
    fn test_depreciation_run() {
        let mut generator = DepreciationRunGenerator::new(DepreciationRunConfig::default());
        let mut assets = vec![create_test_asset()];
        let period = FiscalPeriod::monthly(2024, 1);

        let result = generator.execute_run("1000", &mut assets, &period);

        assert!(result.is_success());
        assert_eq!(result.entries.len(), 1);
        assert!(result.journal_entries.iter().all(|je| je.is_balanced()));

        // Monthly depreciation should be (120000 - 12000) / 60 = 1800
        assert_eq!(result.total_depreciation(), dec!(1800));
    }

    #[test]
    fn test_depreciation_forecast() {
        let generator = DepreciationRunGenerator::new(DepreciationRunConfig::default());
        let assets = vec![create_test_asset()];
        let period = FiscalPeriod::monthly(2024, 1);

        let forecast = generator.forecast_depreciation(&assets, &period, 12);

        assert_eq!(forecast.len(), 12);
        assert!(forecast
            .iter()
            .all(|f| f.forecasted_depreciation == dec!(1800)));
    }
}
