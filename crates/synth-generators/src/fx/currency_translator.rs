//! Currency translation for financial statements.
//!
//! Translates trial balances and financial statements from local currency
//! to group reporting currency using appropriate translation methods.

use chrono::NaiveDate;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use std::collections::HashMap;

use synth_core::models::balance::TrialBalance;
use synth_core::models::{
    FxRateTable, RateType, TranslatedAmount, TranslationAccountType, TranslationMethod,
};

/// Configuration for currency translation.
#[derive(Debug, Clone)]
pub struct CurrencyTranslatorConfig {
    /// Translation method to use.
    pub method: TranslationMethod,
    /// Group (reporting) currency.
    pub group_currency: String,
    /// Account type mappings (account code prefix -> translation account type).
    pub account_type_map: HashMap<String, TranslationAccountType>,
    /// Equity accounts that use historical rates.
    pub historical_rate_accounts: Vec<String>,
    /// Retained earnings account code.
    pub retained_earnings_account: String,
    /// CTA (Currency Translation Adjustment) account code.
    pub cta_account: String,
}

impl Default for CurrencyTranslatorConfig {
    fn default() -> Self {
        let mut account_type_map = HashMap::new();
        // Assets
        account_type_map.insert("1".to_string(), TranslationAccountType::Asset);
        // Liabilities
        account_type_map.insert("2".to_string(), TranslationAccountType::Liability);
        // Equity
        account_type_map.insert("3".to_string(), TranslationAccountType::Equity);
        // Revenue
        account_type_map.insert("4".to_string(), TranslationAccountType::Revenue);
        // Expenses
        account_type_map.insert("5".to_string(), TranslationAccountType::Expense);
        account_type_map.insert("6".to_string(), TranslationAccountType::Expense);

        Self {
            method: TranslationMethod::CurrentRate,
            group_currency: "USD".to_string(),
            account_type_map,
            historical_rate_accounts: vec![
                "3100".to_string(), // Common Stock
                "3200".to_string(), // APIC
            ],
            retained_earnings_account: "3300".to_string(),
            cta_account: "3900".to_string(),
        }
    }
}

/// Currency translator for financial statements.
pub struct CurrencyTranslator {
    config: CurrencyTranslatorConfig,
}

impl CurrencyTranslator {
    /// Creates a new currency translator.
    pub fn new(config: CurrencyTranslatorConfig) -> Self {
        Self { config }
    }

    /// Translates a trial balance from local to group currency.
    pub fn translate_trial_balance(
        &self,
        trial_balance: &TrialBalance,
        rate_table: &FxRateTable,
        historical_rates: &HashMap<String, Decimal>,
    ) -> TranslatedTrialBalance {
        let local_currency = &trial_balance.currency;
        let period_end = trial_balance.period_end_date;

        // Get closing and average rates
        let closing_rate = rate_table
            .get_closing_rate(local_currency, &self.config.group_currency, period_end)
            .map(|r| r.rate)
            .unwrap_or(Decimal::ONE);

        let average_rate = rate_table
            .get_average_rate(local_currency, &self.config.group_currency, period_end)
            .map(|r| r.rate)
            .unwrap_or(closing_rate);

        let mut translated_lines = Vec::new();
        let mut total_local_debit = Decimal::ZERO;
        let mut total_local_credit = Decimal::ZERO;
        let mut total_group_debit = Decimal::ZERO;
        let mut total_group_credit = Decimal::ZERO;

        for line in &trial_balance.lines {
            let account_type = self.determine_account_type(&line.account_code);
            let rate = self.determine_rate(
                &line.account_code,
                &account_type,
                closing_rate,
                average_rate,
                historical_rates,
            );

            let group_debit = (line.debit_balance * rate).round_dp(2);
            let group_credit = (line.credit_balance * rate).round_dp(2);

            translated_lines.push(TranslatedTrialBalanceLine {
                account_code: line.account_code.clone(),
                account_description: line.account_description.clone(),
                account_type: account_type.clone(),
                local_debit: line.debit_balance,
                local_credit: line.credit_balance,
                rate_used: rate,
                rate_type: self.rate_type_for_account(&account_type),
                group_debit,
                group_credit,
            });

            total_local_debit += line.debit_balance;
            total_local_credit += line.credit_balance;
            total_group_debit += group_debit;
            total_group_credit += group_credit;
        }

        // Calculate CTA to balance the translated trial balance
        let cta_amount = total_group_debit - total_group_credit;

        TranslatedTrialBalance {
            company_code: trial_balance.company_code.clone(),
            company_name: trial_balance.company_name.clone(),
            local_currency: local_currency.clone(),
            group_currency: self.config.group_currency.clone(),
            period_end_date: period_end,
            fiscal_year: trial_balance.fiscal_year,
            fiscal_period: trial_balance.fiscal_period,
            lines: translated_lines,
            closing_rate,
            average_rate,
            total_local_debit,
            total_local_credit,
            total_group_debit,
            total_group_credit,
            cta_amount,
            translation_method: self.config.method.clone(),
        }
    }

