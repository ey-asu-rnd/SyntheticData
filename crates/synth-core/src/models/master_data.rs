//! Master data models for vendors and customers.
//!
//! Provides realistic vendor and customer entities for transaction
//! attribution and header/line text generation.

use rand::seq::SliceRandom;
use rand::Rng;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Type of vendor relationship.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum VendorType {
    /// General supplier of goods
    #[default]
    Supplier,
    /// Service provider
    ServiceProvider,
    /// Utility company
    Utility,
    /// Professional services (legal, accounting, consulting)
    ProfessionalServices,
    /// Technology/software vendor
    Technology,
    /// Logistics/shipping
    Logistics,
    /// Contractor/freelancer
    Contractor,
    /// Landlord/property management
    RealEstate,
    /// Financial services
    Financial,
    /// Employee expense reimbursement
    EmployeeReimbursement,
}

impl VendorType {
    /// Get typical expense categories for this vendor type.
    pub fn typical_expense_categories(&self) -> &'static [&'static str] {
        match self {
            Self::Supplier => &["Materials", "Inventory", "Office Supplies", "Equipment"],
            Self::ServiceProvider => &["Services", "Maintenance", "Support"],
            Self::Utility => &["Electricity", "Gas", "Water", "Telecommunications"],
            Self::ProfessionalServices => &["Legal", "Audit", "Consulting", "Tax Services"],
            Self::Technology => &["Software", "Licenses", "Cloud Services", "IT Support"],
            Self::Logistics => &["Freight", "Shipping", "Warehousing", "Customs"],
            Self::Contractor => &["Contract Labor", "Professional Fees", "Consulting"],
            Self::RealEstate => &["Rent", "Property Management", "Facilities"],
            Self::Financial => &["Bank Fees", "Interest", "Insurance", "Financing Costs"],
            Self::EmployeeReimbursement => &["Travel", "Meals", "Entertainment", "Expenses"],
        }
    }
}

/// Vendor master data.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Vendor {
    /// Vendor ID (e.g., "V-001234")
    pub vendor_id: String,

    /// Vendor name
    pub name: String,

    /// Type of vendor
    pub vendor_type: VendorType,

    /// Country code (ISO 3166-1 alpha-2)
    pub country: String,

    /// Payment terms in days
    pub payment_terms_days: u8,

    /// Typical invoice amount range (min, max)
    pub typical_amount_range: (Decimal, Decimal),

    /// Is this vendor active
    pub is_active: bool,

    /// Vendor account number in sub-ledger
    pub account_number: Option<String>,

    /// Tax ID / VAT number
    pub tax_id: Option<String>,
}

impl Vendor {
    /// Create a new vendor.
    pub fn new(vendor_id: &str, name: &str, vendor_type: VendorType) -> Self {
        Self {
            vendor_id: vendor_id.to_string(),
            name: name.to_string(),
            vendor_type,
            country: "US".to_string(),
            payment_terms_days: 30,
            typical_amount_range: (Decimal::from(100), Decimal::from(10000)),
            is_active: true,
            account_number: None,
            tax_id: None,
        }
    }

    /// Set country.
    pub fn with_country(mut self, country: &str) -> Self {
        self.country = country.to_string();
        self
    }

    /// Set payment terms.
    pub fn with_payment_terms(mut self, days: u8) -> Self {
        self.payment_terms_days = days;
        self
    }

    /// Set amount range.
    pub fn with_amount_range(mut self, min: Decimal, max: Decimal) -> Self {
        self.typical_amount_range = (min, max);
        self
    }

    /// Generate a random amount within the typical range.
    pub fn generate_amount(&self, rng: &mut impl Rng) -> Decimal {
        let (min, max) = self.typical_amount_range;
        let range = max - min;
        let random_fraction = Decimal::from_f64_retain(rng.gen::<f64>()).unwrap_or(Decimal::ZERO);
        min + range * random_fraction
    }
}

/// Type of customer relationship.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum CustomerType {
    /// Business-to-business customer
    #[default]
    Corporate,
    /// Small/medium business
    SmallBusiness,
    /// Individual consumer
    Consumer,
    /// Government entity
    Government,
    /// Non-profit organization
    NonProfit,
    /// Intercompany (related party)
    Intercompany,
    /// Distributor/reseller
    Distributor,
}

