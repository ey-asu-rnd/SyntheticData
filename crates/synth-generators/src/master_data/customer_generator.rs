//! Enhanced customer generator with credit management and payment behavior.

use chrono::NaiveDate;
use rand::prelude::*;
use rand_chacha::ChaCha8Rng;
use rust_decimal::Decimal;
use synth_core::models::{
    CreditRating, Customer, CustomerPaymentBehavior, CustomerPool, PaymentTerms,
};

/// Configuration for customer generation.
#[derive(Debug, Clone)]
pub struct CustomerGeneratorConfig {
    /// Distribution of credit ratings (rating, probability)
    pub credit_rating_distribution: Vec<(CreditRating, f64)>,
    /// Distribution of payment behaviors (behavior, probability)
    pub payment_behavior_distribution: Vec<(CustomerPaymentBehavior, f64)>,
    /// Distribution of payment terms (terms, probability)
    pub payment_terms_distribution: Vec<(PaymentTerms, f64)>,
    /// Probability of customer being intercompany
    pub intercompany_rate: f64,
    /// Default country for customers
    pub default_country: String,
    /// Default currency
    pub default_currency: String,
    /// Credit limit ranges by rating (min, max)
    pub credit_limits: Vec<(CreditRating, Decimal, Decimal)>,
}

impl Default for CustomerGeneratorConfig {
    fn default() -> Self {
        Self {
            credit_rating_distribution: vec![
                (CreditRating::AAA, 0.05),
                (CreditRating::AA, 0.10),
                (CreditRating::A, 0.25),
                (CreditRating::BBB, 0.30),
                (CreditRating::BB, 0.15),
                (CreditRating::B, 0.10),
                (CreditRating::CCC, 0.04),
                (CreditRating::D, 0.01),
            ],
            payment_behavior_distribution: vec![
                (CustomerPaymentBehavior::EarlyPayer, 0.15),
                (CustomerPaymentBehavior::OnTime, 0.45),
                (CustomerPaymentBehavior::SlightlyLate, 0.25),
                (CustomerPaymentBehavior::OftenLate, 0.10),
                (CustomerPaymentBehavior::HighRisk, 0.05),
            ],
            payment_terms_distribution: vec![
                (PaymentTerms::Net30, 0.50),
                (PaymentTerms::Net60, 0.20),
                (PaymentTerms::TwoTenNet30, 0.20),
                (PaymentTerms::Net15, 0.05),
                (PaymentTerms::Immediate, 0.05),
            ],
            intercompany_rate: 0.05,
            default_country: "US".to_string(),
            default_currency: "USD".to_string(),
            credit_limits: vec![
                (CreditRating::AAA, Decimal::from(1_000_000), Decimal::from(10_000_000)),
                (CreditRating::AA, Decimal::from(500_000), Decimal::from(2_000_000)),
                (CreditRating::A, Decimal::from(250_000), Decimal::from(1_000_000)),
                (CreditRating::BBB, Decimal::from(100_000), Decimal::from(500_000)),
                (CreditRating::BB, Decimal::from(50_000), Decimal::from(250_000)),
                (CreditRating::B, Decimal::from(25_000), Decimal::from(100_000)),
                (CreditRating::CCC, Decimal::from(10_000), Decimal::from(50_000)),
                (CreditRating::D, Decimal::from(0), Decimal::from(10_000)),
            ],
        }
    }
}

/// Customer name templates by industry.
const CUSTOMER_NAME_TEMPLATES: &[(&str, &[&str])] = &[
    ("Retail", &[
        "Consumer Goods Corp.",
        "Retail Solutions Inc.",
        "Shop Direct Ltd.",
        "Market Leaders LLC",
        "Consumer Brands Group",
        "Retail Partners Co.",
        "Shopping Networks Inc.",
        "Direct Sales Corp.",
    ]),
    ("Manufacturing", &[
        "Industrial Manufacturing Inc.",
        "Production Systems Corp.",
        "Assembly Technologies LLC",
        "Manufacturing Partners Group",
        "Factory Solutions Ltd.",
        "Production Line Inc.",
        "Industrial Works Corp.",
        "Manufacturing Excellence Co.",
    ]),
    ("Healthcare", &[
        "Healthcare Systems Inc.",
        "Medical Solutions Corp.",
        "Health Partners LLC",
        "Medical Equipment Group",
        "Healthcare Providers Ltd.",
        "Clinical Services Inc.",
        "Health Networks Corp.",
        "Medical Supplies Co.",
    ]),
    ("Technology", &[
        "Tech Innovations Inc.",
        "Digital Solutions Corp.",
        "Software Systems LLC",
        "Technology Partners Group",
        "IT Solutions Ltd.",
        "Tech Enterprises Inc.",
        "Digital Networks Corp.",
        "Innovation Labs Co.",
    ]),
    ("Financial", &[
        "Financial Services Inc.",
        "Banking Solutions Corp.",
        "Investment Partners LLC",
        "Financial Networks Group",
        "Capital Services Ltd.",
        "Banking Partners Inc.",
        "Finance Solutions Corp.",
        "Investment Group Co.",
    ]),
    ("Energy", &[
        "Energy Solutions Inc.",
        "Power Systems Corp.",
        "Renewable Partners LLC",
        "Energy Networks Group",
        "Utility Services Ltd.",
        "Power Generation Inc.",
        "Energy Partners Corp.",
        "Sustainable Energy Co.",
    ]),
    ("Transportation", &[
        "Transport Solutions Inc.",
        "Logistics Systems Corp.",
        "Freight Partners LLC",
        "Transportation Networks Group",
        "Shipping Services Ltd.",
        "Fleet Management Inc.",
        "Logistics Partners Corp.",
        "Transport Dynamics Co.",
    ]),
    ("Construction", &[
        "Construction Solutions Inc.",
        "Building Systems Corp.",
        "Development Partners LLC",
        "Construction Group Ltd.",
        "Building Services Inc.",
        "Property Development Corp.",
        "Construction Partners Co.",
        "Infrastructure Systems LLC",
    ]),
];