    /// Translates a single amount.
    pub fn translate_amount(
        &self,
        amount: Decimal,
        local_currency: &str,
        account_type: &TranslationAccountType,
        rate_table: &FxRateTable,
        date: NaiveDate,
    ) -> TranslatedAmount {
        let (rate, rate_type) = match account_type {
            TranslationAccountType::Asset | TranslationAccountType::Liability => {
                let rate = rate_table
                    .get_closing_rate(local_currency, &self.config.group_currency, date)
                    .map(|r| r.rate)
                    .unwrap_or(Decimal::ONE);
                (rate, RateType::Closing)
            }
            TranslationAccountType::Revenue | TranslationAccountType::Expense => {
                let rate = rate_table
                    .get_average_rate(local_currency, &self.config.group_currency, date)
                    .map(|r| r.rate)
                    .unwrap_or(Decimal::ONE);
                (rate, RateType::Average)
            }
            TranslationAccountType::Equity
            | TranslationAccountType::CommonStock
            | TranslationAccountType::AdditionalPaidInCapital => {
                (Decimal::ONE, RateType::Historical) // Would need actual historical rate
            }
            TranslationAccountType::RetainedEarnings => {
                // Retained earnings is a plug figure
                (Decimal::ONE, RateType::Historical)
            }
        };

        TranslatedAmount {
            local_amount: amount,
            local_currency: local_currency.to_string(),
            group_amount: (amount * rate).round_dp(2),
            group_currency: self.config.group_currency.clone(),
            rate_used: rate,
            rate_type,
            translation_date: date,
        }
    }

    /// Determines the account type based on account code.
    fn determine_account_type(&self, account_code: &str) -> TranslationAccountType {
        // Check for specific accounts first
        if self
            .config
            .historical_rate_accounts
            .contains(&account_code.to_string())
        {
            if account_code.starts_with("31") {
                return TranslationAccountType::CommonStock;
            } else if account_code.starts_with("32") {
                return TranslationAccountType::AdditionalPaidInCapital;
            }
        }

        if account_code == self.config.retained_earnings_account {
            return TranslationAccountType::RetainedEarnings;
        }

        // Use prefix mapping
        for (prefix, account_type) in &self.config.account_type_map {
            if account_code.starts_with(prefix) {
                return account_type.clone();
            }
        }

        // Default to asset
        TranslationAccountType::Asset
    }

