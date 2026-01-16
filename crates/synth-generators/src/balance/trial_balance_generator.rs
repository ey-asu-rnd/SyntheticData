//! Trial balance generator.
//!
//! Generates trial balances at period end from running balance snapshots,
//! with support for:
//! - Unadjusted, adjusted, and post-closing trial balances
//! - Category summaries and subtotals
//! - Comparative trial balances across periods
//! - Consolidated trial balances across companies

use chrono::NaiveDate;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use std::collections::HashMap;

use synth_core::models::balance::{
    AccountBalance, AccountCategory, AccountType, BalanceSnapshot, CategorySummary,
    ComparativeTrialBalance, TrialBalance, TrialBalanceLine, TrialBalanceStatus, TrialBalanceType,
};
use synth_core::models::ChartOfAccounts;

use super::RunningBalanceTracker;

/// Configuration for trial balance generation.
#[derive(Debug, Clone)]
pub struct TrialBalanceConfig {
    /// Include zero balance accounts.
    pub include_zero_balances: bool,
    /// Group accounts by category.
    pub group_by_category: bool,
    /// Generate category subtotals.
    pub generate_subtotals: bool,
    /// Sort accounts by code.
    pub sort_by_account_code: bool,
    /// Trial balance type to generate.
    pub trial_balance_type: TrialBalanceType,
}

impl Default for TrialBalanceConfig {
    fn default() -> Self {
        Self {
            include_zero_balances: false,
            group_by_category: true,
            generate_subtotals: true,
            sort_by_account_code: true,
            trial_balance_type: TrialBalanceType::Unadjusted,
        }
    }
}

/// Generator for trial balance reports.
pub struct TrialBalanceGenerator {
    config: TrialBalanceConfig,
    /// Account category mappings.
    category_mappings: HashMap<String, AccountCategory>,
    /// Account descriptions.
    account_descriptions: HashMap<String, String>,
}

impl TrialBalanceGenerator {
    /// Creates a new trial balance generator.
    pub fn new(config: TrialBalanceConfig) -> Self {
        Self {
            config,
            category_mappings: HashMap::new(),
            account_descriptions: HashMap::new(),
        }
    }

    /// Creates a generator with default configuration.
    pub fn with_defaults() -> Self {
        Self::new(TrialBalanceConfig::default())
    }

    /// Registers category mappings from chart of accounts.
    pub fn register_from_chart(&mut self, chart: &ChartOfAccounts) {
        for account in &chart.accounts {
            self.account_descriptions
                .insert(account.account_code().to_string(), account.description().to_string());

            // Determine category from account code prefix
            let category = self.determine_category(account.account_code());
            self.category_mappings
                .insert(account.account_code().to_string(), category);
        }
    }

    /// Registers a custom category mapping.
    pub fn register_category(&mut self, account_code: &str, category: AccountCategory) {
        self.category_mappings
            .insert(account_code.to_string(), category);
    }

    /// Generates a trial balance from a balance snapshot.
    pub fn generate_from_snapshot(
        &self,
        snapshot: &BalanceSnapshot,
        fiscal_year: i32,
        fiscal_period: u32,
    ) -> TrialBalance {
        let mut lines = Vec::new();
        let mut total_debits = Decimal::ZERO;
        let mut total_credits = Decimal::ZERO;

        // Convert balances to trial balance lines
        for (account_code, balance) in &snapshot.balances {
            if !self.config.include_zero_balances && balance.closing_balance == Decimal::ZERO {
                continue;
            }

            let (debit, credit) = self.split_balance(balance);
            total_debits += debit;
            total_credits += credit;

            let category = self.determine_category(account_code);
            let description = self
                .account_descriptions
                .get(account_code)
                .cloned()
                .unwrap_or_else(|| format!("Account {}", account_code));

            lines.push(TrialBalanceLine {
                account_code: account_code.clone(),
                account_description: description,
                category,
                debit_balance: debit,
                credit_balance: credit,
                opening_balance: balance.opening_balance,
                period_debits: balance.period_debits,
                period_credits: balance.period_credits,
                closing_balance: balance.closing_balance,
            });
        }

        // Sort lines
        if self.config.sort_by_account_code {
            lines.sort_by(|a, b| a.account_code.cmp(&b.account_code));
        }

        // Calculate category summaries
        let category_summaries = if self.config.group_by_category {
            self.calculate_category_summaries(&lines)
        } else {
            Vec::new()
        };

        TrialBalance {
            company_code: snapshot.company_code.clone(),
            as_of_date: snapshot.as_of_date,
            fiscal_year,
            fiscal_period,
            trial_balance_type: self.config.trial_balance_type,
            lines,
            total_debits,
            total_credits,
            is_balanced: (total_debits - total_credits).abs() < dec!(0.01),
            category_summaries,
            status: TrialBalanceStatus::Draft,
            generated_at: chrono::Utc::now(),
            generated_by: Some("TrialBalanceGenerator".to_string()),
            notes: None,
        }
    }

