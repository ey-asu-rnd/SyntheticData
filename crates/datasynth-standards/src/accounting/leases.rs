//! Lease Accounting Models (ASC 842 / IFRS 16).
//!
//! Implements the lease accounting standards for both lessees and lessors:
//!
//! - Lease identification and classification
//! - Right-of-use asset and lease liability measurement
//! - Amortization schedules
//! - Framework-specific classification rules (bright-line vs principles-based)
//!
//! Key differences between frameworks:
//! - US GAAP (ASC 842): Maintains finance vs operating lease distinction for lessees
//! - IFRS 16: Single on-balance-sheet model for lessees (except short-term and low-value)

use chrono::NaiveDate;
use rust_decimal::prelude::*;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::framework::AccountingFramework;

/// Lease contract model.
///
/// Represents a lease arrangement with all data needed for proper
/// accounting treatment under ASC 842 or IFRS 16.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Lease {
    /// Unique lease identifier.
    pub lease_id: Uuid,

    /// Company code (lessee).
    pub company_code: String,

    /// Lessor name.
    pub lessor_name: String,

    /// Lease description.
    pub description: String,

    /// Asset class being leased.
    pub asset_class: LeaseAssetClass,

    /// Lease classification (determined based on framework rules).
    pub classification: LeaseClassification,

    /// Commencement date (when asset is available for use).
    pub commencement_date: NaiveDate,

    /// Lease term in months.
    pub lease_term_months: u32,

    /// Non-cancellable term in months.
    pub noncancelable_term_months: u32,

    /// Optional renewal periods in months.
    pub renewal_option_months: Option<u32>,

    /// Termination option periods in months.
    pub termination_option_months: Option<u32>,

    /// Fixed lease payment per period.
    #[serde(with = "rust_decimal::serde::str")]
    pub fixed_payment: Decimal,

    /// Payment frequency.
    pub payment_frequency: PaymentFrequency,

    /// Variable payment components.
    pub variable_payments: Vec<VariableLeasePayment>,

    /// Discount rate used for present value calculations.
    /// Either implicit rate (if determinable) or incremental borrowing rate.
    #[serde(with = "rust_decimal::serde::str")]
    pub discount_rate: Decimal,

    /// Whether the implicit rate is readily determinable.
    pub implicit_rate_determinable: bool,

    /// Fair value of underlying asset at commencement.
    #[serde(with = "rust_decimal::serde::str")]
    pub fair_value_at_commencement: Decimal,

    /// Economic life of underlying asset in months.
    pub economic_life_months: u32,

    /// Right-of-use asset details.
    pub rou_asset: ROUAsset,

    /// Lease liability details.
    pub lease_liability: LeaseLiability,

    /// Accounting framework applied.
    pub framework: AccountingFramework,

    /// Whether this is a short-term lease election.
    pub short_term_election: bool,

    /// Whether this is a low-value asset lease election (IFRS 16 only).
    pub low_value_election: bool,

    /// Related fixed asset ID (if any).
    pub fixed_asset_id: Option<Uuid>,

    /// Reference to journal entries.
    #[serde(default)]
    pub journal_entry_ids: Vec<Uuid>,
}