    /// Determines the appropriate rate to use for an account.
    fn determine_rate(
        &self,
        account_code: &str,
        account_type: &TranslationAccountType,
        closing_rate: Decimal,
        average_rate: Decimal,
        historical_rates: &HashMap<String, Decimal>,
    ) -> Decimal {
        match self.config.method {
            TranslationMethod::CurrentRate => {
                match account_type {
                    TranslationAccountType::Asset | TranslationAccountType::Liability => {
                        closing_rate
                    }
                    TranslationAccountType::Revenue | TranslationAccountType::Expense => {
                        average_rate
                    }
                    TranslationAccountType::CommonStock
                    | TranslationAccountType::AdditionalPaidInCapital => {
                        // Use historical rate if available
                        historical_rates
                            .get(account_code)
                            .copied()
                            .unwrap_or(closing_rate)
                    }
                    TranslationAccountType::Equity | TranslationAccountType::RetainedEarnings => {
                        // These are typically calculated separately
                        closing_rate
                    }
                }
            }
            TranslationMethod::Temporal => {
                // Temporal method: monetary items at closing, non-monetary at historical
                match account_type {
                    TranslationAccountType::Asset => {
                        // Would need to distinguish monetary vs non-monetary
                        // For simplicity, using closing rate
                        closing_rate
                    }
                    TranslationAccountType::Liability => closing_rate,
                    _ => average_rate,
                }
            }
            TranslationMethod::MonetaryNonMonetary => closing_rate, // Simplified
        }
    }

    /// Returns the rate type for a given account type.
    fn rate_type_for_account(&self, account_type: &TranslationAccountType) -> RateType {
        match account_type {
            TranslationAccountType::Asset | TranslationAccountType::Liability => RateType::Closing,
            TranslationAccountType::Revenue | TranslationAccountType::Expense => RateType::Average,
            TranslationAccountType::Equity
            | TranslationAccountType::CommonStock
            | TranslationAccountType::AdditionalPaidInCapital
            | TranslationAccountType::RetainedEarnings => RateType::Historical,
        }
    }
}

/// Translated trial balance in group currency.
#[derive(Debug, Clone)]
pub struct TranslatedTrialBalance {
    /// Company code.
    pub company_code: String,
    /// Company name.
    pub company_name: String,
    /// Local (functional) currency.
    pub local_currency: String,
    /// Group (reporting) currency.
    pub group_currency: String,
    /// Period end date.
    pub period_end_date: NaiveDate,
    /// Fiscal year.
    pub fiscal_year: i32,
    /// Fiscal period.
    pub fiscal_period: u8,
    /// Translated line items.
    pub lines: Vec<TranslatedTrialBalanceLine>,
    /// Closing rate used.
    pub closing_rate: Decimal,
    /// Average rate used.
    pub average_rate: Decimal,
    /// Total local currency debits.
    pub total_local_debit: Decimal,
    /// Total local currency credits.
    pub total_local_credit: Decimal,
    /// Total group currency debits.
    pub total_group_debit: Decimal,
    /// Total group currency credits.
    pub total_group_credit: Decimal,
    /// Currency Translation Adjustment amount.
    pub cta_amount: Decimal,
    /// Translation method used.
    pub translation_method: TranslationMethod,
}

impl TranslatedTrialBalance {
    /// Returns true if the local currency trial balance is balanced.
    pub fn is_local_balanced(&self) -> bool {
        (self.total_local_debit - self.total_local_credit).abs() < dec!(0.01)
    }

    /// Returns true if the group currency trial balance is balanced (including CTA).
    pub fn is_group_balanced(&self) -> bool {
        let balance = self.total_group_debit - self.total_group_credit - self.cta_amount;
        balance.abs() < dec!(0.01)
    }

    /// Gets the net assets in local currency.
    pub fn local_net_assets(&self) -> Decimal {
        let assets: Decimal = self
            .lines
            .iter()
            .filter(|l| matches!(l.account_type, TranslationAccountType::Asset))
            .map(|l| l.local_debit - l.local_credit)
            .sum();

        let liabilities: Decimal = self
            .lines
            .iter()
            .filter(|l| matches!(l.account_type, TranslationAccountType::Liability))
            .map(|l| l.local_credit - l.local_debit)
            .sum();

        assets - liabilities
    }