    /// Generates a trial balance from the balance tracker.
    pub fn generate_from_tracker(
        &self,
        tracker: &RunningBalanceTracker,
        company_code: &str,
        as_of_date: NaiveDate,
        fiscal_year: i32,
        fiscal_period: u32,
    ) -> Option<TrialBalance> {
        tracker
            .get_snapshot(company_code, as_of_date)
            .map(|snapshot| self.generate_from_snapshot(&snapshot, fiscal_year, fiscal_period))
    }

    /// Generates trial balances for all companies in the tracker.
    pub fn generate_all_from_tracker(
        &self,
        tracker: &RunningBalanceTracker,
        as_of_date: NaiveDate,
        fiscal_year: i32,
        fiscal_period: u32,
    ) -> Vec<TrialBalance> {
        tracker
            .get_all_snapshots(as_of_date)
            .iter()
            .map(|snapshot| self.generate_from_snapshot(snapshot, fiscal_year, fiscal_period))
            .collect()
    }

    /// Generates a comparative trial balance across multiple periods.
    pub fn generate_comparative(
        &self,
        snapshots: &[(NaiveDate, BalanceSnapshot)],
        fiscal_year: i32,
    ) -> ComparativeTrialBalance {
        let periods: Vec<TrialBalance> = snapshots
            .iter()
            .enumerate()
            .map(|(i, (date, snapshot))| {
                let mut tb = self.generate_from_snapshot(snapshot, fiscal_year, (i + 1) as u32);
                tb.as_of_date = *date;
                tb
            })
            .collect();

        let variances = self.calculate_period_variances(&periods);

        ComparativeTrialBalance {
            company_code: snapshots
                .first()
                .map(|(_, s)| s.company_code.clone())
                .unwrap_or_default(),
            periods,
            variances,
        }
    }

    /// Generates a consolidated trial balance across companies.
    pub fn generate_consolidated(
        &self,
        trial_balances: &[TrialBalance],
        consolidated_company_code: &str,
    ) -> TrialBalance {
        let mut consolidated_balances: HashMap<String, TrialBalanceLine> = HashMap::new();

        for tb in trial_balances {
            for line in &tb.lines {
                let entry = consolidated_balances
                    .entry(line.account_code.clone())
                    .or_insert_with(|| TrialBalanceLine {
                        account_code: line.account_code.clone(),
                        account_description: line.account_description.clone(),
                        category: line.category,
                        debit_balance: Decimal::ZERO,
                        credit_balance: Decimal::ZERO,
                        opening_balance: Decimal::ZERO,
                        period_debits: Decimal::ZERO,
                        period_credits: Decimal::ZERO,
                        closing_balance: Decimal::ZERO,
                    });

                entry.debit_balance += line.debit_balance;
                entry.credit_balance += line.credit_balance;
                entry.opening_balance += line.opening_balance;
                entry.period_debits += line.period_debits;
                entry.period_credits += line.period_credits;
                entry.closing_balance += line.closing_balance;
            }
        }

        let mut lines: Vec<TrialBalanceLine> = consolidated_balances.into_values().collect();
        if self.config.sort_by_account_code {
            lines.sort_by(|a, b| a.account_code.cmp(&b.account_code));
        }

        let total_debits: Decimal = lines.iter().map(|l| l.debit_balance).sum();
        let total_credits: Decimal = lines.iter().map(|l| l.credit_balance).sum();

        let category_summaries = if self.config.group_by_category {
            self.calculate_category_summaries(&lines)
        } else {
            Vec::new()
        };

        let as_of_date = trial_balances
            .first()
            .map(|tb| tb.as_of_date)
            .unwrap_or_else(|| chrono::Local::now().date_naive());

        let fiscal_year = trial_balances.first().map(|tb| tb.fiscal_year).unwrap_or(0);
        let fiscal_period = trial_balances
            .first()
            .map(|tb| tb.fiscal_period)
            .unwrap_or(0);

        TrialBalance {
            company_code: consolidated_company_code.to_string(),
            as_of_date,
            fiscal_year,
            fiscal_period,
            trial_balance_type: TrialBalanceType::Consolidated,
            lines,
            total_debits,
            total_credits,
            is_balanced: (total_debits - total_credits).abs() < dec!(0.01),
            category_summaries,
            status: TrialBalanceStatus::Draft,
            generated_at: chrono::Utc::now(),
            generated_by: Some("TrialBalanceGenerator (Consolidated)".to_string()),
            notes: Some(format!(
                "Consolidated from {} companies",
                trial_balances.len()
            )),
        }
    }

