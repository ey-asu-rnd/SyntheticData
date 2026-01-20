//! Behavioral persona blueprints module.
//!
//! This module defines behavioral patterns for different customer types:
//! - Retail personas (student, early career, mid-career, etc.)
//! - Business personas (SME, mid-market, enterprise, etc.)
//! - Trust personas (family trust, charitable foundation, etc.)

mod business;
mod retail;
mod trust;

// Re-export specific functions with qualified names
pub use business::get_profile as get_business_profile_impl;
pub use retail::get_profile as get_retail_profile_impl;
pub use trust::get_profile as get_trust_profile_impl;

use synth_core::models::banking::{BusinessPersona, RetailPersona, TrustPersona};

/// Transaction behavior profile.
#[derive(Debug, Clone)]
pub struct TransactionBehavior {
    /// Average transactions per month
    pub monthly_tx_count: u32,
    /// Standard deviation of monthly count
    pub monthly_tx_std: f64,
    /// Average transaction amount
    pub avg_amount: f64,
    /// Amount standard deviation
    pub amount_std: f64,
    /// Minimum transaction amount
    pub min_amount: f64,
    /// Maximum transaction amount
    pub max_amount: f64,
    /// Percentage of transactions that are cash
    pub cash_percentage: f64,
    /// Percentage of transactions that are international
    pub international_percentage: f64,
    /// Typical transaction hours (start, end)
    pub active_hours: (u8, u8),
    /// Weekend activity multiplier
    pub weekend_multiplier: f64,
}

impl Default for TransactionBehavior {
    fn default() -> Self {
        Self {
            monthly_tx_count: 30,
            monthly_tx_std: 10.0,
            avg_amount: 150.0,
            amount_std: 100.0,
            min_amount: 5.0,
            max_amount: 5000.0,
            cash_percentage: 0.1,
            international_percentage: 0.01,
            active_hours: (8, 22),
            weekend_multiplier: 1.0,
        }
    }
}

/// Spending category distribution.
#[derive(Debug, Clone)]
pub struct SpendingProfile {
    /// Groceries percentage
    pub groceries: f64,
    /// Dining/restaurants percentage
    pub dining: f64,
    /// Entertainment percentage
    pub entertainment: f64,
    /// Shopping/retail percentage
    pub shopping: f64,
    /// Transportation percentage
    pub transportation: f64,
    /// Utilities percentage
    pub utilities: f64,
    /// Healthcare percentage
    pub healthcare: f64,
    /// Travel percentage
    pub travel: f64,
    /// Other/misc percentage
    pub other: f64,
}

impl Default for SpendingProfile {
    fn default() -> Self {
        Self {
            groceries: 0.20,
            dining: 0.12,
            entertainment: 0.08,
            shopping: 0.15,
            transportation: 0.10,
            utilities: 0.15,
            healthcare: 0.05,
            travel: 0.05,
            other: 0.10,
        }
    }
}

/// Income profile for retail customers.
#[derive(Debug, Clone)]
pub struct IncomeProfile {
    /// Primary income source
    pub source: IncomeSource,
    /// Monthly income amount
    pub monthly_amount: f64,
    /// Income frequency
    pub frequency: IncomeFrequency,
    /// Day of month for income (if applicable)
    pub income_day: Option<u8>,
    /// Has multiple income streams
    pub has_secondary: bool,
}

/// Income source types.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IncomeSource {
    Salary,
    HourlyWage,
    SelfEmployment,
    Pension,
    SocialSecurity,
    Investment,
    Rental,
    Gig,
    ParentalSupport,
    Other,
}

/// Income frequency.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IncomeFrequency {
    Weekly,
    BiWeekly,
    SemiMonthly,
    Monthly,
    Irregular,
}

/// Full persona profile combining all aspects.
#[derive(Debug, Clone)]
pub struct PersonaProfile {
    /// Transaction behavior
    pub transaction_behavior: TransactionBehavior,
    /// Spending categories
    pub spending_profile: SpendingProfile,
    /// Income profile (for retail)
    pub income_profile: Option<IncomeProfile>,
    /// Risk appetite (0.0 = very conservative, 1.0 = very aggressive)
    pub risk_appetite: f64,
    /// Saving rate (percentage of income saved)
    pub saving_rate: f64,
    /// Likelihood to use credit
    pub credit_usage: f64,
}

/// Get persona profile for a retail persona.
pub fn get_retail_profile(persona: RetailPersona) -> PersonaProfile {
    retail::get_profile(persona)
}

/// Get persona profile for a business persona.
pub fn get_business_profile(persona: BusinessPersona) -> PersonaProfile {
    business::get_profile(persona)
}

/// Get persona profile for a trust persona.
pub fn get_trust_profile(persona: TrustPersona) -> PersonaProfile {
    trust::get_profile(persona)
}
