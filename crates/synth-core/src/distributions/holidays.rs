//! Regional holiday calendars for transaction generation.
//!
//! Supports holidays for US, DE (Germany), GB (UK), CN (China),
//! JP (Japan), and IN (India) with appropriate activity multipliers.

use chrono::{Datelike, Duration, NaiveDate, Weekday};
use serde::{Deserialize, Serialize};

/// Supported regions for holiday calendars.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum Region {
    /// United States
    US,
    /// Germany
    DE,
    /// United Kingdom
    GB,
    /// China
    CN,
    /// Japan
    JP,
    /// India
    IN,
}

impl std::fmt::Display for Region {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Region::US => write!(f, "United States"),
            Region::DE => write!(f, "Germany"),
            Region::GB => write!(f, "United Kingdom"),
            Region::CN => write!(f, "China"),
            Region::JP => write!(f, "Japan"),
            Region::IN => write!(f, "India"),
        }
    }
}

/// A holiday with its associated activity multiplier.
#[derive(Debug, Clone)]
pub struct Holiday {
    /// Holiday name.
    pub name: String,
    /// Date of the holiday.
    pub date: NaiveDate,
    /// Activity multiplier (0.0 = completely closed, 1.0 = normal).
    pub activity_multiplier: f64,
    /// Whether this is a bank holiday (affects financial transactions).
    pub is_bank_holiday: bool,
}

impl Holiday {
    /// Create a new holiday.
    pub fn new(name: impl Into<String>, date: NaiveDate, multiplier: f64) -> Self {
        Self {
            name: name.into(),
            date,
            activity_multiplier: multiplier,
            is_bank_holiday: true,
        }
    }

    /// Set whether this is a bank holiday.
    pub fn with_bank_holiday(mut self, is_bank_holiday: bool) -> Self {
        self.is_bank_holiday = is_bank_holiday;
        self
    }
}

/// A calendar of holidays for a specific region and year.
#[derive(Debug, Clone)]
pub struct HolidayCalendar {
    /// Region for this calendar.
    pub region: Region,
    /// Year for this calendar.
    pub year: i32,
    /// List of holidays.
    pub holidays: Vec<Holiday>,
}

impl HolidayCalendar {
    /// Create a new empty holiday calendar.
    pub fn new(region: Region, year: i32) -> Self {
        Self {
            region,
            year,
            holidays: Vec::new(),
        }
    }

    /// Create a holiday calendar for a specific region and year.
    pub fn for_region(region: Region, year: i32) -> Self {
        match region {
            Region::US => Self::us_holidays(year),
            Region::DE => Self::de_holidays(year),
            Region::GB => Self::gb_holidays(year),
            Region::CN => Self::cn_holidays(year),
            Region::JP => Self::jp_holidays(year),
            Region::IN => Self::in_holidays(year),
        }
    }

    /// Check if a date is a holiday.
    pub fn is_holiday(&self, date: NaiveDate) -> bool {
        self.holidays.iter().any(|h| h.date == date)
    }

    /// Get the activity multiplier for a date.
    pub fn get_multiplier(&self, date: NaiveDate) -> f64 {
        self.holidays
            .iter()
            .find(|h| h.date == date)
            .map(|h| h.activity_multiplier)
            .unwrap_or(1.0)
    }

    /// Get all holidays for a date (may include multiple on same day).
    pub fn get_holidays(&self, date: NaiveDate) -> Vec<&Holiday> {
        self.holidays.iter().filter(|h| h.date == date).collect()
    }

    /// Add a holiday to the calendar.
    pub fn add_holiday(&mut self, holiday: Holiday) {
        self.holidays.push(holiday);
    }

    /// Get all dates in the calendar.
    pub fn all_dates(&self) -> Vec<NaiveDate> {
        self.holidays.iter().map(|h| h.date).collect()
    }