/// Generator for customer master data.
pub struct CustomerGenerator {
    rng: ChaCha8Rng,
    seed: u64,
    config: CustomerGeneratorConfig,
    customer_counter: usize,
}

impl CustomerGenerator {
    /// Create a new customer generator.
    pub fn new(seed: u64) -> Self {
        Self::with_config(seed, CustomerGeneratorConfig::default())
    }

    /// Create a new customer generator with custom configuration.
    pub fn with_config(seed: u64, config: CustomerGeneratorConfig) -> Self {
        Self {
            rng: ChaCha8Rng::seed_from_u64(seed),
            seed,
            config,
            customer_counter: 0,
        }
    }

    /// Generate a single customer.
    pub fn generate_customer(
        &mut self,
        company_code: &str,
        effective_date: NaiveDate,
    ) -> Customer {
        self.customer_counter += 1;

        let customer_id = format!("C-{:06}", self.customer_counter);
        let (industry, name) = self.select_customer_name();

        let mut customer = Customer::new(
            customer_id,
            name.to_string(),
            company_code.to_string(),
        );

        customer.country = self.config.default_country.clone();
        customer.currency = self.config.default_currency.clone();
        customer.industry = Some(industry.to_string());
        customer.effective_date = effective_date;

        // Set credit rating and limit
        customer.credit_rating = self.select_credit_rating();
        customer.credit_limit = self.generate_credit_limit(&customer.credit_rating);

        // Set payment behavior
        customer.payment_behavior = self.select_payment_behavior();

        // Set payment terms
        customer.payment_terms = self.select_payment_terms();

        // Check if intercompany
        if self.rng.gen::<f64>() < self.config.intercompany_rate {
            customer.is_intercompany = true;
            customer.intercompany_code = Some(format!("IC-{}", company_code));
        }

        // Set address
        customer.address = Some(self.generate_address());

        // Generate contact info
        customer.contact_name = Some(self.generate_contact_name());
        customer.contact_email = Some(self.generate_contact_email(&customer.name));

        customer
    }

    /// Generate an intercompany customer (always intercompany).
    pub fn generate_intercompany_customer(
        &mut self,
        company_code: &str,
        partner_company_code: &str,
        effective_date: NaiveDate,
    ) -> Customer {
        let mut customer = self.generate_customer(company_code, effective_date);
        customer.is_intercompany = true;
        customer.intercompany_code = Some(partner_company_code.to_string());
        customer.name = format!("{} - IC", partner_company_code);
        customer.credit_rating = CreditRating::AAA; // IC always highest rating
        customer.credit_limit = Decimal::from(100_000_000); // High limit for IC
        customer.payment_behavior = CustomerPaymentBehavior::OnTime;
        customer
    }

    /// Generate a customer with specific credit profile.
    pub fn generate_customer_with_credit(
        &mut self,
        company_code: &str,
        credit_rating: CreditRating,
        credit_limit: Decimal,
        effective_date: NaiveDate,
    ) -> Customer {
        let mut customer = self.generate_customer(company_code, effective_date);
        customer.credit_rating = credit_rating;
        customer.credit_limit = credit_limit;

        // Adjust payment behavior based on credit rating
        customer.payment_behavior = match credit_rating {
            CreditRating::AAA | CreditRating::AA => {
                if self.rng.gen::<f64>() < 0.7 {
                    CustomerPaymentBehavior::EarlyPayer
                } else {
                    CustomerPaymentBehavior::OnTime
                }
            }
            CreditRating::A | CreditRating::BBB => CustomerPaymentBehavior::OnTime,
            CreditRating::BB | CreditRating::B => CustomerPaymentBehavior::SlightlyLate,
            CreditRating::CCC => CustomerPaymentBehavior::OftenLate,
            CreditRating::D => CustomerPaymentBehavior::HighRisk,
        };

        customer
    }

