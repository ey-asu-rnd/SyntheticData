//! Period close models.
//!
//! This module provides models for fiscal period management and
//! period-end close processes including:
//! - Fiscal period definitions
//! - Close tasks and workflows
//! - Accrual definitions and schedules
//! - Year-end closing entries

use chrono::{Datelike, NaiveDate};
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use std::collections::HashMap;

/// Fiscal period representation.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct FiscalPeriod {
    /// Fiscal year.
    pub year: i32,
    /// Period number (1-12 for monthly, 1-4 for quarterly).
    pub period: u8,
    /// Period start date.
    pub start_date: NaiveDate,
    /// Period end date.
    pub end_date: NaiveDate,
    /// Period type.
    pub period_type: FiscalPeriodType,
    /// Is this the year-end period?
    pub is_year_end: bool,
    /// Period status.
    pub status: PeriodStatus,
}

impl FiscalPeriod {
    /// Creates a monthly fiscal period.
    pub fn monthly(year: i32, month: u8) -> Self {
        let start_date = NaiveDate::from_ymd_opt(year, month as u32, 1).unwrap();
        let end_date = if month == 12 {
            NaiveDate::from_ymd_opt(year + 1, 1, 1)
                .unwrap()
                .pred_opt()
                .unwrap()
        } else {
            NaiveDate::from_ymd_opt(year, month as u32 + 1, 1)
                .unwrap()
                .pred_opt()
                .unwrap()
        };

        Self {
            year,
            period: month,
            start_date,
            end_date,
            period_type: FiscalPeriodType::Monthly,
            is_year_end: month == 12,
            status: PeriodStatus::Open,
        }
    }

    /// Creates a quarterly fiscal period.
    pub fn quarterly(year: i32, quarter: u8) -> Self {
        let start_month = (quarter - 1) * 3 + 1;
        let end_month = quarter * 3;

        let start_date = NaiveDate::from_ymd_opt(year, start_month as u32, 1).unwrap();
        let end_date = if end_month == 12 {
            NaiveDate::from_ymd_opt(year + 1, 1, 1)
                .unwrap()
                .pred_opt()
                .unwrap()
        } else {
            NaiveDate::from_ymd_opt(year, end_month as u32 + 1, 1)
                .unwrap()
                .pred_opt()
                .unwrap()
        };

        Self {
            year,
            period: quarter,
            start_date,
            end_date,
            period_type: FiscalPeriodType::Quarterly,
            is_year_end: quarter == 4,
            status: PeriodStatus::Open,
        }
    }

    /// Returns the number of days in the period.
    pub fn days(&self) -> i64 {
        (self.end_date - self.start_date).num_days() + 1
    }

    /// Returns the period key (e.g., "2024-01" for monthly).
    pub fn key(&self) -> String {
        format!("{}-{:02}", self.year, self.period)
    }

    /// Checks if a date falls within this period.
    pub fn contains(&self, date: NaiveDate) -> bool {
        date >= self.start_date && date <= self.end_date
    }
}

/// Type of fiscal period.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FiscalPeriodType {
    /// Monthly period.
    Monthly,
    /// Quarterly period.
    Quarterly,
    /// Special period (13th period, adjustments).
    Special,
}

/// Status of a fiscal period.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PeriodStatus {
    /// Period is open for posting.
    Open,
    /// Soft close - limited posting allowed.
    SoftClosed,
    /// Hard close - no posting allowed.
    Closed,
    /// Period is locked for audit.
    Locked,
}

/// Close task types for period-end processing.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum CloseTask {
    /// Run depreciation for fixed assets.
    RunDepreciation,
    /// Post inventory revaluation adjustments.
    PostInventoryRevaluation,
    /// Reconcile AR subledger to GL.
    ReconcileArToGl,
    /// Reconcile AP subledger to GL.
    ReconcileApToGl,
    /// Reconcile FA subledger to GL.
    ReconcileFaToGl,
    /// Reconcile Inventory to GL.
    ReconcileInventoryToGl,
    /// Post accrued expenses.
    PostAccruedExpenses,
    /// Post accrued revenue.
    PostAccruedRevenue,
    /// Post prepaid expense amortization.
    PostPrepaidAmortization,
    /// Allocate corporate overhead.
    AllocateCorporateOverhead,
    /// Post intercompany settlements.
    PostIntercompanySettlements,
    /// Revalue foreign currency balances.
    RevalueForeignCurrency,
    /// Calculate and post tax provision.
    CalculateTaxProvision,
    /// Translate foreign subsidiary trial balances.
    TranslateForeignSubsidiaries,
    /// Eliminate intercompany balances.
    EliminateIntercompany,
    /// Generate trial balance.
    GenerateTrialBalance,
    /// Generate financial statements.
    GenerateFinancialStatements,
    /// Close income statement accounts (year-end).
    CloseIncomeStatement,
    /// Post retained earnings rollforward (year-end).
    PostRetainedEarningsRollforward,
    /// Custom task.
    Custom(String),
}