    /// US Federal Holidays.
    fn us_holidays(year: i32) -> Self {
        let mut cal = Self::new(Region::US, year);

        // New Year's Day - Jan 1 (observed)
        let new_years = NaiveDate::from_ymd_opt(year, 1, 1).unwrap();
        cal.add_holiday(Holiday::new("New Year's Day", Self::observe_weekend(new_years), 0.02));

        // Martin Luther King Jr. Day - 3rd Monday of January
        let mlk = Self::nth_weekday_of_month(year, 1, Weekday::Mon, 3);
        cal.add_holiday(Holiday::new("Martin Luther King Jr. Day", mlk, 0.1));

        // Presidents' Day - 3rd Monday of February
        let presidents = Self::nth_weekday_of_month(year, 2, Weekday::Mon, 3);
        cal.add_holiday(Holiday::new("Presidents' Day", presidents, 0.1));

        // Memorial Day - Last Monday of May
        let memorial = Self::last_weekday_of_month(year, 5, Weekday::Mon);
        cal.add_holiday(Holiday::new("Memorial Day", memorial, 0.05));

        // Juneteenth - June 19
        let juneteenth = NaiveDate::from_ymd_opt(year, 6, 19).unwrap();
        cal.add_holiday(Holiday::new("Juneteenth", Self::observe_weekend(juneteenth), 0.1));

        // Independence Day - July 4
        let independence = NaiveDate::from_ymd_opt(year, 7, 4).unwrap();
        cal.add_holiday(Holiday::new("Independence Day", Self::observe_weekend(independence), 0.02));

        // Labor Day - 1st Monday of September
        let labor = Self::nth_weekday_of_month(year, 9, Weekday::Mon, 1);
        cal.add_holiday(Holiday::new("Labor Day", labor, 0.05));

        // Columbus Day - 2nd Monday of October
        let columbus = Self::nth_weekday_of_month(year, 10, Weekday::Mon, 2);
        cal.add_holiday(Holiday::new("Columbus Day", columbus, 0.2));

        // Veterans Day - November 11
        let veterans = NaiveDate::from_ymd_opt(year, 11, 11).unwrap();
        cal.add_holiday(Holiday::new("Veterans Day", Self::observe_weekend(veterans), 0.1));

        // Thanksgiving - 4th Thursday of November
        let thanksgiving = Self::nth_weekday_of_month(year, 11, Weekday::Thu, 4);
        cal.add_holiday(Holiday::new("Thanksgiving", thanksgiving, 0.02));

        // Day after Thanksgiving
        cal.add_holiday(Holiday::new("Day after Thanksgiving", thanksgiving + Duration::days(1), 0.1));

        // Christmas Eve - December 24
        let christmas_eve = NaiveDate::from_ymd_opt(year, 12, 24).unwrap();
        cal.add_holiday(Holiday::new("Christmas Eve", christmas_eve, 0.1));

        // Christmas Day - December 25
        let christmas = NaiveDate::from_ymd_opt(year, 12, 25).unwrap();
        cal.add_holiday(Holiday::new("Christmas Day", Self::observe_weekend(christmas), 0.02));

        // New Year's Eve - December 31
        let new_years_eve = NaiveDate::from_ymd_opt(year, 12, 31).unwrap();
        cal.add_holiday(Holiday::new("New Year's Eve", new_years_eve, 0.1));

        cal
    }

