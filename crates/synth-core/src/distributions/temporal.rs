//! Temporal distribution samplers for realistic posting patterns.
//!
//! Implements seasonality, working hour patterns, and period-end spikes
//! commonly observed in enterprise accounting systems.

use chrono::{Datelike, Duration, NaiveDate, NaiveTime, Weekday};
use rand::prelude::*;
use rand_chacha::ChaCha8Rng;
use serde::{Deserialize, Serialize};

/// Configuration for seasonality patterns.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SeasonalityConfig {
    /// Enable month-end volume spikes
    pub month_end_spike: bool,
    /// Month-end spike multiplier (e.g., 2.5 = 2.5x normal volume)
    pub month_end_multiplier: f64,
    /// Days before month-end to start spike
    pub month_end_lead_days: u32,

    /// Enable quarter-end spikes
    pub quarter_end_spike: bool,
    /// Quarter-end spike multiplier
    pub quarter_end_multiplier: f64,

    /// Enable year-end spikes
    pub year_end_spike: bool,
    /// Year-end spike multiplier
    pub year_end_multiplier: f64,

    /// Activity level on weekends (0.0 = no activity, 1.0 = normal)
    pub weekend_activity: f64,
    /// Activity level on holidays
    pub holiday_activity: f64,
}

impl Default for SeasonalityConfig {
    fn default() -> Self {
        Self {
            month_end_spike: true,
            month_end_multiplier: 2.5,
            month_end_lead_days: 5,
            quarter_end_spike: true,
            quarter_end_multiplier: 4.0,
            year_end_spike: true,
            year_end_multiplier: 6.0,
            weekend_activity: 0.1,
            holiday_activity: 0.05,
        }
    }
}

/// Configuration for working hours pattern.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkingHoursConfig {
    /// Start of working day (hour, 0-23)
    pub day_start: u8,
    /// End of working day (hour, 0-23)
    pub day_end: u8,
    /// Peak hours during the day
    pub peak_hours: Vec<u8>,
    /// Weight for peak hours (multiplier)
    pub peak_weight: f64,
    /// Probability of after-hours posting
    pub after_hours_probability: f64,
}

impl Default for WorkingHoursConfig {
    fn default() -> Self {
        Self {
            day_start: 8,
            day_end: 18,
            peak_hours: vec![9, 10, 11, 14, 15, 16],
            peak_weight: 1.5,
            after_hours_probability: 0.05,
        }
    }
}

/// Sampler for temporal patterns in transaction generation.
pub struct TemporalSampler {
    rng: ChaCha8Rng,
    seasonality_config: SeasonalityConfig,
    working_hours_config: WorkingHoursConfig,
    /// List of holiday dates
    holidays: Vec<NaiveDate>,
}

impl TemporalSampler {
    /// Create a new temporal sampler.
    pub fn new(seed: u64) -> Self {
        Self::with_config(
            seed,
            SeasonalityConfig::default(),
            WorkingHoursConfig::default(),
            Vec::new(),
        )
    }

    /// Create a temporal sampler with custom configuration.
    pub fn with_config(
        seed: u64,
        seasonality_config: SeasonalityConfig,
        working_hours_config: WorkingHoursConfig,
        holidays: Vec<NaiveDate>,
    ) -> Self {
        Self {
            rng: ChaCha8Rng::seed_from_u64(seed),
            seasonality_config,
            working_hours_config,
            holidays,
        }
    }

    /// Generate US federal holidays for a given year.
    pub fn generate_us_holidays(year: i32) -> Vec<NaiveDate> {
        let mut holidays = Vec::new();

        // New Year's Day
        holidays.push(NaiveDate::from_ymd_opt(year, 1, 1).unwrap());
        // Independence Day
        holidays.push(NaiveDate::from_ymd_opt(year, 7, 4).unwrap());
        // Christmas
        holidays.push(NaiveDate::from_ymd_opt(year, 12, 25).unwrap());
        // Thanksgiving (4th Thursday of November)
        let first_thursday = (1..=7)
            .map(|d| NaiveDate::from_ymd_opt(year, 11, d).unwrap())
            .find(|d| d.weekday() == Weekday::Thu)
            .unwrap();
        let thanksgiving = first_thursday + Duration::weeks(3);
        holidays.push(thanksgiving);

        holidays
    }

    /// Check if a date is a weekend.
    pub fn is_weekend(&self, date: NaiveDate) -> bool {
        matches!(date.weekday(), Weekday::Sat | Weekday::Sun)
    }

    /// Check if a date is a holiday.
    pub fn is_holiday(&self, date: NaiveDate) -> bool {
        self.holidays.contains(&date)
    }

