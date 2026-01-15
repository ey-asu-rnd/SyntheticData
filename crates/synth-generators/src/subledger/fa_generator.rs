//! Fixed Assets (FA) generator.

use chrono::NaiveDate;
use rand::Rng;
use rand_chacha::ChaCha8Rng;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;

use synth_core::models::subledger::fa::{
    AssetDisposal, AssetStatus, DepreciationEntry, DepreciationMethod, DepreciationRun,
    DepreciationRunStatus, DisposalReason, DisposalType, FixedAssetRecord,
};
use synth_core::models::{JournalEntry, JournalEntryLine};

/// Configuration for FA generation.
#[derive(Debug, Clone)]
pub struct FAGeneratorConfig {
    /// Default depreciation method.
    pub default_depreciation_method: DepreciationMethod,
    /// Default useful life in months.
    pub default_useful_life_months: u32,
    /// Salvage value percentage.
    pub salvage_value_percent: Decimal,
    /// Average acquisition cost.
    pub avg_acquisition_cost: Decimal,
    /// Cost variation factor.
    pub cost_variation: Decimal,
    /// Disposal rate per year.
    pub annual_disposal_rate: Decimal,
}

impl Default for FAGeneratorConfig {
    fn default() -> Self {
        Self {
            default_depreciation_method: DepreciationMethod::StraightLine,
            default_useful_life_months: 60,
            salvage_value_percent: dec!(0.10),
            avg_acquisition_cost: dec!(50000),
            cost_variation: dec!(0.7),
            annual_disposal_rate: dec!(0.05),
        }
    }
}

/// Generator for Fixed Assets transactions.
pub struct FAGenerator {
    config: FAGeneratorConfig,
    rng: ChaCha8Rng,
    asset_counter: u64,
    depreciation_run_counter: u64,
    disposal_counter: u64,
}

impl FAGenerator {
    /// Creates a new FA generator.
    pub fn new(config: FAGeneratorConfig, rng: ChaCha8Rng) -> Self {
        Self {
            config,
            rng,
            asset_counter: 0,
            depreciation_run_counter: 0,
            disposal_counter: 0,
        }
    }

    /// Generates a new fixed asset acquisition.
    pub fn generate_asset_acquisition(
        &mut self,
        company_code: &str,
        asset_class: &str,
        description: &str,
        acquisition_date: NaiveDate,
        currency: &str,
        cost_center: Option<&str>,
    ) -> (FixedAssetRecord, JournalEntry) {
        self.asset_counter += 1;
        let asset_id = format!("FA{:08}", self.asset_counter);

        let acquisition_cost = self.generate_acquisition_cost();
        let salvage_value = (acquisition_cost * self.config.salvage_value_percent).round_dp(2);

        let asset = FixedAssetRecord {
            asset_id: asset_id.clone(),
            company_code: company_code.to_string(),
            asset_class: asset_class.to_string(),
            description: description.to_string(),
            serial_number: Some(format!("SN-{:010}", self.rng.gen::<u32>())),
            inventory_number: Some(format!("INV-{:08}", self.asset_counter)),
            acquisition_date,
            capitalization_date: acquisition_date,
            original_acquisition_cost: acquisition_cost,
            current_acquisition_cost: acquisition_cost,
            salvage_value,
            useful_life_months: self.config.default_useful_life_months,
            depreciation_method: self.config.default_depreciation_method.clone(),
            depreciation_start_date: acquisition_date,
            accumulated_depreciation: Decimal::ZERO,
            net_book_value: acquisition_cost,
            ytd_depreciation: Decimal::ZERO,
            currency: currency.to_string(),
            cost_center: cost_center.map(|s| s.to_string()),
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
        };

        let je = self.generate_acquisition_je(&asset);
        (asset, je)
    }