    /// German holidays (nationwide).
    fn de_holidays(year: i32) -> Self {
        let mut cal = Self::new(Region::DE, year);

        // Neujahr - January 1
        cal.add_holiday(Holiday::new("Neujahr", NaiveDate::from_ymd_opt(year, 1, 1).unwrap(), 0.02));

        // Karfreitag - Good Friday (Easter - 2 days)
        let easter = Self::easter_date(year);
        cal.add_holiday(Holiday::new("Karfreitag", easter - Duration::days(2), 0.02));

        // Ostermontag - Easter Monday
        cal.add_holiday(Holiday::new("Ostermontag", easter + Duration::days(1), 0.02));

        // Tag der Arbeit - May 1
        cal.add_holiday(Holiday::new("Tag der Arbeit", NaiveDate::from_ymd_opt(year, 5, 1).unwrap(), 0.02));

        // Christi Himmelfahrt - Ascension Day (Easter + 39 days)
        cal.add_holiday(Holiday::new("Christi Himmelfahrt", easter + Duration::days(39), 0.02));

        // Pfingstmontag - Whit Monday (Easter + 50 days)
        cal.add_holiday(Holiday::new("Pfingstmontag", easter + Duration::days(50), 0.02));

        // Tag der Deutschen Einheit - October 3
        cal.add_holiday(Holiday::new("Tag der Deutschen Einheit", NaiveDate::from_ymd_opt(year, 10, 3).unwrap(), 0.02));

        // Weihnachten - December 25-26
        cal.add_holiday(Holiday::new("1. Weihnachtstag", NaiveDate::from_ymd_opt(year, 12, 25).unwrap(), 0.02));
        cal.add_holiday(Holiday::new("2. Weihnachtstag", NaiveDate::from_ymd_opt(year, 12, 26).unwrap(), 0.02));

        // Silvester - December 31
        cal.add_holiday(Holiday::new("Silvester", NaiveDate::from_ymd_opt(year, 12, 31).unwrap(), 0.1));

        cal
    }

    /// UK bank holidays.
    fn gb_holidays(year: i32) -> Self {
        let mut cal = Self::new(Region::GB, year);

        // New Year's Day
        let new_years = NaiveDate::from_ymd_opt(year, 1, 1).unwrap();
        cal.add_holiday(Holiday::new("New Year's Day", Self::observe_weekend(new_years), 0.02));

        // Good Friday
        let easter = Self::easter_date(year);
        cal.add_holiday(Holiday::new("Good Friday", easter - Duration::days(2), 0.02));

        // Easter Monday
        cal.add_holiday(Holiday::new("Easter Monday", easter + Duration::days(1), 0.02));

        // Early May Bank Holiday - 1st Monday of May
        let early_may = Self::nth_weekday_of_month(year, 5, Weekday::Mon, 1);
        cal.add_holiday(Holiday::new("Early May Bank Holiday", early_may, 0.02));

        // Spring Bank Holiday - Last Monday of May
        let spring = Self::last_weekday_of_month(year, 5, Weekday::Mon);
        cal.add_holiday(Holiday::new("Spring Bank Holiday", spring, 0.02));

        // Summer Bank Holiday - Last Monday of August
        let summer = Self::last_weekday_of_month(year, 8, Weekday::Mon);
        cal.add_holiday(Holiday::new("Summer Bank Holiday", summer, 0.02));

        // Christmas Day
        let christmas = NaiveDate::from_ymd_opt(year, 12, 25).unwrap();
        cal.add_holiday(Holiday::new("Christmas Day", Self::observe_weekend(christmas), 0.02));

        // Boxing Day
        let boxing = NaiveDate::from_ymd_opt(year, 12, 26).unwrap();
        cal.add_holiday(Holiday::new("Boxing Day", Self::observe_weekend(boxing), 0.02));

        cal
    }