impl Lease {
    /// Create a new lease and automatically determine classification.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        company_code: impl Into<String>,
        lessor_name: impl Into<String>,
        description: impl Into<String>,
        asset_class: LeaseAssetClass,
        commencement_date: NaiveDate,
        lease_term_months: u32,
        fixed_payment: Decimal,
        payment_frequency: PaymentFrequency,
        discount_rate: Decimal,
        fair_value_at_commencement: Decimal,
        economic_life_months: u32,
        framework: AccountingFramework,
    ) -> Self {
        let lease_id = Uuid::now_v7();

        // Create initial ROUAsset and LeaseLiability (will be calculated properly)
        let rou_asset = ROUAsset {
            lease_id,
            initial_measurement: Decimal::ZERO,
            accumulated_depreciation: Decimal::ZERO,
            accumulated_impairment: Decimal::ZERO,
            carrying_amount: Decimal::ZERO,
            useful_life_months: lease_term_months,
            depreciation_method: DepreciationMethod::StraightLine,
        };

        let lease_liability = LeaseLiability {
            lease_id,
            initial_measurement: Decimal::ZERO,
            current_portion: Decimal::ZERO,
            non_current_portion: Decimal::ZERO,
            accumulated_interest: Decimal::ZERO,
            amortization_schedule: Vec::new(),
        };

        let mut lease = Self {
            lease_id,
            company_code: company_code.into(),
            lessor_name: lessor_name.into(),
            description: description.into(),
            asset_class,
            classification: LeaseClassification::Operating, // Will be updated
            commencement_date,
            lease_term_months,
            noncancelable_term_months: lease_term_months,
            renewal_option_months: None,
            termination_option_months: None,
            fixed_payment,
            payment_frequency,
            variable_payments: Vec::new(),
            discount_rate,
            implicit_rate_determinable: false,
            fair_value_at_commencement,
            economic_life_months,
            rou_asset,
            lease_liability,
            framework,
            short_term_election: false,
            low_value_election: false,
            fixed_asset_id: None,
            journal_entry_ids: Vec::new(),
        };

        // Determine classification and calculate measurements
        lease.classify();
        lease.calculate_initial_measurement();

        lease
    }

    /// Classify the lease based on the applicable framework.
    pub fn classify(&mut self) {
        // Short-term and low-value elections result in operating treatment
        if self.short_term_election || self.low_value_election {
            self.classification = LeaseClassification::Operating;
            return;
        }

        match self.framework {
            AccountingFramework::UsGaap => self.classify_us_gaap(),
            AccountingFramework::Ifrs => self.classify_ifrs(),
            AccountingFramework::DualReporting => {
                // For dual reporting, use US GAAP classification but note IFRS treatment
                self.classify_us_gaap();
            }
        }
    }

    /// US GAAP classification using bright-line tests (ASC 842).
    fn classify_us_gaap(&mut self) {
        // Test 1: Transfer of ownership at end of lease term
        // (Would need additional data to determine this)

        // Test 2: Bargain purchase option
        // (Would need additional data to determine this)

        // Test 3: Lease term >= 75% of economic life
        let term_ratio =
            Decimal::from(self.lease_term_months) / Decimal::from(self.economic_life_months);
        if term_ratio >= Decimal::from_str_exact("0.75").unwrap() {
            self.classification = LeaseClassification::Finance;
            return;
        }

        // Test 4: Present value of lease payments >= 90% of fair value
        let pv = self.calculate_present_value_of_payments();
        let pv_ratio = pv / self.fair_value_at_commencement;
        if pv_ratio >= Decimal::from_str_exact("0.90").unwrap() {
            self.classification = LeaseClassification::Finance;
            return;
        }

        // Test 5: Specialized nature (no alternative use)
        // (Would need additional data to determine this)

        self.classification = LeaseClassification::Operating;
    }

    /// IFRS classification using principles-based approach (IFRS 16).
    fn classify_ifrs(&mut self) {
        // Under IFRS 16, lessees recognize most leases on balance sheet
        // The finance vs operating distinction is less relevant for lessees
        // but maintained for lessors

        // For simplicity, use similar logic to US GAAP but with more judgment
        // In practice, IFRS looks at transfer of substantially all risks and rewards

        let term_ratio =
            Decimal::from(self.lease_term_months) / Decimal::from(self.economic_life_months);
        let pv = self.calculate_present_value_of_payments();
        let pv_ratio = if self.fair_value_at_commencement > Decimal::ZERO {
            pv / self.fair_value_at_commencement
        } else {
            Decimal::ZERO
        };

        // Major part of economic life or substantially all fair value
        if term_ratio >= Decimal::from_str_exact("0.75").unwrap()
            || pv_ratio >= Decimal::from_str_exact("0.90").unwrap()
        {
            self.classification = LeaseClassification::Finance;
        } else {
            self.classification = LeaseClassification::Operating;
        }
    }

    /// Calculate present value of lease payments.
    pub fn calculate_present_value_of_payments(&self) -> Decimal {
        let mut pv = Decimal::ZERO;
        let periods = self.total_payment_periods();
        let periodic_rate = self.periodic_discount_rate();

        for period in 1..=periods {
            let discount_factor =
                Decimal::ONE / (Decimal::ONE + periodic_rate).powd(Decimal::from(period as i64));
            pv += self.fixed_payment * discount_factor;
        }

        pv
    }

    /// Calculate initial measurement of ROU asset and lease liability.
    pub fn calculate_initial_measurement(&mut self) {
        let pv = self.calculate_present_value_of_payments();

        // Lease liability = PV of lease payments
        self.lease_liability.initial_measurement = pv;

        // ROU Asset = Lease liability + initial direct costs + prepayments - incentives
        // For simplicity, assume ROU asset = lease liability
        self.rou_asset.initial_measurement = pv;
        self.rou_asset.carrying_amount = pv;

        // Split liability into current and non-current
        self.update_liability_classification();

        // Generate amortization schedule
        self.generate_amortization_schedule();
    }

    /// Update current vs non-current liability split.
    fn update_liability_classification(&mut self) {
        let payments_per_year = self.payments_per_year();
        let annual_payments = self.fixed_payment * Decimal::from(payments_per_year);

        // Simplified: current portion = payments due within 12 months
        // (less interest portion, but we'll use a simplified approach)
        let periodic_rate = self.periodic_discount_rate();
        let total_liability = self.lease_liability.initial_measurement;

        // First year's principal payments (simplified)
        let first_year_interest =
            total_liability * periodic_rate * Decimal::from(payments_per_year);
        let first_year_principal = (annual_payments - first_year_interest).max(Decimal::ZERO);

        self.lease_liability.current_portion = first_year_principal.min(total_liability);
        self.lease_liability.non_current_portion =
            total_liability - self.lease_liability.current_portion;
    }

    /// Generate amortization schedule.
    fn generate_amortization_schedule(&mut self) {
        let periods = self.total_payment_periods();
        let periodic_rate = self.periodic_discount_rate();
        let mut balance = self.lease_liability.initial_measurement;
        let mut cumulative_interest = Decimal::ZERO;

        self.lease_liability.amortization_schedule.clear();

        for period in 1..=periods {
            let interest_expense = balance * periodic_rate;
            let principal_payment = self.fixed_payment - interest_expense;
            let new_balance = (balance - principal_payment).max(Decimal::ZERO);
            cumulative_interest += interest_expense;

            let entry = LeaseAmortizationEntry {
                period_number: period,
                period_date: self.period_date(period),
                beginning_balance: balance,
                payment_amount: self.fixed_payment,
                interest_expense,
                principal_payment,
                ending_balance: new_balance,
            };

            self.lease_liability.amortization_schedule.push(entry);
            balance = new_balance;
        }

        self.lease_liability.accumulated_interest = cumulative_interest;
    }

    /// Get the number of payments per year.
    fn payments_per_year(&self) -> u32 {
        match self.payment_frequency {
            PaymentFrequency::Monthly => 12,
            PaymentFrequency::Quarterly => 4,
            PaymentFrequency::SemiAnnual => 2,
            PaymentFrequency::Annual => 1,
        }
    }

    /// Get total number of payment periods.
    fn total_payment_periods(&self) -> u32 {
        let periods_per_month = match self.payment_frequency {
            PaymentFrequency::Monthly => 1,
            PaymentFrequency::Quarterly => 3,
            PaymentFrequency::SemiAnnual => 6,
            PaymentFrequency::Annual => 12,
        };
        self.lease_term_months / periods_per_month
    }

    /// Get periodic discount rate.
    fn periodic_discount_rate(&self) -> Decimal {
        self.discount_rate / Decimal::from(self.payments_per_year())
    }

    /// Calculate period date for a given period number.
    fn period_date(&self, period: u32) -> NaiveDate {
        let months_offset = match self.payment_frequency {
            PaymentFrequency::Monthly => period,
            PaymentFrequency::Quarterly => period * 3,
            PaymentFrequency::SemiAnnual => period * 6,
            PaymentFrequency::Annual => period * 12,
        };

        self.commencement_date
            .checked_add_months(chrono::Months::new(months_offset))
            .unwrap_or(self.commencement_date)
    }
}