impl CloseTask {
    /// Returns true if this is a year-end only task.
    pub fn is_year_end_only(&self) -> bool {
        matches!(
            self,
            CloseTask::CloseIncomeStatement | CloseTask::PostRetainedEarningsRollforward
        )
    }

    /// Returns the task name.
    pub fn name(&self) -> &str {
        match self {
            CloseTask::RunDepreciation => "Run Depreciation",
            CloseTask::PostInventoryRevaluation => "Post Inventory Revaluation",
            CloseTask::ReconcileArToGl => "Reconcile AR to GL",
            CloseTask::ReconcileApToGl => "Reconcile AP to GL",
            CloseTask::ReconcileFaToGl => "Reconcile FA to GL",
            CloseTask::ReconcileInventoryToGl => "Reconcile Inventory to GL",
            CloseTask::PostAccruedExpenses => "Post Accrued Expenses",
            CloseTask::PostAccruedRevenue => "Post Accrued Revenue",
            CloseTask::PostPrepaidAmortization => "Post Prepaid Amortization",
            CloseTask::AllocateCorporateOverhead => "Allocate Corporate Overhead",
            CloseTask::PostIntercompanySettlements => "Post IC Settlements",
            CloseTask::RevalueForeignCurrency => "Revalue Foreign Currency",
            CloseTask::CalculateTaxProvision => "Calculate Tax Provision",
            CloseTask::TranslateForeignSubsidiaries => "Translate Foreign Subs",
            CloseTask::EliminateIntercompany => "Eliminate Intercompany",
            CloseTask::GenerateTrialBalance => "Generate Trial Balance",
            CloseTask::GenerateFinancialStatements => "Generate Financials",
            CloseTask::CloseIncomeStatement => "Close Income Statement",
            CloseTask::PostRetainedEarningsRollforward => "Post RE Rollforward",
            CloseTask::Custom(name) => name,
        }
    }
}

/// Status of a close task execution.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CloseTaskStatus {
    /// Not started.
    Pending,
    /// In progress.
    InProgress,
    /// Completed successfully.
    Completed,
    /// Completed with warnings.
    CompletedWithWarnings(Vec<String>),
    /// Failed.
    Failed(String),
    /// Skipped.
    Skipped(String),
}

/// Result of executing a close task.
#[derive(Debug, Clone)]
pub struct CloseTaskResult {
    /// Task that was executed.
    pub task: CloseTask,
    /// Company code.
    pub company_code: String,
    /// Fiscal period.
    pub fiscal_period: FiscalPeriod,
    /// Status.
    pub status: CloseTaskStatus,
    /// Start time.
    pub started_at: Option<NaiveDate>,
    /// End time.
    pub completed_at: Option<NaiveDate>,
    /// Journal entries created.
    pub journal_entries_created: u32,
    /// Total amount posted.
    pub total_amount: Decimal,
    /// Execution notes.
    pub notes: Vec<String>,
}

impl CloseTaskResult {
    /// Creates a new task result.
    pub fn new(task: CloseTask, company_code: String, fiscal_period: FiscalPeriod) -> Self {
        Self {
            task,
            company_code,
            fiscal_period,
            status: CloseTaskStatus::Pending,
            started_at: None,
            completed_at: None,
            journal_entries_created: 0,
            total_amount: Decimal::ZERO,
            notes: Vec::new(),
        }
    }

    /// Returns true if the task completed successfully.
    pub fn is_success(&self) -> bool {
        matches!(
            self.status,
            CloseTaskStatus::Completed | CloseTaskStatus::CompletedWithWarnings(_)
        )
    }
}