    /// Chinese holidays (simplified - fixed dates only).
    fn cn_holidays(year: i32) -> Self {
        let mut cal = Self::new(Region::CN, year);

        // New Year's Day - January 1
        cal.add_holiday(Holiday::new("New Year", NaiveDate::from_ymd_opt(year, 1, 1).unwrap(), 0.05));

        // Spring Festival (Chinese New Year) - approximate late Jan/early Feb
        // Using a simplified calculation - typically 7-day holiday
        let cny = Self::approximate_chinese_new_year(year);
        for i in 0..7 {
            cal.add_holiday(Holiday::new(
                if i == 0 { "Spring Festival" } else { "Spring Festival Holiday" },
                cny + Duration::days(i),
                0.02,
            ));
        }

        // Qingming Festival - April 4-6 (approximate)
        cal.add_holiday(Holiday::new("Qingming Festival", NaiveDate::from_ymd_opt(year, 4, 5).unwrap(), 0.05));

        // Labor Day - May 1 (3-day holiday)
        for i in 0..3 {
            cal.add_holiday(Holiday::new(
                if i == 0 { "Labor Day" } else { "Labor Day Holiday" },
                NaiveDate::from_ymd_opt(year, 5, 1).unwrap() + Duration::days(i),
                0.05,
            ));
        }

        // Dragon Boat Festival - approximate early June
        cal.add_holiday(Holiday::new("Dragon Boat Festival", NaiveDate::from_ymd_opt(year, 6, 10).unwrap(), 0.05));

        // Mid-Autumn Festival - approximate late September
        cal.add_holiday(Holiday::new("Mid-Autumn Festival", NaiveDate::from_ymd_opt(year, 9, 15).unwrap(), 0.05));

        // National Day - October 1 (7-day holiday)
        for i in 0..7 {
            cal.add_holiday(Holiday::new(
                if i == 0 { "National Day" } else { "National Day Holiday" },
                NaiveDate::from_ymd_opt(year, 10, 1).unwrap() + Duration::days(i),
                0.02,
            ));
        }

        cal
    }

    /// Japanese holidays.
    fn jp_holidays(year: i32) -> Self {
        let mut cal = Self::new(Region::JP, year);

        // Ganjitsu - January 1
        cal.add_holiday(Holiday::new("Ganjitsu (New Year)", NaiveDate::from_ymd_opt(year, 1, 1).unwrap(), 0.02));

        // New Year holidays - January 2-3
        cal.add_holiday(Holiday::new("New Year Holiday", NaiveDate::from_ymd_opt(year, 1, 2).unwrap(), 0.05));
        cal.add_holiday(Holiday::new("New Year Holiday", NaiveDate::from_ymd_opt(year, 1, 3).unwrap(), 0.05));

        // Seijin no Hi - Coming of Age Day - 2nd Monday of January
        let seijin = Self::nth_weekday_of_month(year, 1, Weekday::Mon, 2);
        cal.add_holiday(Holiday::new("Seijin no Hi", seijin, 0.05));

        // Kenkoku Kinen no Hi - National Foundation Day - February 11
        cal.add_holiday(Holiday::new("Kenkoku Kinen no Hi", NaiveDate::from_ymd_opt(year, 2, 11).unwrap(), 0.02));

        // Tenno Tanjobi - Emperor's Birthday - February 23
        cal.add_holiday(Holiday::new("Tenno Tanjobi", NaiveDate::from_ymd_opt(year, 2, 23).unwrap(), 0.02));

        // Shunbun no Hi - Vernal Equinox - around March 20-21
        cal.add_holiday(Holiday::new("Shunbun no Hi", NaiveDate::from_ymd_opt(year, 3, 20).unwrap(), 0.02));

        // Showa no Hi - Showa Day - April 29
        cal.add_holiday(Holiday::new("Showa no Hi", NaiveDate::from_ymd_opt(year, 4, 29).unwrap(), 0.02));

        // Golden Week - April 29 - May 5
        cal.add_holiday(Holiday::new("Kenpo Kinenbi", NaiveDate::from_ymd_opt(year, 5, 3).unwrap(), 0.02));
        cal.add_holiday(Holiday::new("Midori no Hi", NaiveDate::from_ymd_opt(year, 5, 4).unwrap(), 0.02));
        cal.add_holiday(Holiday::new("Kodomo no Hi", NaiveDate::from_ymd_opt(year, 5, 5).unwrap(), 0.02));

        // Umi no Hi - Marine Day - 3rd Monday of July
        let umi = Self::nth_weekday_of_month(year, 7, Weekday::Mon, 3);
        cal.add_holiday(Holiday::new("Umi no Hi", umi, 0.05));

        // Yama no Hi - Mountain Day - August 11
        cal.add_holiday(Holiday::new("Yama no Hi", NaiveDate::from_ymd_opt(year, 8, 11).unwrap(), 0.05));

        // Keiro no Hi - Respect for the Aged Day - 3rd Monday of September
        let keiro = Self::nth_weekday_of_month(year, 9, Weekday::Mon, 3);
        cal.add_holiday(Holiday::new("Keiro no Hi", keiro, 0.05));

        // Shubun no Hi - Autumnal Equinox - around September 22-23
        cal.add_holiday(Holiday::new("Shubun no Hi", NaiveDate::from_ymd_opt(year, 9, 23).unwrap(), 0.02));

        // Sports Day - 2nd Monday of October
        let sports = Self::nth_weekday_of_month(year, 10, Weekday::Mon, 2);
        cal.add_holiday(Holiday::new("Sports Day", sports, 0.05));

        // Bunka no Hi - Culture Day - November 3
        cal.add_holiday(Holiday::new("Bunka no Hi", NaiveDate::from_ymd_opt(year, 11, 3).unwrap(), 0.02));

        // Kinro Kansha no Hi - Labor Thanksgiving Day - November 23
        cal.add_holiday(Holiday::new("Kinro Kansha no Hi", NaiveDate::from_ymd_opt(year, 11, 23).unwrap(), 0.02));

        cal
    }