/// Lease asset class.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum LeaseAssetClass {
    /// Real estate (buildings, land).
    #[default]
    RealEstate,
    /// Equipment and machinery.
    Equipment,
    /// Vehicles and transportation.
    Vehicles,
    /// Information technology assets.
    InformationTechnology,
    /// Furniture and fixtures.
    FurnitureAndFixtures,
    /// Other assets.
    Other,
}

impl std::fmt::Display for LeaseAssetClass {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::RealEstate => write!(f, "Real Estate"),
            Self::Equipment => write!(f, "Equipment"),
            Self::Vehicles => write!(f, "Vehicles"),
            Self::InformationTechnology => write!(f, "Information Technology"),
            Self::FurnitureAndFixtures => write!(f, "Furniture and Fixtures"),
            Self::Other => write!(f, "Other"),
        }
    }
}

/// Lease classification.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum LeaseClassification {
    /// Finance lease (ASC 842) / Finance lease (IFRS 16).
    /// Transfers substantially all risks and rewards of ownership.
    Finance,
    /// Operating lease.
    /// Does not transfer substantially all risks and rewards.
    #[default]
    Operating,
    /// Short-term lease (term <= 12 months).
    /// May elect simplified treatment.
    ShortTerm,
    /// Low-value asset lease (IFRS 16 only).
    /// May elect simplified treatment.
    LowValue,
}