    /// Generate a customer pool with specified count.
    pub fn generate_customer_pool(
        &mut self,
        count: usize,
        company_code: &str,
        effective_date: NaiveDate,
    ) -> CustomerPool {
        let mut pool = CustomerPool::new();

        for _ in 0..count {
            let customer = self.generate_customer(company_code, effective_date);
            pool.add_customer(customer);
        }

        pool
    }

    /// Generate a customer pool with intercompany customers.
    pub fn generate_customer_pool_with_ic(
        &mut self,
        count: usize,
        company_code: &str,
        partner_company_codes: &[String],
        effective_date: NaiveDate,
    ) -> CustomerPool {
        let mut pool = CustomerPool::new();

        // Generate regular customers
        let regular_count = count.saturating_sub(partner_company_codes.len());
        for _ in 0..regular_count {
            let customer = self.generate_customer(company_code, effective_date);
            pool.add_customer(customer);
        }

        // Generate IC customers for each partner
        for partner in partner_company_codes {
            let customer = self.generate_intercompany_customer(company_code, partner, effective_date);
            pool.add_customer(customer);
        }

        pool
    }

    /// Generate a diverse customer pool with various credit profiles.
    pub fn generate_diverse_pool(
        &mut self,
        count: usize,
        company_code: &str,
        effective_date: NaiveDate,
    ) -> CustomerPool {
        let mut pool = CustomerPool::new();

        // Generate customers with varied credit ratings ensuring coverage
        let rating_counts = [
            (CreditRating::AAA, (count as f64 * 0.05) as usize),
            (CreditRating::AA, (count as f64 * 0.10) as usize),
            (CreditRating::A, (count as f64 * 0.20) as usize),
            (CreditRating::BBB, (count as f64 * 0.30) as usize),
            (CreditRating::BB, (count as f64 * 0.15) as usize),
            (CreditRating::B, (count as f64 * 0.10) as usize),
            (CreditRating::CCC, (count as f64 * 0.07) as usize),
            (CreditRating::D, (count as f64 * 0.03) as usize),
        ];

        for (rating, rating_count) in rating_counts {
            for _ in 0..rating_count {
                let credit_limit = self.generate_credit_limit(&rating);
                let customer = self.generate_customer_with_credit(
                    company_code,
                    rating,
                    credit_limit,
                    effective_date,
                );
                pool.add_customer(customer);
            }
        }

        // Fill any remaining slots
        while pool.customers.len() < count {
            let customer = self.generate_customer(company_code, effective_date);
            pool.add_customer(customer);
        }

        pool
    }

    /// Select a customer name from templates.
    fn select_customer_name(&mut self) -> (&'static str, &'static str) {
        let industry_idx = self.rng.gen_range(0..CUSTOMER_NAME_TEMPLATES.len());
        let (industry, names) = CUSTOMER_NAME_TEMPLATES[industry_idx];
        let name_idx = self.rng.gen_range(0..names.len());
        (industry, names[name_idx])
    }

    /// Select credit rating based on distribution.
    fn select_credit_rating(&mut self) -> CreditRating {
        let roll: f64 = self.rng.gen();
        let mut cumulative = 0.0;

        for (rating, prob) in &self.config.credit_rating_distribution {
            cumulative += prob;
            if roll < cumulative {
                return *rating;
            }
        }

        CreditRating::BBB
    }

    /// Generate credit limit for rating.
    fn generate_credit_limit(&mut self, rating: &CreditRating) -> Decimal {
        for (r, min, max) in &self.config.credit_limits {
            if r == rating {
                let range = (*max - *min).to_string().parse::<f64>().unwrap_or(0.0);
                let offset = Decimal::from_f64_retain(self.rng.gen::<f64>() * range)
                    .unwrap_or(Decimal::ZERO);
                return *min + offset;
            }
        }

        Decimal::from(100_000)
    }

    /// Select payment behavior based on distribution.
    fn select_payment_behavior(&mut self) -> CustomerPaymentBehavior {
        let roll: f64 = self.rng.gen();
        let mut cumulative = 0.0;

        for (behavior, prob) in &self.config.payment_behavior_distribution {
            cumulative += prob;
            if roll < cumulative {
                return *behavior;
            }
        }

        CustomerPaymentBehavior::OnTime
    }