    /// Indian holidays (national holidays).
    fn in_holidays(year: i32) -> Self {
        let mut cal = Self::new(Region::IN, year);

        // Republic Day - January 26
        cal.add_holiday(Holiday::new("Republic Day", NaiveDate::from_ymd_opt(year, 1, 26).unwrap(), 0.02));

        // Holi - approximate March (lunar calendar)
        cal.add_holiday(Holiday::new("Holi", NaiveDate::from_ymd_opt(year, 3, 10).unwrap(), 0.05));

        // Good Friday
        let easter = Self::easter_date(year);
        cal.add_holiday(Holiday::new("Good Friday", easter - Duration::days(2), 0.05));

        // Independence Day - August 15
        cal.add_holiday(Holiday::new("Independence Day", NaiveDate::from_ymd_opt(year, 8, 15).unwrap(), 0.02));

        // Gandhi Jayanti - October 2
        cal.add_holiday(Holiday::new("Gandhi Jayanti", NaiveDate::from_ymd_opt(year, 10, 2).unwrap(), 0.02));

        // Dussehra - approximate October (lunar calendar)
        cal.add_holiday(Holiday::new("Dussehra", NaiveDate::from_ymd_opt(year, 10, 15).unwrap(), 0.05));

        // Diwali - approximate October/November (5-day festival)
        let diwali = Self::approximate_diwali(year);
        for i in 0..5 {
            cal.add_holiday(Holiday::new(
                match i {
                    0 => "Dhanteras",
                    1 => "Naraka Chaturdashi",
                    2 => "Diwali",
                    3 => "Govardhan Puja",
                    _ => "Bhai Dooj",
                },
                diwali + Duration::days(i),
                if i == 2 { 0.02 } else { 0.1 },
            ));
        }

        // Christmas - December 25
        cal.add_holiday(Holiday::new("Christmas", NaiveDate::from_ymd_opt(year, 12, 25).unwrap(), 0.1));

        cal
    }

    /// Calculate Easter date using the anonymous Gregorian algorithm.
    fn easter_date(year: i32) -> NaiveDate {
        let a = year % 19;
        let b = year / 100;
        let c = year % 100;
        let d = b / 4;
        let e = b % 4;
        let f = (b + 8) / 25;
        let g = (b - f + 1) / 3;
        let h = (19 * a + b - d - g + 15) % 30;
        let i = c / 4;
        let k = c % 4;
        let l = (32 + 2 * e + 2 * i - h - k) % 7;
        let m = (a + 11 * h + 22 * l) / 451;
        let month = (h + l - 7 * m + 114) / 31;
        let day = ((h + l - 7 * m + 114) % 31) + 1;

        NaiveDate::from_ymd_opt(year, month as u32, day as u32).unwrap()
    }

