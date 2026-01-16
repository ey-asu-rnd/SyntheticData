//! Enhanced vendor generator with realistic payment behavior and bank accounts.

use chrono::NaiveDate;
use rand::prelude::*;
use rand_chacha::ChaCha8Rng;
use rust_decimal::Decimal;
use synth_core::models::{BankAccount, PaymentTerms, Vendor, VendorBehavior, VendorPool};

/// Configuration for vendor generation.
#[derive(Debug, Clone)]
pub struct VendorGeneratorConfig {
    /// Distribution of payment terms (terms, probability)
    pub payment_terms_distribution: Vec<(PaymentTerms, f64)>,
    /// Distribution of vendor behaviors (behavior, probability)
    pub behavior_distribution: Vec<(VendorBehavior, f64)>,
    /// Probability of vendor being intercompany
    pub intercompany_rate: f64,
    /// Default country for vendors
    pub default_country: String,
    /// Default currency
    pub default_currency: String,
    /// Generate bank accounts
    pub generate_bank_accounts: bool,
    /// Probability of vendor having multiple bank accounts
    pub multiple_bank_account_rate: f64,
}

impl Default for VendorGeneratorConfig {
    fn default() -> Self {
        Self {
            payment_terms_distribution: vec![
                (PaymentTerms::Net30, 0.40),
                (PaymentTerms::Net60, 0.20),
                (PaymentTerms::TwoTenNet30, 0.25),
                (PaymentTerms::Net15, 0.10),
                (PaymentTerms::Immediate, 0.05),
            ],
            behavior_distribution: vec![
                (VendorBehavior::Flexible, 0.60),
                (VendorBehavior::Strict, 0.25),
                (VendorBehavior::VeryFlexible, 0.10),
                (VendorBehavior::Aggressive, 0.05),
            ],
            intercompany_rate: 0.05,
            default_country: "US".to_string(),
            default_currency: "USD".to_string(),
            generate_bank_accounts: true,
            multiple_bank_account_rate: 0.20,
        }
    }
}

/// Vendor name templates by category.
const VENDOR_NAME_TEMPLATES: &[(&str, &[&str])] = &[
    (
        "Manufacturing",
        &[
            "Global Manufacturing Solutions",
            "Precision Parts Inc.",
            "Industrial Components Ltd.",
            "Advanced Materials Corp.",
            "Quality Fabrication Services",
            "Metalworks International",
            "Polymer Technologies",
            "Assembly Dynamics",
        ],
    ),
    (
        "Services",
        &[
            "Professional Services Group",
            "Consulting Partners LLC",
            "Business Solutions Inc.",
            "Technical Services Corp.",
            "Support Systems International",
            "Managed Services Ltd.",
            "Advisory Group Partners",
            "Strategic Consulting Co.",
        ],
    ),
    (
        "Technology",
        &[
            "Tech Solutions Inc.",
            "Digital Systems Corp.",
            "Software Innovations LLC",
            "Cloud Services Partners",
            "IT Infrastructure Group",
            "Data Systems International",
            "Network Solutions Ltd.",
            "Cyber Systems Corp.",
        ],
    ),
    (
        "Logistics",
        &[
            "Global Logistics Partners",
            "Freight Solutions Inc.",
            "Supply Chain Services",
            "Distribution Networks LLC",
            "Warehouse Solutions Corp.",
            "Transportation Partners",
            "Shipping Dynamics Ltd.",
            "Fulfillment Services Inc.",
        ],
    ),
    (
        "Office",
        &[
            "Office Supplies Direct",
            "Business Products Inc.",
            "Stationery Solutions",
            "Equipment Suppliers Ltd.",
            "Furniture Systems Corp.",
            "Workplace Supplies LLC",
            "Office Essentials Inc.",
            "Business Equipment Co.",
        ],
    ),
    (
        "Utilities",
        &[
            "Power Solutions Inc.",
            "Energy Services Corp.",
            "Utility Management LLC",
            "Water Services Group",
            "Telecom Solutions Ltd.",
            "Communications Partners",
            "Internet Services Inc.",
            "Utility Systems Corp.",
        ],
    ),
];

/// Bank name templates.
const BANK_NAMES: &[&str] = &[
    "First National Bank",
    "Commerce Bank",
    "United Banking Corp",
    "Regional Trust Bank",
    "Merchants Bank",
    "Citizens Financial",
    "Pacific Coast Bank",
    "Atlantic Commerce Bank",
    "Midwest Trust Company",
    "Capital One Commercial",
];