/// Customer master data.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Customer {
    /// Customer ID (e.g., "C-001234")
    pub customer_id: String,

    /// Customer name
    pub name: String,

    /// Type of customer
    pub customer_type: CustomerType,

    /// Country code (ISO 3166-1 alpha-2)
    pub country: String,

    /// Credit limit
    pub credit_limit: Decimal,

    /// Payment terms in days
    pub payment_terms_days: u8,

    /// Is this customer active
    pub is_active: bool,

    /// Customer account number in sub-ledger
    pub account_number: Option<String>,

    /// Typical order amount range (min, max)
    pub typical_order_range: (Decimal, Decimal),
}

impl Customer {
    /// Create a new customer.
    pub fn new(customer_id: &str, name: &str, customer_type: CustomerType) -> Self {
        Self {
            customer_id: customer_id.to_string(),
            name: name.to_string(),
            customer_type,
            country: "US".to_string(),
            credit_limit: Decimal::from(100000),
            payment_terms_days: 30,
            is_active: true,
            account_number: None,
            typical_order_range: (Decimal::from(500), Decimal::from(50000)),
        }
    }

    /// Set country.
    pub fn with_country(mut self, country: &str) -> Self {
        self.country = country.to_string();
        self
    }

    /// Set credit limit.
    pub fn with_credit_limit(mut self, limit: Decimal) -> Self {
        self.credit_limit = limit;
        self
    }

    /// Set payment terms.
    pub fn with_payment_terms(mut self, days: u8) -> Self {
        self.payment_terms_days = days;
        self
    }

    /// Generate a random order amount within typical range.
    pub fn generate_order_amount(&self, rng: &mut impl Rng) -> Decimal {
        let (min, max) = self.typical_order_range;
        let range = max - min;
        let random_fraction = Decimal::from_f64_retain(rng.gen::<f64>()).unwrap_or(Decimal::ZERO);
        min + range * random_fraction
    }
}

/// Pool of vendors for transaction generation.
#[derive(Debug, Clone, Default)]
pub struct VendorPool {
    /// All vendors
    pub vendors: Vec<Vendor>,
    /// Index by vendor type
    type_index: HashMap<VendorType, Vec<usize>>,
}

impl VendorPool {
    /// Create a new empty vendor pool.
    pub fn new() -> Self {
        Self {
            vendors: Vec::new(),
            type_index: HashMap::new(),
        }
    }

    /// Add a vendor to the pool.
    pub fn add_vendor(&mut self, vendor: Vendor) {
        let idx = self.vendors.len();
        let vendor_type = vendor.vendor_type;
        self.vendors.push(vendor);
        self.type_index.entry(vendor_type).or_default().push(idx);
    }

    /// Get a random vendor.
    pub fn random_vendor(&self, rng: &mut impl Rng) -> Option<&Vendor> {
        self.vendors.choose(rng)
    }

    /// Get a random vendor of a specific type.
    pub fn random_vendor_of_type(
        &self,
        vendor_type: VendorType,
        rng: &mut impl Rng,
    ) -> Option<&Vendor> {
        self.type_index
            .get(&vendor_type)
            .and_then(|indices| indices.choose(rng))
            .map(|&idx| &self.vendors[idx])
    }

    /// Rebuild the type index (call after deserialization).
    pub fn rebuild_index(&mut self) {
        self.type_index.clear();
        for (idx, vendor) in self.vendors.iter().enumerate() {
            self.type_index
                .entry(vendor.vendor_type)
                .or_default()
                .push(idx);
        }
    }