impl std::fmt::Display for LeaseClassification {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Finance => write!(f, "Finance Lease"),
            Self::Operating => write!(f, "Operating Lease"),
            Self::ShortTerm => write!(f, "Short-Term Lease"),
            Self::LowValue => write!(f, "Low-Value Asset Lease"),
        }
    }
}

/// Payment frequency for lease payments.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum PaymentFrequency {
    /// Monthly payments.
    #[default]
    Monthly,
    /// Quarterly payments.
    Quarterly,
    /// Semi-annual payments.
    SemiAnnual,
    /// Annual payments.
    Annual,
}

/// Variable lease payment component.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VariableLeasePayment {
    /// Type of variable payment.
    pub payment_type: VariablePaymentType,

    /// Description of the variable component.
    pub description: String,

    /// Basis for calculation (e.g., index, rate).
    pub calculation_basis: String,

    /// Estimated annual amount.
    #[serde(with = "rust_decimal::serde::str")]
    pub estimated_annual_amount: Decimal,
}

/// Type of variable lease payment.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VariablePaymentType {
    /// Based on index (e.g., CPI).
    IndexBased,
    /// Based on rate (e.g., LIBOR).
    RateBased,
    /// Based on usage or performance.
    UsageBased,
    /// Common area maintenance or other operating costs.
    OperatingCosts,
    /// Property taxes.
    PropertyTaxes,
    /// Insurance.
    Insurance,
}

/// Right-of-use asset.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ROUAsset {
    /// Parent lease ID.
    pub lease_id: Uuid,

    /// Initial measurement at commencement.
    #[serde(with = "rust_decimal::serde::str")]
    pub initial_measurement: Decimal,

    /// Accumulated depreciation/amortization.
    #[serde(with = "rust_decimal::serde::str")]
    pub accumulated_depreciation: Decimal,

    /// Accumulated impairment losses.
    #[serde(with = "rust_decimal::serde::str")]
    pub accumulated_impairment: Decimal,

    /// Current carrying amount.
    #[serde(with = "rust_decimal::serde::str")]
    pub carrying_amount: Decimal,

    /// Useful life (typically lease term) in months.
    pub useful_life_months: u32,

    /// Depreciation method.
    pub depreciation_method: DepreciationMethod,
}

impl ROUAsset {
    /// Calculate monthly depreciation.
    pub fn monthly_depreciation(&self) -> Decimal {
        if self.useful_life_months == 0 {
            return Decimal::ZERO;
        }
        self.initial_measurement / Decimal::from(self.useful_life_months)
    }

    /// Record depreciation for a period.
    pub fn record_depreciation(&mut self, months: u32) {
        let depreciation = self.monthly_depreciation() * Decimal::from(months);
        self.accumulated_depreciation += depreciation;
        self.carrying_amount =
            self.initial_measurement - self.accumulated_depreciation - self.accumulated_impairment;
        self.carrying_amount = self.carrying_amount.max(Decimal::ZERO);
    }
}

/// Depreciation method for ROU assets.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum DepreciationMethod {
    /// Straight-line over lease term.
    #[default]
    StraightLine,
    /// Based on economic benefits (rare for leases).
    UnitsOfProduction,
}