/// Generator for vendor master data.
pub struct VendorGenerator {
    rng: ChaCha8Rng,
    seed: u64,
    config: VendorGeneratorConfig,
    vendor_counter: usize,
}

impl VendorGenerator {
    /// Create a new vendor generator.
    pub fn new(seed: u64) -> Self {
        Self::with_config(seed, VendorGeneratorConfig::default())
    }

    /// Create a new vendor generator with custom configuration.
    pub fn with_config(seed: u64, config: VendorGeneratorConfig) -> Self {
        Self {
            rng: ChaCha8Rng::seed_from_u64(seed),
            seed,
            config,
            vendor_counter: 0,
        }
    }

    /// Generate a single vendor.
    pub fn generate_vendor(&mut self, company_code: &str, effective_date: NaiveDate) -> Vendor {
        self.vendor_counter += 1;

        let vendor_id = format!("V-{:06}", self.vendor_counter);
        let (category, name) = self.select_vendor_name();
        let tax_id = self.generate_tax_id();

        let mut vendor = Vendor::new(&vendor_id, name, synth_core::models::VendorType::Supplier);
        vendor.tax_id = Some(tax_id);
        vendor.country = self.config.default_country.clone();
        vendor.currency = self.config.default_currency.clone();
        // Note: category, effective_date, address are not fields on Vendor

        // Set payment terms
        vendor.payment_terms = self.select_payment_terms();

        // Set behavior
        vendor.behavior = self.select_vendor_behavior();

        // Check if intercompany
        if self.rng.gen::<f64>() < self.config.intercompany_rate {
            vendor.is_intercompany = true;
            vendor.intercompany_code = Some(format!("IC-{}", company_code));
        }

        // Generate bank accounts
        if self.config.generate_bank_accounts {
            let bank_account = self.generate_bank_account(&vendor.vendor_id);
            vendor.bank_accounts.push(bank_account);

            if self.rng.gen::<f64>() < self.config.multiple_bank_account_rate {
                let bank_account2 = self.generate_bank_account(&vendor.vendor_id);
                vendor.bank_accounts.push(bank_account2);
            }
        }

        vendor
    }

    /// Generate an intercompany vendor (always intercompany).
    pub fn generate_intercompany_vendor(
        &mut self,
        company_code: &str,
        partner_company_code: &str,
        effective_date: NaiveDate,
    ) -> Vendor {
        let mut vendor = self.generate_vendor(company_code, effective_date);
        vendor.is_intercompany = true;
        vendor.intercompany_code = Some(partner_company_code.to_string());
        vendor.name = format!("{} - IC", partner_company_code);
        vendor.payment_terms = PaymentTerms::Immediate; // IC usually immediate
        vendor
    }

    /// Generate a vendor pool with specified count.
    pub fn generate_vendor_pool(
        &mut self,
        count: usize,
        company_code: &str,
        effective_date: NaiveDate,
    ) -> VendorPool {
        let mut pool = VendorPool::new();

        for _ in 0..count {
            let vendor = self.generate_vendor(company_code, effective_date);
            pool.add_vendor(vendor);
        }

        pool
    }

    /// Generate a vendor pool with intercompany vendors.
    pub fn generate_vendor_pool_with_ic(
        &mut self,
        count: usize,
        company_code: &str,
        partner_company_codes: &[String],
        effective_date: NaiveDate,
    ) -> VendorPool {
        let mut pool = VendorPool::new();

        // Generate regular vendors
        let regular_count = count.saturating_sub(partner_company_codes.len());
        for _ in 0..regular_count {
            let vendor = self.generate_vendor(company_code, effective_date);
            pool.add_vendor(vendor);
        }

        // Generate IC vendors for each partner
        for partner in partner_company_codes {
            let vendor = self.generate_intercompany_vendor(company_code, partner, effective_date);
            pool.add_vendor(vendor);
        }

        pool
    }