    /// Generate a standard vendor pool with realistic vendors.
    pub fn standard() -> Self {
        let mut pool = Self::new();

        // Suppliers
        let suppliers = [
            ("V-000001", "Acme Supplies Inc", VendorType::Supplier),
            ("V-000002", "Global Materials Corp", VendorType::Supplier),
            ("V-000003", "Office Depot Business", VendorType::Supplier),
            ("V-000004", "Industrial Parts Co", VendorType::Supplier),
            ("V-000005", "Premium Components Ltd", VendorType::Supplier),
        ];

        // Service providers
        let services = [
            ("V-000010", "CleanCo Services", VendorType::ServiceProvider),
            (
                "V-000011",
                "Building Maintenance Inc",
                VendorType::ServiceProvider,
            ),
            (
                "V-000012",
                "Security Solutions LLC",
                VendorType::ServiceProvider,
            ),
        ];

        // Utilities
        let utilities = [
            ("V-000020", "City Electric Utility", VendorType::Utility),
            ("V-000021", "Natural Gas Co", VendorType::Utility),
            ("V-000022", "Metro Water Authority", VendorType::Utility),
            ("V-000023", "Telecom Network Inc", VendorType::Utility),
        ];

        // Professional services
        let professional = [
            (
                "V-000030",
                "Baker & Associates LLP",
                VendorType::ProfessionalServices,
            ),
            (
                "V-000031",
                "PricewaterhouseCoopers",
                VendorType::ProfessionalServices,
            ),
            (
                "V-000032",
                "McKinsey & Company",
                VendorType::ProfessionalServices,
            ),
            (
                "V-000033",
                "Deloitte Consulting",
                VendorType::ProfessionalServices,
            ),
        ];

        // Technology
        let technology = [
            ("V-000040", "Microsoft Corporation", VendorType::Technology),
            ("V-000041", "Amazon Web Services", VendorType::Technology),
            ("V-000042", "Salesforce Inc", VendorType::Technology),
            ("V-000043", "SAP America Inc", VendorType::Technology),
            ("V-000044", "Oracle Corporation", VendorType::Technology),
            ("V-000045", "Adobe Systems", VendorType::Technology),
        ];

        // Logistics
        let logistics = [
            ("V-000050", "FedEx Corporation", VendorType::Logistics),
            ("V-000051", "UPS Shipping", VendorType::Logistics),
            ("V-000052", "DHL Express", VendorType::Logistics),
        ];

        // Real estate
        let real_estate = [
            (
                "V-000060",
                "Commercial Properties LLC",
                VendorType::RealEstate,
            ),
            ("V-000061", "CBRE Group", VendorType::RealEstate),
        ];

        // Add all vendors
        for (id, name, vtype) in suppliers {
            pool.add_vendor(
                Vendor::new(id, name, vtype)
                    .with_amount_range(Decimal::from(500), Decimal::from(50000)),
            );
        }

        for (id, name, vtype) in services {
            pool.add_vendor(
                Vendor::new(id, name, vtype)
                    .with_amount_range(Decimal::from(200), Decimal::from(5000)),
            );
        }

        for (id, name, vtype) in utilities {
            pool.add_vendor(
                Vendor::new(id, name, vtype)
                    .with_amount_range(Decimal::from(500), Decimal::from(20000)),
            );
        }

        for (id, name, vtype) in professional {
            pool.add_vendor(
                Vendor::new(id, name, vtype)
                    .with_amount_range(Decimal::from(5000), Decimal::from(500000)),
            );
        }

        for (id, name, vtype) in technology {
            pool.add_vendor(
                Vendor::new(id, name, vtype)
                    .with_amount_range(Decimal::from(100), Decimal::from(100000)),
            );
        }

        for (id, name, vtype) in logistics {
            pool.add_vendor(
                Vendor::new(id, name, vtype)
                    .with_amount_range(Decimal::from(50), Decimal::from(10000)),
            );
        }

        for (id, name, vtype) in real_estate {
            pool.add_vendor(
                Vendor::new(id, name, vtype)
                    .with_amount_range(Decimal::from(5000), Decimal::from(100000)),
            );
        }

        pool
    }
}

/// Pool of customers for transaction generation.
#[derive(Debug, Clone, Default)]
pub struct CustomerPool {
    /// All customers
    pub customers: Vec<Customer>,
    /// Index by customer type
    type_index: HashMap<CustomerType, Vec<usize>>,
}

impl CustomerPool {
    /// Create a new empty customer pool.
    pub fn new() -> Self {
        Self {
            customers: Vec::new(),
            type_index: HashMap::new(),
        }
    }

    /// Add a customer to the pool.
    pub fn add_customer(&mut self, customer: Customer) {
        let idx = self.customers.len();
        let customer_type = customer.customer_type;
        self.customers.push(customer);
        self.type_index.entry(customer_type).or_default().push(idx);
    }

    /// Get a random customer.
    pub fn random_customer(&self, rng: &mut impl Rng) -> Option<&Customer> {
        self.customers.choose(rng)
    }

    /// Get a random customer of a specific type.
    pub fn random_customer_of_type(
        &self,
        customer_type: CustomerType,
        rng: &mut impl Rng,
    ) -> Option<&Customer> {
        self.type_index
            .get(&customer_type)
            .and_then(|indices| indices.choose(rng))
            .map(|&idx| &self.customers[idx])
    }

    /// Rebuild the type index.
    pub fn rebuild_index(&mut self) {
        self.type_index.clear();
        for (idx, customer) in self.customers.iter().enumerate() {
            self.type_index
                .entry(customer.customer_type)
                .or_default()
                .push(idx);
        }
    }