    /// Gets the net assets in group currency.
    pub fn group_net_assets(&self) -> Decimal {
        let assets: Decimal = self
            .lines
            .iter()
            .filter(|l| matches!(l.account_type, TranslationAccountType::Asset))
            .map(|l| l.group_debit - l.group_credit)
            .sum();

        let liabilities: Decimal = self
            .lines
            .iter()
            .filter(|l| matches!(l.account_type, TranslationAccountType::Liability))
            .map(|l| l.group_credit - l.group_debit)
            .sum();

        assets - liabilities
    }
}

/// A line in a translated trial balance.
#[derive(Debug, Clone)]
pub struct TranslatedTrialBalanceLine {
    /// Account code.
    pub account_code: String,
    /// Account description.
    pub account_description: Option<String>,
    /// Account type for translation.
    pub account_type: TranslationAccountType,
    /// Debit balance in local currency.
    pub local_debit: Decimal,
    /// Credit balance in local currency.
    pub local_credit: Decimal,
    /// Exchange rate used.
    pub rate_used: Decimal,
    /// Rate type used.
    pub rate_type: RateType,
    /// Debit balance in group currency.
    pub group_debit: Decimal,
    /// Credit balance in group currency.
    pub group_credit: Decimal,
}

impl TranslatedTrialBalanceLine {
    /// Gets the net balance in local currency.
    pub fn local_net(&self) -> Decimal {
        self.local_debit - self.local_credit
    }

    /// Gets the net balance in group currency.
    pub fn group_net(&self) -> Decimal {
        self.group_debit - self.group_credit
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use synth_core::models::balance::TrialBalanceLine;
    use synth_core::models::FxRate;

    fn create_test_trial_balance() -> TrialBalance {
        TrialBalance {
            company_code: "1200".to_string(),
            company_name: "Test Subsidiary".to_string(),
            currency: "EUR".to_string(),
            period_end_date: NaiveDate::from_ymd_opt(2024, 12, 31).unwrap(),
            fiscal_year: 2024,
            fiscal_period: 12,
            lines: vec![
                TrialBalanceLine {
                    account_code: "1000".to_string(),
                    account_description: Some("Cash".to_string()),
                    debit_balance: dec!(100000),
                    credit_balance: Decimal::ZERO,
                    account_category: None,
                },
                TrialBalanceLine {
                    account_code: "2000".to_string(),
                    account_description: Some("Accounts Payable".to_string()),
                    debit_balance: Decimal::ZERO,
                    credit_balance: dec!(50000),
                    account_category: None,
                },
                TrialBalanceLine {
                    account_code: "4000".to_string(),
                    account_description: Some("Revenue".to_string()),
                    debit_balance: Decimal::ZERO,
                    credit_balance: dec!(150000),
                    account_category: None,
                },
                TrialBalanceLine {
                    account_code: "5000".to_string(),
                    account_description: Some("Expenses".to_string()),
                    debit_balance: dec!(100000),
                    credit_balance: Decimal::ZERO,
                    account_category: None,
                },
            ],
            total_debits: dec!(200000),
            total_credits: dec!(200000),
        }
    }

    #[test]
    fn test_translate_trial_balance() {
        let translator = CurrencyTranslator::new(CurrencyTranslatorConfig::default());
        let trial_balance = create_test_trial_balance();

        let mut rate_table = FxRateTable::new("USD");
        rate_table.add_rate(FxRate::new(
            "EUR",
            "USD",
            RateType::Closing,
            NaiveDate::from_ymd_opt(2024, 12, 31).unwrap(),
            dec!(1.10),
            "TEST",
        ));
        rate_table.add_rate(FxRate::new(
            "EUR",
            "USD",
            RateType::Average,
            NaiveDate::from_ymd_opt(2024, 12, 31).unwrap(),
            dec!(1.08),
            "TEST",
        ));

        let historical_rates = HashMap::new();
        let translated =
            translator.translate_trial_balance(&trial_balance, &rate_table, &historical_rates);

        assert!(translated.is_local_balanced());
        assert_eq!(translated.closing_rate, dec!(1.10));
        assert_eq!(translated.average_rate, dec!(1.08));
    }
}