    /// Splits a balance into debit and credit components.
    fn split_balance(&self, balance: &AccountBalance) -> (Decimal, Decimal) {
        let closing = balance.closing_balance;

        // Determine natural balance side based on account type
        match balance.account_type {
            AccountType::Asset | AccountType::Expense => {
                if closing >= Decimal::ZERO {
                    (closing, Decimal::ZERO)
                } else {
                    (Decimal::ZERO, closing.abs())
                }
            }
            AccountType::ContraAsset | AccountType::ContraLiability | AccountType::ContraEquity => {
                // Contra accounts have opposite natural balance
                if closing >= Decimal::ZERO {
                    (Decimal::ZERO, closing)
                } else {
                    (closing.abs(), Decimal::ZERO)
                }
            }
            AccountType::Liability | AccountType::Equity | AccountType::Revenue => {
                if closing >= Decimal::ZERO {
                    (Decimal::ZERO, closing)
                } else {
                    (closing.abs(), Decimal::ZERO)
                }
            }
        }
    }

    /// Determines account category from code prefix.
    fn determine_category(&self, account_code: &str) -> AccountCategory {
        // Check registered mappings first
        if let Some(category) = self.category_mappings.get(account_code) {
            return *category;
        }

        // Default logic based on account code ranges
        let prefix: u32 = account_code
            .chars()
            .take(2)
            .collect::<String>()
            .parse()
            .unwrap_or(0);

        match prefix {
            10..=14 => AccountCategory::CurrentAssets,
            15..=19 => AccountCategory::NonCurrentAssets,
            20..=24 => AccountCategory::CurrentLiabilities,
            25..=29 => AccountCategory::NonCurrentLiabilities,
            30..=39 => AccountCategory::Equity,
            40..=44 => AccountCategory::Revenue,
            50..=54 => AccountCategory::CostOfGoodsSold,
            55..=69 => AccountCategory::OperatingExpenses,
            70..=74 => AccountCategory::OtherIncome,
            75..=99 => AccountCategory::OtherExpenses,
            _ => AccountCategory::OtherExpenses,
        }
    }

    /// Calculates category summaries from lines.
    fn calculate_category_summaries(&self, lines: &[TrialBalanceLine]) -> Vec<CategorySummary> {
        let mut summaries: HashMap<AccountCategory, CategorySummary> = HashMap::new();

        for line in lines {
            let summary = summaries
                .entry(line.category)
                .or_insert_with(|| CategorySummary {
                    category: line.category,
                    account_count: 0,
                    total_debits: Decimal::ZERO,
                    total_credits: Decimal::ZERO,
                    net_balance: Decimal::ZERO,
                });

            summary.account_count += 1;
            summary.total_debits += line.debit_balance;
            summary.total_credits += line.credit_balance;
            summary.net_balance += line.closing_balance;
        }

        let mut result: Vec<CategorySummary> = summaries.into_values().collect();
        result.sort_by_key(|s| s.category as u8);
        result
    }

    /// Calculates variances between periods.
    fn calculate_period_variances(
        &self,
        periods: &[TrialBalance],
    ) -> HashMap<String, Vec<Decimal>> {
        let mut variances: HashMap<String, Vec<Decimal>> = HashMap::new();

        if periods.len() < 2 {
            return variances;
        }

        // Collect all account codes
        let mut all_accounts: Vec<String> = periods
            .iter()
            .flat_map(|p| p.lines.iter().map(|l| l.account_code.clone()))
            .collect();
        all_accounts.sort();
        all_accounts.dedup();

        // Calculate period-over-period variances
        for account in all_accounts {
            let mut period_variances = Vec::new();

            for i in 1..periods.len() {
                let current = periods[i]
                    .lines
                    .iter()
                    .find(|l| l.account_code == account)
                    .map(|l| l.closing_balance)
                    .unwrap_or_default();

                let previous = periods[i - 1]
                    .lines
                    .iter()
                    .find(|l| l.account_code == account)
                    .map(|l| l.closing_balance)
                    .unwrap_or_default();

                period_variances.push(current - previous);
            }

            variances.insert(account, period_variances);
        }

        variances
    }

    /// Finalizes a trial balance (changes status to Final).
    pub fn finalize(&self, mut trial_balance: TrialBalance) -> TrialBalance {
        trial_balance.status = TrialBalanceStatus::Final;
        trial_balance
    }

    /// Approves a trial balance.
    pub fn approve(&self, mut trial_balance: TrialBalance, approver: &str) -> TrialBalance {
        trial_balance.status = TrialBalanceStatus::Approved;
        trial_balance.notes = Some(format!(
            "{}Approved by {} on {}",
            trial_balance
                .notes
                .map(|n| format!("{}. ", n))
                .unwrap_or_default(),
            approver,
            chrono::Local::now().format("%Y-%m-%d %H:%M")
        ));
        trial_balance
    }
}