/// Lease liability.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LeaseLiability {
    /// Parent lease ID.
    pub lease_id: Uuid,

    /// Initial measurement (PV of lease payments).
    #[serde(with = "rust_decimal::serde::str")]
    pub initial_measurement: Decimal,

    /// Current portion (due within 12 months).
    #[serde(with = "rust_decimal::serde::str")]
    pub current_portion: Decimal,

    /// Non-current portion (due after 12 months).
    #[serde(with = "rust_decimal::serde::str")]
    pub non_current_portion: Decimal,

    /// Total interest recognized.
    #[serde(with = "rust_decimal::serde::str")]
    pub accumulated_interest: Decimal,

    /// Amortization schedule.
    pub amortization_schedule: Vec<LeaseAmortizationEntry>,
}

impl LeaseLiability {
    /// Get current total balance.
    pub fn total_balance(&self) -> Decimal {
        self.current_portion + self.non_current_portion
    }
}

/// Lease amortization schedule entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LeaseAmortizationEntry {
    /// Period number (1-based).
    pub period_number: u32,

    /// Date of period end.
    pub period_date: NaiveDate,

    /// Beginning liability balance.
    #[serde(with = "rust_decimal::serde::str")]
    pub beginning_balance: Decimal,

    /// Total payment amount.
    #[serde(with = "rust_decimal::serde::str")]
    pub payment_amount: Decimal,

    /// Interest expense portion.
    #[serde(with = "rust_decimal::serde::str")]
    pub interest_expense: Decimal,

    /// Principal reduction portion.
    #[serde(with = "rust_decimal::serde::str")]
    pub principal_payment: Decimal,

    /// Ending liability balance.
    #[serde(with = "rust_decimal::serde::str")]
    pub ending_balance: Decimal,
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn test_lease_creation() {
        let lease = Lease::new(
            "1000",
            "ABC Leasing",
            "Office Space Lease",
            LeaseAssetClass::RealEstate,
            NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            60, // 5 years
            dec!(10000),
            PaymentFrequency::Monthly,
            dec!(0.05),   // 5% annual rate
            dec!(500000), // Fair value
            120,          // 10 year economic life
            AccountingFramework::UsGaap,
        );

        assert_eq!(lease.lease_term_months, 60);
        // 60/120 = 50% < 75%, so likely operating unless PV test passes
        assert!(lease.lease_liability.initial_measurement > Decimal::ZERO);
    }

    #[test]
    fn test_finance_lease_classification() {
        // Lease term is 90% of economic life - should be finance lease
        let lease = Lease::new(
            "1000",
            "ABC Leasing",
            "Equipment Lease",
            LeaseAssetClass::Equipment,
            NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            108, // 9 years
            dec!(5000),
            PaymentFrequency::Monthly,
            dec!(0.05),
            dec!(400000),
            120, // 10 year economic life (108/120 = 90% >= 75%)
            AccountingFramework::UsGaap,
        );

        assert_eq!(lease.classification, LeaseClassification::Finance);
    }

    #[test]
    fn test_amortization_schedule() {
        let lease = Lease::new(
            "1000",
            "ABC Leasing",
            "Vehicle Lease",
            LeaseAssetClass::Vehicles,
            NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            36, // 3 years
            dec!(500),
            PaymentFrequency::Monthly,
            dec!(0.06),
            dec!(15000),
            60, // 5 year economic life
            AccountingFramework::UsGaap,
        );

        assert_eq!(lease.lease_liability.amortization_schedule.len(), 36);

        // First payment should have higher interest than last
        let first_entry = &lease.lease_liability.amortization_schedule[0];
        let last_entry = &lease.lease_liability.amortization_schedule[35];
        assert!(first_entry.interest_expense > last_entry.interest_expense);

        // Last balance should be zero (or very close)
        assert!(last_entry.ending_balance < dec!(1));
    }

    #[test]
    fn test_rou_asset_depreciation() {
        let lease = Lease::new(
            "1000",
            "Lessor Co",
            "Office Equipment",
            LeaseAssetClass::Equipment,
            NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            24,
            dec!(1000),
            PaymentFrequency::Monthly,
            dec!(0.05),
            dec!(20000),
            60,
            AccountingFramework::UsGaap,
        );

        let mut rou_asset = lease.rou_asset.clone();
        let initial = rou_asset.carrying_amount;
        let monthly_dep = rou_asset.monthly_depreciation();

        rou_asset.record_depreciation(6);

        assert_eq!(rou_asset.accumulated_depreciation, monthly_dep * dec!(6));
        assert!(rou_asset.carrying_amount < initial);
    }
}