/// Accrual definition for recurring period-end entries.
#[derive(Debug, Clone)]
pub struct AccrualDefinition {
    /// Accrual ID.
    pub accrual_id: String,
    /// Company code.
    pub company_code: String,
    /// Description.
    pub description: String,
    /// Accrual type.
    pub accrual_type: AccrualType,
    /// Expense/Revenue account to debit/credit.
    pub expense_revenue_account: String,
    /// Accrual liability/asset account.
    pub accrual_account: String,
    /// Calculation method.
    pub calculation_method: AccrualCalculationMethod,
    /// Fixed amount (if applicable).
    pub fixed_amount: Option<Decimal>,
    /// Percentage rate (if applicable).
    pub percentage_rate: Option<Decimal>,
    /// Base account for percentage calculation.
    pub base_account: Option<String>,
    /// Frequency.
    pub frequency: AccrualFrequency,
    /// Auto-reverse on first day of next period.
    pub auto_reverse: bool,
    /// Cost center.
    pub cost_center: Option<String>,
    /// Active flag.
    pub is_active: bool,
    /// Start date.
    pub effective_from: NaiveDate,
    /// End date (if defined).
    pub effective_to: Option<NaiveDate>,
}

impl AccrualDefinition {
    /// Creates a new accrual definition.
    pub fn new(
        accrual_id: String,
        company_code: String,
        description: String,
        accrual_type: AccrualType,
        expense_revenue_account: String,
        accrual_account: String,
    ) -> Self {
        Self {
            accrual_id,
            company_code,
            description,
            accrual_type,
            expense_revenue_account,
            accrual_account,
            calculation_method: AccrualCalculationMethod::FixedAmount,
            fixed_amount: None,
            percentage_rate: None,
            base_account: None,
            frequency: AccrualFrequency::Monthly,
            auto_reverse: true,
            cost_center: None,
            is_active: true,
            effective_from: NaiveDate::from_ymd_opt(2020, 1, 1).unwrap(),
            effective_to: None,
        }
    }

    /// Sets the fixed amount.
    pub fn with_fixed_amount(mut self, amount: Decimal) -> Self {
        self.calculation_method = AccrualCalculationMethod::FixedAmount;
        self.fixed_amount = Some(amount);
        self
    }

    /// Sets percentage-based calculation.
    pub fn with_percentage(mut self, rate: Decimal, base_account: &str) -> Self {
        self.calculation_method = AccrualCalculationMethod::PercentageOfBase;
        self.percentage_rate = Some(rate);
        self.base_account = Some(base_account.to_string());
        self
    }

    /// Checks if the accrual is effective for a given date.
    pub fn is_effective_on(&self, date: NaiveDate) -> bool {
        if !self.is_active {
            return false;
        }
        if date < self.effective_from {
            return false;
        }
        if let Some(end) = self.effective_to {
            if date > end {
                return false;
            }
        }
        true
    }
}

/// Type of accrual.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AccrualType {
    /// Accrued expense (debit expense, credit liability).
    AccruedExpense,
    /// Accrued revenue (debit asset, credit revenue).
    AccruedRevenue,
    /// Prepaid expense (debit expense, credit asset).
    PrepaidExpense,
    /// Deferred revenue (debit liability, credit revenue).
    DeferredRevenue,
}

/// Calculation method for accruals.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AccrualCalculationMethod {
    /// Fixed amount each period.
    FixedAmount,
    /// Percentage of a base account balance.
    PercentageOfBase,
    /// Days-based proration.
    DaysBased,
    /// Calculated externally (manual entry).
    Manual,
}

/// Frequency for accrual posting.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AccrualFrequency {
    /// Every month.
    Monthly,
    /// Every quarter.
    Quarterly,
    /// Every year.
    Annually,
}

/// Corporate overhead allocation definition.
#[derive(Debug, Clone)]
pub struct OverheadAllocation {
    /// Allocation ID.
    pub allocation_id: String,
    /// Source company code (corporate).
    pub source_company: String,
    /// Source cost center.
    pub source_cost_center: String,
    /// Source account.
    pub source_account: String,
    /// Allocation basis.
    pub allocation_basis: AllocationBasis,
    /// Target allocations.
    pub targets: Vec<AllocationTarget>,
    /// Description.
    pub description: String,
    /// Active flag.
    pub is_active: bool,
}

/// Basis for overhead allocation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AllocationBasis {
    /// Based on revenue.
    Revenue,
    /// Based on headcount.
    Headcount,
    /// Based on direct costs.
    DirectCosts,
    /// Based on square footage.
    SquareFootage,
    /// Fixed percentages.
    FixedPercentage,
    /// Custom formula.
    Custom(String),
}