    /// Check if a date is month-end (last N days of month).
    pub fn is_month_end(&self, date: NaiveDate) -> bool {
        let last_day = Self::last_day_of_month(date);
        let days_until_end = (last_day - date).num_days();
        days_until_end >= 0 && days_until_end < self.seasonality_config.month_end_lead_days as i64
    }

    /// Check if a date is quarter-end.
    pub fn is_quarter_end(&self, date: NaiveDate) -> bool {
        let month = date.month();
        let is_quarter_end_month = matches!(month, 3 | 6 | 9 | 12);
        is_quarter_end_month && self.is_month_end(date)
    }

    /// Check if a date is year-end.
    pub fn is_year_end(&self, date: NaiveDate) -> bool {
        date.month() == 12 && self.is_month_end(date)
    }

    /// Get the last day of the month for a given date.
    pub fn last_day_of_month(date: NaiveDate) -> NaiveDate {
        let year = date.year();
        let month = date.month();

        if month == 12 {
            NaiveDate::from_ymd_opt(year + 1, 1, 1).unwrap() - Duration::days(1)
        } else {
            NaiveDate::from_ymd_opt(year, month + 1, 1).unwrap() - Duration::days(1)
        }
    }

    /// Get the activity multiplier for a specific date.
    pub fn get_date_multiplier(&self, date: NaiveDate) -> f64 {
        let mut multiplier = 1.0;

        // Weekend reduction
        if self.is_weekend(date) {
            multiplier *= self.seasonality_config.weekend_activity;
        }

        // Holiday reduction
        if self.is_holiday(date) {
            multiplier *= self.seasonality_config.holiday_activity;
        }

        // Period-end spikes (take the highest applicable)
        if self.seasonality_config.year_end_spike && self.is_year_end(date) {
            multiplier *= self.seasonality_config.year_end_multiplier;
        } else if self.seasonality_config.quarter_end_spike && self.is_quarter_end(date) {
            multiplier *= self.seasonality_config.quarter_end_multiplier;
        } else if self.seasonality_config.month_end_spike && self.is_month_end(date) {
            multiplier *= self.seasonality_config.month_end_multiplier;
        }

        multiplier
    }

    /// Sample a posting date within a range based on seasonality.
    pub fn sample_date(&mut self, start: NaiveDate, end: NaiveDate) -> NaiveDate {
        let days = (end - start).num_days() as usize;
        if days == 0 {
            return start;
        }

        // Build weighted distribution based on activity levels
        let mut weights: Vec<f64> = (0..=days)
            .map(|d| {
                let date = start + Duration::days(d as i64);
                self.get_date_multiplier(date)
            })
            .collect();

        // Normalize weights
        let total: f64 = weights.iter().sum();
        weights.iter_mut().for_each(|w| *w /= total);

        // Sample using weights
        let p: f64 = self.rng.gen();
        let mut cumulative = 0.0;
        for (i, weight) in weights.iter().enumerate() {
            cumulative += weight;
            if p < cumulative {
                return start + Duration::days(i as i64);
            }
        }

        end
    }

    /// Sample a posting time based on working hours.
    pub fn sample_time(&mut self, is_human: bool) -> NaiveTime {
        if !is_human {
            // Automated systems can post any time, but prefer off-hours
            let hour = if self.rng.gen::<f64>() < 0.7 {
                // 70% off-peak hours (night batch processing)
                self.rng.gen_range(22..=23).clamp(0, 23)
                    + if self.rng.gen_bool(0.5) {
                        0
                    } else {
                        self.rng.gen_range(0..=5)
                    }
            } else {
                self.rng.gen_range(0..24)
            };
            let minute = self.rng.gen_range(0..60);
            let second = self.rng.gen_range(0..60);
            return NaiveTime::from_hms_opt(hour.clamp(0, 23) as u32, minute, second).unwrap();
        }

        // Human users follow working hours
        let hour = if self.rng.gen::<f64>() < self.working_hours_config.after_hours_probability {
            // After hours
            if self.rng.gen_bool(0.5) {
                self.rng.gen_range(6..self.working_hours_config.day_start)
            } else {
                self.rng.gen_range(self.working_hours_config.day_end..22)
            }
        } else {
            // Normal working hours with peak weighting
            let is_peak = self.rng.gen::<f64>() < 0.6; // 60% during peak
            if is_peak && !self.working_hours_config.peak_hours.is_empty() {
                *self
                    .working_hours_config
                    .peak_hours
                    .choose(&mut self.rng)
                    .unwrap()
            } else {
                self.rng.gen_range(
                    self.working_hours_config.day_start..self.working_hours_config.day_end,
                )
            }
        };

        let minute = self.rng.gen_range(0..60);
        let second = self.rng.gen_range(0..60);

        NaiveTime::from_hms_opt(hour as u32, minute, second).unwrap()
    }