    /// Get nth weekday of a month (e.g., 3rd Monday of January).
    fn nth_weekday_of_month(year: i32, month: u32, weekday: Weekday, n: u32) -> NaiveDate {
        let first = NaiveDate::from_ymd_opt(year, month, 1).unwrap();
        let first_weekday = first.weekday();

        let days_until = (weekday.num_days_from_monday() as i64
            - first_weekday.num_days_from_monday() as i64
            + 7)
            % 7;

        let date = first + Duration::days(days_until + (n - 1) as i64 * 7);
        date
    }

    /// Get last weekday of a month (e.g., last Monday of May).
    fn last_weekday_of_month(year: i32, month: u32, weekday: Weekday) -> NaiveDate {
        let last = if month == 12 {
            NaiveDate::from_ymd_opt(year + 1, 1, 1).unwrap() - Duration::days(1)
        } else {
            NaiveDate::from_ymd_opt(year, month + 1, 1).unwrap() - Duration::days(1)
        };

        let last_weekday = last.weekday();
        let days_back = (last_weekday.num_days_from_monday() as i64
            - weekday.num_days_from_monday() as i64
            + 7)
            % 7;

        last - Duration::days(days_back)
    }

    /// Observe weekend holidays on nearest weekday.
    fn observe_weekend(date: NaiveDate) -> NaiveDate {
        match date.weekday() {
            Weekday::Sat => date - Duration::days(1), // Friday
            Weekday::Sun => date + Duration::days(1), // Monday
            _ => date,
        }
    }

    /// Approximate Chinese New Year date (simplified calculation).
    fn approximate_chinese_new_year(year: i32) -> NaiveDate {
        // Chinese New Year falls between Jan 21 and Feb 20
        // This is a simplified approximation
        let base_year = 2000;
        let cny_2000 = NaiveDate::from_ymd_opt(2000, 2, 5).unwrap();

        let years_diff = year - base_year;
        let lunar_cycle = 29.5306; // days per lunar month
        let days_offset = (years_diff as f64 * 12.0 * lunar_cycle) % 365.25;

        let mut result = cny_2000 + Duration::days(days_offset as i64);

        // Ensure it falls in Jan-Feb range
        while result.month() > 2 || (result.month() == 2 && result.day() > 20) {
            result = result - Duration::days(29);
        }
        while result.month() < 1 || (result.month() == 1 && result.day() < 21) {
            result = result + Duration::days(29);
        }

        // Adjust year if needed
        if result.year() != year {
            let year_diff = year - result.year();
            result = NaiveDate::from_ymd_opt(
                year,
                result.month(),
                result.day().min(28),
            ).unwrap_or_else(|| NaiveDate::from_ymd_opt(year, result.month(), 28).unwrap());
        }

        result
    }

    /// Approximate Diwali date (simplified calculation).
    fn approximate_diwali(year: i32) -> NaiveDate {
        // Diwali typically falls in October-November
        // This is a simplified approximation
        match year % 4 {
            0 => NaiveDate::from_ymd_opt(year, 11, 1).unwrap(),
            1 => NaiveDate::from_ymd_opt(year, 10, 24).unwrap(),
            2 => NaiveDate::from_ymd_opt(year, 11, 12).unwrap(),
            _ => NaiveDate::from_ymd_opt(year, 11, 4).unwrap(),
        }
    }
}

/// Custom holiday configuration for YAML/JSON input.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomHolidayConfig {
    /// Holiday name.
    pub name: String,
    /// Month (1-12).
    pub month: u8,
    /// Day of month.
    pub day: u8,
    /// Activity multiplier (optional, defaults to 0.05).
    #[serde(default = "default_holiday_multiplier")]
    pub activity_multiplier: f64,
}

fn default_holiday_multiplier() -> f64 {
    0.05
}