/// Builder for trial balance generation with fluent API.
pub struct TrialBalanceBuilder {
    generator: TrialBalanceGenerator,
    snapshots: Vec<(String, BalanceSnapshot)>,
    fiscal_year: i32,
    fiscal_period: u32,
}

impl TrialBalanceBuilder {
    /// Creates a new builder.
    pub fn new(fiscal_year: i32, fiscal_period: u32) -> Self {
        Self {
            generator: TrialBalanceGenerator::with_defaults(),
            snapshots: Vec::new(),
            fiscal_year,
            fiscal_period,
        }
    }

    /// Adds a balance snapshot.
    pub fn add_snapshot(mut self, company_code: &str, snapshot: BalanceSnapshot) -> Self {
        self.snapshots.push((company_code.to_string(), snapshot));
        self
    }

    /// Sets configuration.
    pub fn with_config(mut self, config: TrialBalanceConfig) -> Self {
        self.generator = TrialBalanceGenerator::new(config);
        self
    }

    /// Builds individual trial balances.
    pub fn build(self) -> Vec<TrialBalance> {
        self.snapshots
            .iter()
            .map(|(_, snapshot)| {
                self.generator.generate_from_snapshot(
                    snapshot,
                    self.fiscal_year,
                    self.fiscal_period,
                )
            })
            .collect()
    }

    /// Builds a consolidated trial balance.
    pub fn build_consolidated(self, consolidated_code: &str) -> TrialBalance {
        let individual = self
            .snapshots
            .iter()
            .map(|(_, snapshot)| {
                self.generator.generate_from_snapshot(
                    snapshot,
                    self.fiscal_year,
                    self.fiscal_period,
                )
            })
            .collect::<Vec<_>>();

        self.generator
            .generate_consolidated(&individual, consolidated_code)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn create_test_snapshot() -> BalanceSnapshot {
        let mut balances = HashMap::new();

        // Assets
        balances.insert(
            "1100".to_string(),
            AccountBalance::new(
                "1100".to_string(),
                "TEST".to_string(),
                AccountType::Asset,
                NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            )
            .with_opening_balance(dec!(10000)),
        );

        // Liabilities
        balances.insert(
            "2100".to_string(),
            AccountBalance::new(
                "2100".to_string(),
                "TEST".to_string(),
                AccountType::Liability,
                NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            )
            .with_opening_balance(dec!(5000)),
        );

        // Equity
        balances.insert(
            "3100".to_string(),
            AccountBalance::new(
                "3100".to_string(),
                "TEST".to_string(),
                AccountType::Equity,
                NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            )
            .with_opening_balance(dec!(5000)),
        );

        BalanceSnapshot::new(
            NaiveDate::from_ymd_opt(2024, 1, 31).unwrap(),
            "TEST".to_string(),
            balances,
        )
    }

    #[test]
    fn test_generate_trial_balance() {
        let generator = TrialBalanceGenerator::with_defaults();
        let snapshot = create_test_snapshot();

        let tb = generator.generate_from_snapshot(&snapshot, 2024, 1);

        assert!(tb.is_balanced);
        assert_eq!(tb.lines.len(), 3);
        assert_eq!(tb.total_debits, dec!(10000));
        assert_eq!(tb.total_credits, dec!(10000));
    }

    #[test]
    fn test_category_summaries() {
        let generator = TrialBalanceGenerator::with_defaults();
        let snapshot = create_test_snapshot();

        let tb = generator.generate_from_snapshot(&snapshot, 2024, 1);

        assert!(!tb.category_summaries.is_empty());
    }

    #[test]
    fn test_consolidated_trial_balance() {
        let generator = TrialBalanceGenerator::with_defaults();

        let snapshot1 = create_test_snapshot();
        let mut snapshot2_balances = snapshot1.balances.clone();
        for balance in snapshot2_balances.values_mut() {
            balance.closing_balance *= dec!(2);
        }
        let snapshot2 = BalanceSnapshot::new(
            snapshot1.as_of_date,
            "TEST2".to_string(),
            snapshot2_balances,
        );

        let tb1 = generator.generate_from_snapshot(&snapshot1, 2024, 1);
        let tb2 = generator.generate_from_snapshot(&snapshot2, 2024, 1);

        let consolidated = generator.generate_consolidated(&[tb1, tb2], "CONSOL");

        assert_eq!(consolidated.company_code, "CONSOL");
        assert!(consolidated.is_balanced);
    }

    #[test]
    fn test_builder_pattern() {
        let snapshot = create_test_snapshot();

        let trial_balances = TrialBalanceBuilder::new(2024, 1)
            .add_snapshot("TEST", snapshot)
            .build();

        assert_eq!(trial_balances.len(), 1);
        assert!(trial_balances[0].is_balanced);
    }
}