    /// Runs depreciation for a period.
    pub fn run_depreciation(
        &mut self,
        company_code: &str,
        assets: &[&FixedAssetRecord],
        period_date: NaiveDate,
        fiscal_year: i32,
        fiscal_period: u8,
    ) -> (DepreciationRun, Vec<DepreciationEntry>, Vec<JournalEntry>) {
        self.depreciation_run_counter += 1;
        let run_id = format!("DEPR{:08}", self.depreciation_run_counter);

        let mut run = DepreciationRun {
            run_id: run_id.clone(),
            company_code: company_code.to_string(),
            fiscal_year,
            fiscal_period,
            run_date: period_date,
            status: DepreciationRunStatus::Processing,
            assets_processed: 0,
            assets_with_errors: 0,
            total_depreciation: Decimal::ZERO,
            posted_by: None,
            posted_at: None,
        };

        let mut entries = Vec::new();
        let mut journal_entries = Vec::new();

        for asset in assets {
            if asset.status != AssetStatus::Active {
                continue;
            }

            let depreciation_amount = self.calculate_monthly_depreciation(asset);
            if depreciation_amount <= Decimal::ZERO {
                continue;
            }

            let entry = DepreciationEntry {
                entry_id: format!("{}-{}", run_id, asset.asset_id),
                run_id: run_id.clone(),
                asset_id: asset.asset_id.clone(),
                fiscal_year,
                fiscal_period,
                depreciation_amount,
                accumulated_before: asset.accumulated_depreciation,
                accumulated_after: asset.accumulated_depreciation + depreciation_amount,
                net_book_value_before: asset.net_book_value,
                net_book_value_after: asset.net_book_value - depreciation_amount,
                depreciation_method: asset.depreciation_method.clone(),
                posting_date: period_date,
                document_number: None,
            };

            let je = self.generate_depreciation_je(asset, &entry, period_date);

            run.total_depreciation += depreciation_amount;
            run.assets_processed += 1;

            entries.push(entry);
            journal_entries.push(je);
        }

        run.status = DepreciationRunStatus::Completed;

        (run, entries, journal_entries)
    }

    /// Generates an asset disposal.
    pub fn generate_disposal(
        &mut self,
        asset: &FixedAssetRecord,
        disposal_date: NaiveDate,
        disposal_type: DisposalType,
        proceeds: Decimal,
    ) -> (AssetDisposal, JournalEntry) {
        self.disposal_counter += 1;
        let disposal_id = format!("DISP{:08}", self.disposal_counter);

        let gain_loss = proceeds - asset.net_book_value;

        let disposal = AssetDisposal {
            disposal_id: disposal_id.clone(),
            asset_id: asset.asset_id.clone(),
            company_code: asset.company_code.clone(),
            disposal_date,
            disposal_type,
            disposal_reason: self.random_disposal_reason(),
            proceeds,
            net_book_value_at_disposal: asset.net_book_value,
            accumulated_depreciation_at_disposal: asset.accumulated_depreciation,
            gain_loss,
            gain_loss_account: if gain_loss >= Decimal::ZERO {
                "4900".to_string() // Gain on disposal
            } else {
                "6900".to_string() // Loss on disposal
            },
            buyer_name: Some(format!("Buyer-{}", self.disposal_counter)),
            buyer_id: None,
            invoice_number: Some(format!("SALE-{:08}", self.disposal_counter)),
            approval_status: synth_core::models::subledger::fa::DisposalApprovalStatus::Approved,
            approved_by: Some("SYSTEM".to_string()),
            approved_at: Some(disposal_date),
            document_number: None,
            notes: None,
        };

        let je = self.generate_disposal_je(asset, &disposal);
        (disposal, je)
    }

    fn generate_acquisition_cost(&mut self) -> Decimal {
        let base = self.config.avg_acquisition_cost;
        let variation = base * self.config.cost_variation;
        let random: f64 = self.rng.gen_range(-1.0..1.0);
        (base + variation * Decimal::try_from(random).unwrap_or_default())
            .max(dec!(1000))
            .round_dp(2)
    }