/// Target for overhead allocation.
#[derive(Debug, Clone)]
pub struct AllocationTarget {
    /// Target company code.
    pub company_code: String,
    /// Target cost center.
    pub cost_center: String,
    /// Target account.
    pub account: String,
    /// Allocation percentage (for fixed percentage basis).
    pub percentage: Option<Decimal>,
    /// Allocation driver value (for calculated basis).
    pub driver_value: Option<Decimal>,
}

/// Period close schedule defining the order of tasks.
#[derive(Debug, Clone)]
pub struct CloseSchedule {
    /// Schedule ID.
    pub schedule_id: String,
    /// Company code (or "ALL" for all companies).
    pub company_code: String,
    /// Period type this schedule applies to.
    pub period_type: FiscalPeriodType,
    /// Ordered list of tasks.
    pub tasks: Vec<ScheduledCloseTask>,
    /// Whether this is for year-end.
    pub is_year_end: bool,
}

impl CloseSchedule {
    /// Creates a standard monthly close schedule.
    pub fn standard_monthly(company_code: &str) -> Self {
        Self {
            schedule_id: format!("MONTHLY-{}", company_code),
            company_code: company_code.to_string(),
            period_type: FiscalPeriodType::Monthly,
            tasks: vec![
                ScheduledCloseTask::new(CloseTask::RunDepreciation, 1),
                ScheduledCloseTask::new(CloseTask::PostInventoryRevaluation, 2),
                ScheduledCloseTask::new(CloseTask::PostAccruedExpenses, 3),
                ScheduledCloseTask::new(CloseTask::PostAccruedRevenue, 4),
                ScheduledCloseTask::new(CloseTask::PostPrepaidAmortization, 5),
                ScheduledCloseTask::new(CloseTask::RevalueForeignCurrency, 6),
                ScheduledCloseTask::new(CloseTask::ReconcileArToGl, 7),
                ScheduledCloseTask::new(CloseTask::ReconcileApToGl, 8),
                ScheduledCloseTask::new(CloseTask::ReconcileFaToGl, 9),
                ScheduledCloseTask::new(CloseTask::ReconcileInventoryToGl, 10),
                ScheduledCloseTask::new(CloseTask::PostIntercompanySettlements, 11),
                ScheduledCloseTask::new(CloseTask::AllocateCorporateOverhead, 12),
                ScheduledCloseTask::new(CloseTask::TranslateForeignSubsidiaries, 13),
                ScheduledCloseTask::new(CloseTask::EliminateIntercompany, 14),
                ScheduledCloseTask::new(CloseTask::GenerateTrialBalance, 15),
            ],
            is_year_end: false,
        }
    }

    /// Creates a year-end close schedule.
    pub fn year_end(company_code: &str) -> Self {
        let mut schedule = Self::standard_monthly(company_code);
        schedule.schedule_id = format!("YEAREND-{}", company_code);
        schedule.is_year_end = true;

        // Add year-end specific tasks
        let next_seq = schedule.tasks.len() as u32 + 1;
        schedule.tasks.push(ScheduledCloseTask::new(
            CloseTask::CalculateTaxProvision,
            next_seq,
        ));
        schedule.tasks.push(ScheduledCloseTask::new(
            CloseTask::CloseIncomeStatement,
            next_seq + 1,
        ));
        schedule.tasks.push(ScheduledCloseTask::new(
            CloseTask::PostRetainedEarningsRollforward,
            next_seq + 2,
        ));
        schedule
            .tasks
            .push(ScheduledCloseTask::new(CloseTask::GenerateFinancialStatements, next_seq + 3));

        schedule
    }
}

/// A scheduled close task with sequence and dependencies.
#[derive(Debug, Clone)]
pub struct ScheduledCloseTask {
    /// The task to execute.
    pub task: CloseTask,
    /// Sequence number (execution order).
    pub sequence: u32,
    /// Tasks that must complete before this one.
    pub depends_on: Vec<CloseTask>,
    /// Is this task mandatory?
    pub is_mandatory: bool,
    /// Can this task run in parallel with others at same sequence?
    pub can_parallelize: bool,
}

impl ScheduledCloseTask {
    /// Creates a new scheduled task.
    pub fn new(task: CloseTask, sequence: u32) -> Self {
        Self {
            task,
            sequence,
            depends_on: Vec::new(),
            is_mandatory: true,
            can_parallelize: false,
        }
    }