    /// Select a vendor name from templates.
    fn select_vendor_name(&mut self) -> (&'static str, &'static str) {
        let category_idx = self.rng.gen_range(0..VENDOR_NAME_TEMPLATES.len());
        let (category, names) = VENDOR_NAME_TEMPLATES[category_idx];
        let name_idx = self.rng.gen_range(0..names.len());
        (category, names[name_idx])
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

    /// Select vendor behavior based on distribution.
    fn select_vendor_behavior(&mut self) -> VendorBehavior {
        let roll: f64 = self.rng.gen();
        let mut cumulative = 0.0;

        for (behavior, prob) in &self.config.behavior_distribution {
            cumulative += prob;
            if roll < cumulative {
                return *behavior;
            }
        }

        VendorBehavior::Flexible
    }

    /// Generate a tax ID.
    fn generate_tax_id(&mut self) -> String {
        format!(
            "{:02}-{:07}",
            self.rng.gen_range(10..99),
            self.rng.gen_range(1000000..9999999)
        )
    }

    /// Generate a bank account.
    fn generate_bank_account(&mut self, vendor_id: &str) -> BankAccount {
        let bank_idx = self.rng.gen_range(0..BANK_NAMES.len());
        let bank_name = BANK_NAMES[bank_idx];

        let routing = format!("{:09}", self.rng.gen_range(100000000u64..999999999));
        let account = format!("{:010}", self.rng.gen_range(1000000000u64..9999999999));

        BankAccount {
            bank_name: bank_name.to_string(),
            bank_country: "US".to_string(),
            account_number: account,
            routing_code: routing,
            holder_name: format!("Vendor {}", vendor_id),
            is_primary: self.vendor_counter == 1,
        }
    }

    /// Generate an address.
    fn generate_address(&mut self) -> String {
        let street_num = self.rng.gen_range(100..9999);
        let streets = [
            "Main St",
            "Oak Ave",
            "Industrial Blvd",
            "Commerce Dr",
            "Business Park Way",
        ];
        let cities = [
            "Chicago",
            "Houston",
            "Phoenix",
            "Philadelphia",
            "San Antonio",
            "Dallas",
        ];
        let states = ["IL", "TX", "AZ", "PA", "TX", "TX"];

        let idx = self.rng.gen_range(0..streets.len());
        let zip = self.rng.gen_range(10000..99999);

        format!(
            "{} {}, {}, {} {}",
            street_num, streets[idx], cities[idx], states[idx], zip
        )
    }

    /// Reset the generator.
    pub fn reset(&mut self) {
        self.rng = ChaCha8Rng::seed_from_u64(self.seed);
        self.vendor_counter = 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vendor_generation() {
        let mut gen = VendorGenerator::new(42);
        let vendor = gen.generate_vendor("1000", NaiveDate::from_ymd_opt(2024, 1, 1).unwrap());

        assert!(!vendor.vendor_id.is_empty());
        assert!(!vendor.name.is_empty());
        assert!(vendor.tax_id.is_some());
        assert!(!vendor.bank_accounts.is_empty());
    }

    #[test]
    fn test_vendor_pool_generation() {
        let mut gen = VendorGenerator::new(42);
        let pool =
            gen.generate_vendor_pool(10, "1000", NaiveDate::from_ymd_opt(2024, 1, 1).unwrap());

        assert_eq!(pool.vendors.len(), 10);
    }

    #[test]
    fn test_intercompany_vendor() {
        let mut gen = VendorGenerator::new(42);
        let vendor = gen.generate_intercompany_vendor(
            "1000",
            "2000",
            NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
        );

        assert!(vendor.is_intercompany);
        assert_eq!(vendor.intercompany_code, Some("2000".to_string()));
    }

    #[test]
    fn test_deterministic_generation() {
        let mut gen1 = VendorGenerator::new(42);
        let mut gen2 = VendorGenerator::new(42);

        let vendor1 = gen1.generate_vendor("1000", NaiveDate::from_ymd_opt(2024, 1, 1).unwrap());
        let vendor2 = gen2.generate_vendor("1000", NaiveDate::from_ymd_opt(2024, 1, 1).unwrap());

        assert_eq!(vendor1.vendor_id, vendor2.vendor_id);
        assert_eq!(vendor1.name, vendor2.name);
    }

    #[test]
    fn test_vendor_pool_with_ic() {
        let mut gen = VendorGenerator::new(42);
        let pool = gen.generate_vendor_pool_with_ic(
            10,
            "1000",
            &["2000".to_string(), "3000".to_string()],
            NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
        );

        assert_eq!(pool.vendors.len(), 10);

        let ic_vendors: Vec<_> = pool.vendors.iter().filter(|v| v.is_intercompany).collect();
        assert_eq!(ic_vendors.len(), 2);
    }
}