    fn calculate_monthly_depreciation(&self, asset: &FixedAssetRecord) -> Decimal {
        let depreciable_amount = asset.current_acquisition_cost - asset.salvage_value;

        if depreciable_amount <= Decimal::ZERO || asset.net_book_value <= asset.salvage_value {
            return Decimal::ZERO;
        }

        match &asset.depreciation_method {
            DepreciationMethod::StraightLine => {
                let monthly = depreciable_amount / Decimal::from(asset.useful_life_months);
                monthly
                    .min(asset.net_book_value - asset.salvage_value)
                    .round_dp(2)
            }
            DepreciationMethod::DecliningBalance { rate } => {
                let annual = asset.net_book_value * rate;
                let monthly = annual / dec!(12);
                monthly
                    .min(asset.net_book_value - asset.salvage_value)
                    .round_dp(2)
            }
            DepreciationMethod::DoubleDecliningBalance => {
                let rate = dec!(2) / Decimal::from(asset.useful_life_months) * dec!(12);
                let annual = asset.net_book_value * rate;
                let monthly = annual / dec!(12);
                monthly
                    .min(asset.net_book_value - asset.salvage_value)
                    .round_dp(2)
            }
            DepreciationMethod::SumOfYearsDigits => {
                // Simplified calculation
                let total_months = asset.useful_life_months as i64;
                let sum_of_digits = total_months * (total_months + 1) / 2;
                let elapsed = asset
                    .last_depreciation_date
                    .map(|d| {
                        let days = (d - asset.depreciation_start_date).num_days();
                        (days / 30) as i64
                    })
                    .unwrap_or(0);
                let remaining = (total_months - elapsed).max(1);
                let factor = Decimal::from(remaining) / Decimal::from(sum_of_digits);
                let annual = depreciable_amount * factor;
                (annual / dec!(12))
                    .min(asset.net_book_value - asset.salvage_value)
                    .round_dp(2)
            }
            DepreciationMethod::UnitsOfProduction {
                total_units,
                units_this_period,
            } => {
                if *total_units <= Decimal::ZERO {
                    return Decimal::ZERO;
                }
                let per_unit = depreciable_amount / *total_units;
                (per_unit * *units_this_period)
                    .min(asset.net_book_value - asset.salvage_value)
                    .round_dp(2)
            }
            DepreciationMethod::NoDepreciation => Decimal::ZERO,
        }
    }

    fn random_disposal_reason(&mut self) -> DisposalReason {
        match self.rng.gen_range(0..5) {
            0 => DisposalReason::Sale,
            1 => DisposalReason::Scrapped,
            2 => DisposalReason::Obsolete,
            3 => DisposalReason::Donated,
            _ => DisposalReason::Other("End of useful life".to_string()),
        }
    }

    fn generate_acquisition_je(&self, asset: &FixedAssetRecord) -> JournalEntry {
        let mut je = JournalEntry::new_simple(
            format!("JE-ACQ-{}", asset.asset_id),
            asset.company_code.clone(),
            asset.acquisition_date,
            format!("Asset Acquisition {}", asset.asset_id),
        );

        // Debit Fixed Asset
        je.add_line(JournalEntryLine {
            line_number: 1,
            gl_account: asset.asset_account.clone(),
            debit_amount: asset.original_acquisition_cost,
            cost_center: asset.cost_center.clone(),
            profit_center: asset.profit_center.clone(),
            reference: Some(asset.asset_id.clone()),
            text: Some(asset.description.clone()),
            quantity: Some(dec!(1)),
            unit: Some("EA".to_string()),
            ..Default::default()
        });

        // Credit Cash/AP (assuming cash purchase)
        je.add_line(JournalEntryLine {
            line_number: 2,
            gl_account: "1000".to_string(),
            credit_amount: asset.original_acquisition_cost,
            reference: Some(asset.asset_id.clone()),
            ..Default::default()
        });

        je
    }

    fn generate_depreciation_je(
        &self,
        asset: &FixedAssetRecord,
        entry: &DepreciationEntry,
        posting_date: NaiveDate,
    ) -> JournalEntry {
        let mut je = JournalEntry::new_simple(
            format!("JE-DEP-{}-{}", entry.run_id, asset.asset_id),
            asset.company_code.clone(),
            posting_date,
            format!(
                "Depreciation {} Period {}",
                asset.asset_id, entry.fiscal_period
            ),
        );

        // Debit Depreciation Expense
        je.add_line(JournalEntryLine {
            line_number: 1,
            gl_account: asset.depreciation_expense_account.clone(),
            debit_amount: entry.depreciation_amount,
            cost_center: asset.cost_center.clone(),
            profit_center: asset.profit_center.clone(),
            reference: Some(asset.asset_id.clone()),
            ..Default::default()
        });

        // Credit Accumulated Depreciation
        je.add_line(JournalEntryLine {
            line_number: 2,
            gl_account: asset.accumulated_depreciation_account.clone(),
            credit_amount: entry.depreciation_amount,
            reference: Some(asset.asset_id.clone()),
            ..Default::default()
        });

        je
    }