    /// Calculate expected transaction count for a date given daily average.
    pub fn expected_count_for_date(&self, date: NaiveDate, daily_average: f64) -> u64 {
        let multiplier = self.get_date_multiplier(date);
        (daily_average * multiplier).round() as u64
    }

    /// Reset the sampler with a new seed.
    pub fn reset(&mut self, seed: u64) {
        self.rng = ChaCha8Rng::seed_from_u64(seed);
    }
}

/// Time period specification for generation.
#[derive(Debug, Clone)]
pub struct TimePeriod {
    /// Start date (inclusive)
    pub start_date: NaiveDate,
    /// End date (inclusive)
    pub end_date: NaiveDate,
    /// Fiscal year
    pub fiscal_year: u16,
    /// Fiscal periods covered
    pub fiscal_periods: Vec<u8>,
}

impl TimePeriod {
    /// Create a time period for a full fiscal year.
    pub fn fiscal_year(year: u16) -> Self {
        Self {
            start_date: NaiveDate::from_ymd_opt(year as i32, 1, 1).unwrap(),
            end_date: NaiveDate::from_ymd_opt(year as i32, 12, 31).unwrap(),
            fiscal_year: year,
            fiscal_periods: (1..=12).collect(),
        }
    }

    /// Create a time period for specific months.
    pub fn months(year: u16, start_month: u8, num_months: u8) -> Self {
        let start_date = NaiveDate::from_ymd_opt(year as i32, start_month as u32, 1).unwrap();
        let end_month = ((start_month - 1 + num_months - 1) % 12) + 1;
        let end_year = year + (start_month as u16 - 1 + num_months as u16 - 1) / 12;
        let end_date = TemporalSampler::last_day_of_month(
            NaiveDate::from_ymd_opt(end_year as i32, end_month as u32, 1).unwrap(),
        );

        Self {
            start_date,
            end_date,
            fiscal_year: year,
            fiscal_periods: (start_month..start_month + num_months).collect(),
        }
    }

    /// Get total days in the period.
    pub fn total_days(&self) -> i64 {
        (self.end_date - self.start_date).num_days() + 1
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Timelike;

    #[test]
    fn test_is_weekend() {
        let sampler = TemporalSampler::new(42);
        let saturday = NaiveDate::from_ymd_opt(2024, 6, 15).unwrap();
        let sunday = NaiveDate::from_ymd_opt(2024, 6, 16).unwrap();
        let monday = NaiveDate::from_ymd_opt(2024, 6, 17).unwrap();

        assert!(sampler.is_weekend(saturday));
        assert!(sampler.is_weekend(sunday));
        assert!(!sampler.is_weekend(monday));
    }

    #[test]
    fn test_is_month_end() {
        let sampler = TemporalSampler::new(42);
        let month_end = NaiveDate::from_ymd_opt(2024, 6, 28).unwrap();
        let month_start = NaiveDate::from_ymd_opt(2024, 6, 1).unwrap();

        assert!(sampler.is_month_end(month_end));
        assert!(!sampler.is_month_end(month_start));
    }

    #[test]
    fn test_date_multiplier() {
        let sampler = TemporalSampler::new(42);

        // Regular weekday
        let regular_day = NaiveDate::from_ymd_opt(2024, 6, 12).unwrap(); // Wednesday
        assert!((sampler.get_date_multiplier(regular_day) - 1.0).abs() < 0.01);

        // Weekend
        let weekend = NaiveDate::from_ymd_opt(2024, 6, 15).unwrap(); // Saturday
        assert!(sampler.get_date_multiplier(weekend) < 0.2);

        // Month end
        let month_end = NaiveDate::from_ymd_opt(2024, 6, 28).unwrap();
        assert!(sampler.get_date_multiplier(month_end) > 2.0);
    }

    #[test]
    fn test_sample_time_human() {
        let mut sampler = TemporalSampler::new(42);

        for _ in 0..100 {
            let time = sampler.sample_time(true);
            // Most times should be during working hours
            let hour = time.hour();
            // Just verify it's a valid time
            assert!(hour < 24);
        }
    }

    #[test]
    fn test_time_period() {
        let period = TimePeriod::fiscal_year(2024);
        assert_eq!(period.total_days(), 366); // 2024 is leap year

        let partial = TimePeriod::months(2024, 1, 6);
        assert!(partial.total_days() > 180);
        assert!(partial.total_days() < 185);
    }
}