    /// Adds a dependency.
    pub fn depends_on(mut self, task: CloseTask) -> Self {
        self.depends_on.push(task);
        self
    }

    /// Marks as optional.
    pub fn optional(mut self) -> Self {
        self.is_mandatory = false;
        self
    }

    /// Allows parallel execution.
    pub fn parallelizable(mut self) -> Self {
        self.can_parallelize = true;
        self
    }
}

/// Year-end closing entry specification.
#[derive(Debug, Clone)]
pub struct YearEndClosingSpec {
    /// Company code.
    pub company_code: String,
    /// Fiscal year being closed.
    pub fiscal_year: i32,
    /// Revenue accounts to close.
    pub revenue_accounts: Vec<String>,
    /// Expense accounts to close.
    pub expense_accounts: Vec<String>,
    /// Income summary account (temporary).
    pub income_summary_account: String,
    /// Retained earnings account.
    pub retained_earnings_account: String,
    /// Dividend account (if applicable).
    pub dividend_account: Option<String>,
}

impl Default for YearEndClosingSpec {
    fn default() -> Self {
        Self {
            company_code: String::new(),
            fiscal_year: 0,
            revenue_accounts: vec!["4".to_string()], // All accounts starting with 4
            expense_accounts: vec!["5".to_string(), "6".to_string()], // Accounts starting with 5, 6
            income_summary_account: "3500".to_string(),
            retained_earnings_account: "3300".to_string(),
            dividend_account: Some("3400".to_string()),
        }
    }
}

/// Tax provision calculation inputs.
#[derive(Debug, Clone)]
pub struct TaxProvisionInput {
    /// Company code.
    pub company_code: String,
    /// Fiscal year.
    pub fiscal_year: i32,
    /// Pre-tax book income.
    pub pretax_income: Decimal,
    /// Permanent differences (add back).
    pub permanent_differences: Vec<TaxAdjustment>,
    /// Temporary differences (timing).
    pub temporary_differences: Vec<TaxAdjustment>,
    /// Statutory tax rate.
    pub statutory_rate: Decimal,
    /// Tax credits available.
    pub tax_credits: Decimal,
    /// Prior year over/under provision.
    pub prior_year_adjustment: Decimal,
}

/// Tax adjustment item.
#[derive(Debug, Clone)]
pub struct TaxAdjustment {
    /// Description.
    pub description: String,
    /// Amount.
    pub amount: Decimal,
    /// Is this a deduction (negative) or addition (positive)?
    pub is_addition: bool,
}

/// Tax provision result.
#[derive(Debug, Clone)]
pub struct TaxProvisionResult {
    /// Company code.
    pub company_code: String,
    /// Fiscal year.
    pub fiscal_year: i32,
    /// Pre-tax book income.
    pub pretax_income: Decimal,
    /// Total permanent differences.
    pub permanent_differences: Decimal,
    /// Taxable income.
    pub taxable_income: Decimal,
    /// Current tax expense.
    pub current_tax_expense: Decimal,
    /// Deferred tax expense (benefit).
    pub deferred_tax_expense: Decimal,
    /// Total tax expense.
    pub total_tax_expense: Decimal,
    /// Effective tax rate.
    pub effective_rate: Decimal,
}

impl TaxProvisionResult {
    /// Calculates the tax provision from inputs.
    pub fn calculate(input: &TaxProvisionInput) -> Self {
        let permanent_diff: Decimal = input
            .permanent_differences
            .iter()
            .map(|d| if d.is_addition { d.amount } else { -d.amount })
            .sum();

        let temporary_diff: Decimal = input
            .temporary_differences
            .iter()
            .map(|d| if d.is_addition { d.amount } else { -d.amount })
            .sum();

        let taxable_income = input.pretax_income + permanent_diff;
        let current_tax = (taxable_income * input.statutory_rate / dec!(100)).round_dp(2);
        let deferred_tax = (temporary_diff * input.statutory_rate / dec!(100)).round_dp(2);

        let total_tax = current_tax + deferred_tax - input.tax_credits + input.prior_year_adjustment;

        let effective_rate = if input.pretax_income != Decimal::ZERO {
            (total_tax / input.pretax_income * dec!(100)).round_dp(2)
        } else {
            Decimal::ZERO
        };

        Self {
            company_code: input.company_code.clone(),
            fiscal_year: input.fiscal_year,
            pretax_income: input.pretax_income,
            permanent_differences: permanent_diff,
            taxable_income,
            current_tax_expense: current_tax,
            deferred_tax_expense: deferred_tax,
            total_tax_expense: total_tax,
            effective_rate,
        }
    }
}