    fn generate_disposal_je(
        &self,
        asset: &FixedAssetRecord,
        disposal: &AssetDisposal,
    ) -> JournalEntry {
        let mut je = JournalEntry::new_simple(
            format!("JE-{}", disposal.disposal_id),
            asset.company_code.clone(),
            disposal.disposal_date,
            format!("Asset Disposal {}", asset.asset_id),
        );

        let mut line_num = 1;

        // Debit Cash (if proceeds > 0)
        if disposal.proceeds > Decimal::ZERO {
            je.add_line(JournalEntryLine {
                line_number: line_num,
                gl_account: "1000".to_string(),
                debit_amount: disposal.proceeds,
                reference: Some(disposal.disposal_id.clone()),
                ..Default::default()
            });
            line_num += 1;
        }

        // Debit Accumulated Depreciation
        je.add_line(JournalEntryLine {
            line_number: line_num,
            gl_account: asset.accumulated_depreciation_account.clone(),
            debit_amount: disposal.accumulated_depreciation_at_disposal,
            reference: Some(disposal.disposal_id.clone()),
            ..Default::default()
        });
        line_num += 1;

        // Debit Loss on Disposal (if loss)
        if disposal.gain_loss < Decimal::ZERO {
            je.add_line(JournalEntryLine {
                line_number: line_num,
                gl_account: disposal.gain_loss_account.clone(),
                debit_amount: disposal.gain_loss.abs(),
                cost_center: asset.cost_center.clone(),
                profit_center: asset.profit_center.clone(),
                reference: Some(disposal.disposal_id.clone()),
                ..Default::default()
            });
            line_num += 1;
        }

        // Credit Fixed Asset
        je.add_line(JournalEntryLine {
            line_number: line_num,
            gl_account: asset.asset_account.clone(),
            credit_amount: asset.current_acquisition_cost,
            reference: Some(disposal.disposal_id.clone()),
            ..Default::default()
        });
        line_num += 1;

        // Credit Gain on Disposal (if gain)
        if disposal.gain_loss > Decimal::ZERO {
            je.add_line(JournalEntryLine {
                line_number: line_num,
                gl_account: disposal.gain_loss_account.clone(),
                credit_amount: disposal.gain_loss,
                cost_center: asset.cost_center.clone(),
                profit_center: asset.profit_center.clone(),
                reference: Some(disposal.disposal_id.clone()),
                ..Default::default()
            });
        }

        je
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::SeedableRng;

    #[test]
    fn test_generate_asset_acquisition() {
        let rng = ChaCha8Rng::seed_from_u64(12345);
        let mut generator = FAGenerator::new(FAGeneratorConfig::default(), rng);

        let (asset, je) = generator.generate_asset_acquisition(
            "1000",
            "MACHINERY",
            "CNC Machine",
            NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(),
            "USD",
            Some("CC100"),
        );

        assert_eq!(asset.status, AssetStatus::Active);
        assert!(asset.original_acquisition_cost > Decimal::ZERO);
        assert!(je.is_balanced());
    }

    #[test]
    fn test_run_depreciation() {
        let rng = ChaCha8Rng::seed_from_u64(12345);
        let mut generator = FAGenerator::new(FAGeneratorConfig::default(), rng);

        let (asset, _) = generator.generate_asset_acquisition(
            "1000",
            "MACHINERY",
            "CNC Machine",
            NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(),
            "USD",
            None,
        );

        let (run, entries, jes) = generator.run_depreciation(
            "1000",
            &[&asset],
            NaiveDate::from_ymd_opt(2024, 1, 31).unwrap(),
            2024,
            1,
        );

        assert_eq!(run.status, DepreciationRunStatus::Completed);
        assert_eq!(entries.len(), 1);
        assert!(jes.iter().all(|je| je.is_balanced()));
    }
}