    /// Select payment terms based on distribution.
    fn select_payment_terms(&mut self) -> PaymentTerms {
        let roll: f64 = self.rng.gen();
        let mut cumulative = 0.0;

        for (terms, prob) in &self.config.payment_terms_distribution {
            cumulative += prob;
            if roll < cumulative {
                return *terms;
            }
        }

        PaymentTerms::Net30
    }

    /// Generate an address.
    fn generate_address(&mut self) -> String {
        let street_num = self.rng.gen_range(1..9999);
        let streets = [
            "Corporate Dr", "Business Center", "Commerce Way",
            "Executive Plaza", "Industry Park", "Trade Center",
        ];
        let cities = [
            "New York", "Los Angeles", "Chicago", "Houston",
            "Phoenix", "Philadelphia", "San Antonio", "San Diego",
        ];
        let states = ["NY", "CA", "IL", "TX", "AZ", "PA", "TX", "CA"];

        let idx = self.rng.gen_range(0..cities.len());
        let street_idx = self.rng.gen_range(0..streets.len());
        let zip = self.rng.gen_range(10000..99999);

        format!(
            "{} {}, {}, {} {}",
            street_num, streets[street_idx], cities[idx], states[idx], zip
        )
    }

    /// Generate a contact name.
    fn generate_contact_name(&mut self) -> String {
        let first_names = ["John", "Jane", "Michael", "Sarah", "David", "Emily", "Robert", "Lisa"];
        let last_names = ["Smith", "Johnson", "Williams", "Brown", "Jones", "Garcia", "Miller", "Davis"];

        let first = first_names[self.rng.gen_range(0..first_names.len())];
        let last = last_names[self.rng.gen_range(0..last_names.len())];

        format!("{} {}", first, last)
    }

    /// Generate a contact email.
    fn generate_contact_email(&mut self, company_name: &str) -> String {
        let domain = company_name
            .to_lowercase()
            .replace(' ', "")
            .replace('.', "")
            .replace(',', "")
            .chars()
            .filter(|c| c.is_alphanumeric())
            .take(15)
            .collect::<String>();

        format!("contact@{}.com", domain)
    }

    /// Reset the generator.
    pub fn reset(&mut self) {
        self.rng = ChaCha8Rng::seed_from_u64(self.seed);
        self.customer_counter = 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_customer_generation() {
        let mut gen = CustomerGenerator::new(42);
        let customer = gen.generate_customer(
            "1000",
            NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
        );

        assert!(!customer.customer_id.is_empty());
        assert!(!customer.name.is_empty());
        assert!(customer.credit_limit > Decimal::ZERO);
    }

    #[test]
    fn test_customer_pool_generation() {
        let mut gen = CustomerGenerator::new(42);
        let pool = gen.generate_customer_pool(
            20,
            "1000",
            NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
        );

        assert_eq!(pool.customers.len(), 20);
    }

    #[test]
    fn test_intercompany_customer() {
        let mut gen = CustomerGenerator::new(42);
        let customer = gen.generate_intercompany_customer(
            "1000",
            "2000",
            NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
        );

        assert!(customer.is_intercompany);
        assert_eq!(customer.intercompany_code, Some("2000".to_string()));
        assert_eq!(customer.credit_rating, CreditRating::AAA);
    }

    #[test]
    fn test_diverse_pool() {
        let mut gen = CustomerGenerator::new(42);
        let pool = gen.generate_diverse_pool(
            100,
            "1000",
            NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
        );

        // Should have customers with various credit ratings
        let aaa_count = pool.customers.iter()
            .filter(|c| c.credit_rating == CreditRating::AAA)
            .count();
        let d_count = pool.customers.iter()
            .filter(|c| c.credit_rating == CreditRating::D)
            .count();

        assert!(aaa_count > 0);
        assert!(d_count > 0);
    }

    #[test]
    fn test_deterministic_generation() {
        let mut gen1 = CustomerGenerator::new(42);
        let mut gen2 = CustomerGenerator::new(42);

        let customer1 = gen1.generate_customer(
            "1000",
            NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
        );
        let customer2 = gen2.generate_customer(
            "1000",
            NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
        );

        assert_eq!(customer1.customer_id, customer2.customer_id);
        assert_eq!(customer1.name, customer2.name);
        assert_eq!(customer1.credit_rating, customer2.credit_rating);
    }

    #[test]
    fn test_customer_with_specific_credit() {
        let mut gen = CustomerGenerator::new(42);
        let customer = gen.generate_customer_with_credit(
            "1000",
            CreditRating::D,
            Decimal::from(5000),
            NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
        );

        assert_eq!(customer.credit_rating, CreditRating::D);
        assert_eq!(customer.credit_limit, Decimal::from(5000));
        assert_eq!(customer.payment_behavior, CustomerPaymentBehavior::HighRisk);
    }
}