/// Period close run status.
#[derive(Debug, Clone)]
pub struct PeriodCloseRun {
    /// Run ID.
    pub run_id: String,
    /// Company code.
    pub company_code: String,
    /// Fiscal period.
    pub fiscal_period: FiscalPeriod,
    /// Status.
    pub status: PeriodCloseStatus,
    /// Task results.
    pub task_results: Vec<CloseTaskResult>,
    /// Started at.
    pub started_at: Option<NaiveDate>,
    /// Completed at.
    pub completed_at: Option<NaiveDate>,
    /// Total journal entries created.
    pub total_journal_entries: u32,
    /// Errors encountered.
    pub errors: Vec<String>,
}

/// Status of a period close run.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PeriodCloseStatus {
    /// Not started.
    NotStarted,
    /// In progress.
    InProgress,
    /// Completed successfully.
    Completed,
    /// Completed with errors.
    CompletedWithErrors,
    /// Failed.
    Failed,
}

impl PeriodCloseRun {
    /// Creates a new period close run.
    pub fn new(run_id: String, company_code: String, fiscal_period: FiscalPeriod) -> Self {
        Self {
            run_id,
            company_code,
            fiscal_period,
            status: PeriodCloseStatus::NotStarted,
            task_results: Vec::new(),
            started_at: None,
            completed_at: None,
            total_journal_entries: 0,
            errors: Vec::new(),
        }
    }

    /// Returns true if all tasks completed successfully.
    pub fn is_success(&self) -> bool {
        self.status == PeriodCloseStatus::Completed
    }

    /// Returns the number of failed tasks.
    pub fn failed_task_count(&self) -> usize {
        self.task_results
            .iter()
            .filter(|r| matches!(r.status, CloseTaskStatus::Failed(_)))
            .count()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fiscal_period_monthly() {
        let period = FiscalPeriod::monthly(2024, 1);
        assert_eq!(period.start_date, NaiveDate::from_ymd_opt(2024, 1, 1).unwrap());
        assert_eq!(period.end_date, NaiveDate::from_ymd_opt(2024, 1, 31).unwrap());
        assert_eq!(period.days(), 31);
        assert!(!period.is_year_end);

        let dec_period = FiscalPeriod::monthly(2024, 12);
        assert!(dec_period.is_year_end);
    }

    #[test]
    fn test_fiscal_period_quarterly() {
        let q1 = FiscalPeriod::quarterly(2024, 1);
        assert_eq!(q1.start_date, NaiveDate::from_ymd_opt(2024, 1, 1).unwrap());
        assert_eq!(q1.end_date, NaiveDate::from_ymd_opt(2024, 3, 31).unwrap());

        let q4 = FiscalPeriod::quarterly(2024, 4);
        assert!(q4.is_year_end);
    }

    #[test]
    fn test_close_schedule() {
        let schedule = CloseSchedule::standard_monthly("1000");
        assert!(!schedule.is_year_end);
        assert!(!schedule.tasks.is_empty());

        let year_end = CloseSchedule::year_end("1000");
        assert!(year_end.is_year_end);
        assert!(year_end.tasks.len() > schedule.tasks.len());
    }

    #[test]
    fn test_tax_provision() {
        let input = TaxProvisionInput {
            company_code: "1000".to_string(),
            fiscal_year: 2024,
            pretax_income: dec!(1000000),
            permanent_differences: vec![TaxAdjustment {
                description: "Meals & Entertainment".to_string(),
                amount: dec!(10000),
                is_addition: true,
            }],
            temporary_differences: vec![TaxAdjustment {
                description: "Depreciation Timing".to_string(),
                amount: dec!(50000),
                is_addition: false,
            }],
            statutory_rate: dec!(21),
            tax_credits: dec!(5000),
            prior_year_adjustment: Decimal::ZERO,
        };

        let result = TaxProvisionResult::calculate(&input);
        assert_eq!(result.taxable_income, dec!(1010000)); // 1M + 10K permanent
        assert!(result.current_tax_expense > Decimal::ZERO);
    }
}