impl CustomHolidayConfig {
    /// Convert to a Holiday for a specific year.
    pub fn to_holiday(&self, year: i32) -> Holiday {
        Holiday::new(
            &self.name,
            NaiveDate::from_ymd_opt(year, self.month as u32, self.day as u32).unwrap(),
            self.activity_multiplier,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_us_holidays() {
        let cal = HolidayCalendar::for_region(Region::US, 2024);

        // Check some specific holidays exist
        let christmas = NaiveDate::from_ymd_opt(2024, 12, 25).unwrap();
        assert!(cal.is_holiday(christmas));

        // Independence Day (observed on Friday since July 4 is Thursday in 2024)
        let independence = NaiveDate::from_ymd_opt(2024, 7, 4).unwrap();
        assert!(cal.is_holiday(independence));
    }

    #[test]
    fn test_german_holidays() {
        let cal = HolidayCalendar::for_region(Region::DE, 2024);

        // Tag der Deutschen Einheit - October 3
        let unity = NaiveDate::from_ymd_opt(2024, 10, 3).unwrap();
        assert!(cal.is_holiday(unity));
    }

    #[test]
    fn test_easter_calculation() {
        // Known Easter dates
        assert_eq!(
            HolidayCalendar::easter_date(2024),
            NaiveDate::from_ymd_opt(2024, 3, 31).unwrap()
        );
        assert_eq!(
            HolidayCalendar::easter_date(2025),
            NaiveDate::from_ymd_opt(2025, 4, 20).unwrap()
        );
    }

    #[test]
    fn test_nth_weekday() {
        // 3rd Monday of January 2024
        let mlk = HolidayCalendar::nth_weekday_of_month(2024, 1, Weekday::Mon, 3);
        assert_eq!(mlk, NaiveDate::from_ymd_opt(2024, 1, 15).unwrap());

        // 4th Thursday of November 2024 (Thanksgiving)
        let thanksgiving = HolidayCalendar::nth_weekday_of_month(2024, 11, Weekday::Thu, 4);
        assert_eq!(thanksgiving, NaiveDate::from_ymd_opt(2024, 11, 28).unwrap());
    }

    #[test]
    fn test_last_weekday() {
        // Last Monday of May 2024 (Memorial Day)
        let memorial = HolidayCalendar::last_weekday_of_month(2024, 5, Weekday::Mon);
        assert_eq!(memorial, NaiveDate::from_ymd_opt(2024, 5, 27).unwrap());
    }

    #[test]
    fn test_activity_multiplier() {
        let cal = HolidayCalendar::for_region(Region::US, 2024);

        // Holiday should have low multiplier
        let christmas = NaiveDate::from_ymd_opt(2024, 12, 25).unwrap();
        assert!(cal.get_multiplier(christmas) < 0.1);

        // Regular day should be 1.0
        let regular = NaiveDate::from_ymd_opt(2024, 6, 15).unwrap();
        assert!((cal.get_multiplier(regular) - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_all_regions_have_holidays() {
        let regions = [Region::US, Region::DE, Region::GB, Region::CN, Region::JP, Region::IN];

        for region in regions {
            let cal = HolidayCalendar::for_region(region, 2024);
            assert!(
                !cal.holidays.is_empty(),
                "Region {:?} should have holidays",
                region
            );
        }
    }

    #[test]
    fn test_chinese_holidays() {
        let cal = HolidayCalendar::for_region(Region::CN, 2024);

        // National Day - October 1
        let national = NaiveDate::from_ymd_opt(2024, 10, 1).unwrap();
        assert!(cal.is_holiday(national));
    }

    #[test]
    fn test_japanese_golden_week() {
        let cal = HolidayCalendar::for_region(Region::JP, 2024);

        // Check Golden Week holidays
        let kodomo = NaiveDate::from_ymd_opt(2024, 5, 5).unwrap();
        assert!(cal.is_holiday(kodomo));
    }
}