    /// Generate a standard customer pool.
    pub fn standard() -> Self {
        let mut pool = Self::new();

        // Corporate customers
        let corporate = [
            ("C-000001", "Northwind Traders", CustomerType::Corporate),
            ("C-000002", "Contoso Corporation", CustomerType::Corporate),
            ("C-000003", "Adventure Works", CustomerType::Corporate),
            ("C-000004", "Fabrikam Industries", CustomerType::Corporate),
            ("C-000005", "Wide World Importers", CustomerType::Corporate),
            ("C-000006", "Tailspin Toys", CustomerType::Corporate),
            ("C-000007", "Proseware Inc", CustomerType::Corporate),
            ("C-000008", "Coho Vineyard", CustomerType::Corporate),
            ("C-000009", "Alpine Ski House", CustomerType::Corporate),
            ("C-000010", "VanArsdel Ltd", CustomerType::Corporate),
        ];

        // Small business
        let small_business = [
            ("C-000020", "Smith & Co LLC", CustomerType::SmallBusiness),
            (
                "C-000021",
                "Johnson Enterprises",
                CustomerType::SmallBusiness,
            ),
            (
                "C-000022",
                "Williams Consulting",
                CustomerType::SmallBusiness,
            ),
            (
                "C-000023",
                "Brown Brothers Shop",
                CustomerType::SmallBusiness,
            ),
            (
                "C-000024",
                "Davis Family Business",
                CustomerType::SmallBusiness,
            ),
        ];

        // Government
        let government = [
            (
                "C-000030",
                "US Federal Government",
                CustomerType::Government,
            ),
            ("C-000031", "State of California", CustomerType::Government),
            ("C-000032", "City of New York", CustomerType::Government),
        ];

        // Distributors
        let distributors = [
            (
                "C-000040",
                "National Distribution Co",
                CustomerType::Distributor,
            ),
            (
                "C-000041",
                "Regional Wholesale Inc",
                CustomerType::Distributor,
            ),
            (
                "C-000042",
                "Pacific Distributors",
                CustomerType::Distributor,
            ),
        ];

        for (id, name, ctype) in corporate {
            pool.add_customer(
                Customer::new(id, name, ctype).with_credit_limit(Decimal::from(500000)),
            );
        }

        for (id, name, ctype) in small_business {
            pool.add_customer(
                Customer::new(id, name, ctype).with_credit_limit(Decimal::from(50000)),
            );
        }

        for (id, name, ctype) in government {
            pool.add_customer(
                Customer::new(id, name, ctype)
                    .with_credit_limit(Decimal::from(1000000))
                    .with_payment_terms(45),
            );
        }

        for (id, name, ctype) in distributors {
            pool.add_customer(
                Customer::new(id, name, ctype).with_credit_limit(Decimal::from(250000)),
            );
        }

        pool
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::SeedableRng;
    use rand_chacha::ChaCha8Rng;

    #[test]
    fn test_vendor_creation() {
        let vendor = Vendor::new("V-001", "Test Vendor", VendorType::Supplier)
            .with_country("DE")
            .with_payment_terms(45);

        assert_eq!(vendor.vendor_id, "V-001");
        assert_eq!(vendor.country, "DE");
        assert_eq!(vendor.payment_terms_days, 45);
    }

    #[test]
    fn test_vendor_pool() {
        let pool = VendorPool::standard();

        assert!(!pool.vendors.is_empty());

        let mut rng = ChaCha8Rng::seed_from_u64(42);
        let vendor = pool.random_vendor(&mut rng);
        assert!(vendor.is_some());

        let tech_vendor = pool.random_vendor_of_type(VendorType::Technology, &mut rng);
        assert!(tech_vendor.is_some());
    }

    #[test]
    fn test_customer_pool() {
        let pool = CustomerPool::standard();

        assert!(!pool.customers.is_empty());

        let mut rng = ChaCha8Rng::seed_from_u64(42);
        let customer = pool.random_customer(&mut rng);
        assert!(customer.is_some());
    }

    #[test]
    fn test_amount_generation() {
        let mut rng = ChaCha8Rng::seed_from_u64(42);
        let vendor = Vendor::new("V-001", "Test", VendorType::Supplier)
            .with_amount_range(Decimal::from(100), Decimal::from(1000));

        let amount = vendor.generate_amount(&mut rng);
        assert!(amount >= Decimal::from(100));
        assert!(amount <= Decimal::from(1000));
    }
}
