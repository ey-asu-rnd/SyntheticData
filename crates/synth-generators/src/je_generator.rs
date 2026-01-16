//! Journal Entry generator with statistical distributions.

use chrono::{Datelike, NaiveDate};
use rand::prelude::*;
use rand_chacha::ChaCha8Rng;
use std::sync::Arc;
use uuid::Uuid;

use synth_config::schema::{FraudConfig, GeneratorConfig, TemplateConfig, TransactionConfig};
use synth_core::distributions::*;
use synth_core::models::*;
use synth_core::templates::{
    descriptions::DescriptionContext, DescriptionGenerator, ReferenceGenerator, ReferenceType,
};
use synth_core::traits::Generator;

use crate::company_selector::WeightedCompanySelector;
use crate::user_generator::{UserGenerator, UserGeneratorConfig};

/// Generator for realistic journal entries.
pub struct JournalEntryGenerator {
    rng: ChaCha8Rng,
    seed: u64,
    config: TransactionConfig,
    coa: Arc<ChartOfAccounts>,
    companies: Vec<String>,
    company_selector: WeightedCompanySelector,
    line_sampler: LineItemSampler,
    amount_sampler: AmountSampler,
    temporal_sampler: TemporalSampler,
    start_date: NaiveDate,
    end_date: NaiveDate,
    count: u64,
    doc_counter: u64,
    // Enhanced features
    user_pool: Option<UserPool>,
    description_generator: DescriptionGenerator,
    reference_generator: ReferenceGenerator,
    template_config: TemplateConfig,
    vendor_pool: VendorPool,
    customer_pool: CustomerPool,
    // Material pool for realistic material references
    material_pool: Option<MaterialPool>,
    // Flag indicating whether we're using real master data vs defaults
    using_real_master_data: bool,
    // Fraud generation
    fraud_config: FraudConfig,
    // Persona-based error injection
    persona_errors_enabled: bool,
}

impl JournalEntryGenerator {
    /// Create a new journal entry generator.
    pub fn new_with_params(
        config: TransactionConfig,
        coa: Arc<ChartOfAccounts>,
        companies: Vec<String>,
        start_date: NaiveDate,
        end_date: NaiveDate,
        seed: u64,
    ) -> Self {
        Self::new_with_full_config(
            config,
            coa,
            companies,
            start_date,
            end_date,
            seed,
            TemplateConfig::default(),
            None,
        )
    }

    /// Create a new journal entry generator with full configuration.
    #[allow(clippy::too_many_arguments)]
    pub fn new_with_full_config(
        config: TransactionConfig,
        coa: Arc<ChartOfAccounts>,
        companies: Vec<String>,
        start_date: NaiveDate,
        end_date: NaiveDate,
        seed: u64,
        template_config: TemplateConfig,
        user_pool: Option<UserPool>,
    ) -> Self {
        // Initialize user pool if not provided
        let user_pool = user_pool.or_else(|| {
            if template_config.names.generate_realistic_names {
                let user_gen_config = UserGeneratorConfig {
                    culture_distribution: vec![
                        (
                            synth_core::templates::NameCulture::WesternUs,
                            template_config.names.culture_distribution.western_us,
                        ),
                        (
                            synth_core::templates::NameCulture::Hispanic,
                            template_config.names.culture_distribution.hispanic,
                        ),
                        (
                            synth_core::templates::NameCulture::German,
                            template_config.names.culture_distribution.german,
                        ),
                        (
                            synth_core::templates::NameCulture::French,
                            template_config.names.culture_distribution.french,
                        ),
                        (
                            synth_core::templates::NameCulture::Chinese,
                            template_config.names.culture_distribution.chinese,
                        ),
                        (
                            synth_core::templates::NameCulture::Japanese,
                            template_config.names.culture_distribution.japanese,
                        ),
                        (
                            synth_core::templates::NameCulture::Indian,
                            template_config.names.culture_distribution.indian,
                        ),
                    ],
                    email_domain: template_config.names.email_domain.clone(),
                    generate_realistic_names: true,
                };
                let mut user_gen = UserGenerator::with_config(seed + 100, user_gen_config);
                Some(user_gen.generate_standard(&companies))
            } else {
                None
            }
        });

        // Initialize reference generator
        let mut ref_gen = ReferenceGenerator::new(
            start_date.year(),
            companies.first().map(|s| s.as_str()).unwrap_or("1000"),
        );
        ref_gen.set_prefix(
            ReferenceType::Invoice,
            &template_config.references.invoice_prefix,
        );
        ref_gen.set_prefix(
            ReferenceType::PurchaseOrder,
            &template_config.references.po_prefix,
        );
        ref_gen.set_prefix(
            ReferenceType::SalesOrder,
            &template_config.references.so_prefix,
        );

        // Create weighted company selector (uniform weights for this constructor)
        let company_selector = WeightedCompanySelector::uniform(companies.clone());

        Self {
            rng: ChaCha8Rng::seed_from_u64(seed),
            seed,
            config: config.clone(),
            coa,
            companies,
            company_selector,
            line_sampler: LineItemSampler::with_config(
                seed + 1,
                config.line_item_distribution.clone(),
                config.even_odd_distribution.clone(),
                config.debit_credit_distribution.clone(),
            ),
            amount_sampler: AmountSampler::with_config(seed + 2, config.amounts.clone()),
            temporal_sampler: TemporalSampler::with_config(
                seed + 3,
                config.seasonality.clone(),
                WorkingHoursConfig::default(),
                Vec::new(),
            ),
            start_date,
            end_date,
            count: 0,
            doc_counter: 0,
            user_pool,
            description_generator: DescriptionGenerator::new(),
            reference_generator: ref_gen,
            template_config,
            vendor_pool: VendorPool::standard(),
            customer_pool: CustomerPool::standard(),
            material_pool: None,
            using_real_master_data: false,
            fraud_config: FraudConfig::default(),
            persona_errors_enabled: true, // Enable by default for realism
        }
    }

    /// Create from a full GeneratorConfig.
    ///
    /// This constructor uses the volume_weight from company configs
    /// for weighted company selection, and fraud config from GeneratorConfig.
    pub fn from_generator_config(
        full_config: &GeneratorConfig,
        coa: Arc<ChartOfAccounts>,
        start_date: NaiveDate,
        end_date: NaiveDate,
        seed: u64,
    ) -> Self {
        let companies: Vec<String> = full_config
            .companies
            .iter()
            .map(|c| c.code.clone())
            .collect();

        // Create weighted selector using volume_weight from company configs
        let company_selector = WeightedCompanySelector::from_configs(&full_config.companies);

        let mut generator = Self::new_with_full_config(
            full_config.transactions.clone(),
            coa,
            companies,
            start_date,
            end_date,
            seed,
            full_config.templates.clone(),
            None,
        );

        // Override the uniform selector with weighted selector
        generator.company_selector = company_selector;

        // Set fraud config
        generator.fraud_config = full_config.fraud.clone();

        generator
    }

    /// Set a custom company selector.
    pub fn set_company_selector(&mut self, selector: WeightedCompanySelector) {
        self.company_selector = selector;
    }

    /// Get the current company selector.
    pub fn company_selector(&self) -> &WeightedCompanySelector {
        &self.company_selector
    }

    /// Set fraud configuration.
    pub fn set_fraud_config(&mut self, config: FraudConfig) {
        self.fraud_config = config;
    }

    /// Set vendors from generated master data.
    ///
    /// This replaces the default vendor pool with actual generated vendors,
    /// ensuring JEs reference real master data entities.
    pub fn with_vendors(mut self, vendors: &[Vendor]) -> Self {
        if !vendors.is_empty() {
            self.vendor_pool = VendorPool::from_vendors(vendors.to_vec());
            self.using_real_master_data = true;
        }
        self
    }

    /// Set customers from generated master data.
    ///
    /// This replaces the default customer pool with actual generated customers,
    /// ensuring JEs reference real master data entities.
    pub fn with_customers(mut self, customers: &[Customer]) -> Self {
        if !customers.is_empty() {
            self.customer_pool = CustomerPool::from_customers(customers.to_vec());
            self.using_real_master_data = true;
        }
        self
    }

    /// Set materials from generated master data.
    ///
    /// This provides material references for JEs that involve inventory movements.
    pub fn with_materials(mut self, materials: &[Material]) -> Self {
        if !materials.is_empty() {
            self.material_pool = Some(MaterialPool::from_materials(materials.to_vec()));
            self.using_real_master_data = true;
        }
        self
    }

    /// Set all master data at once for convenience.
    ///
    /// This is the recommended way to configure the JE generator with
    /// generated master data to ensure data coherence.
    pub fn with_master_data(
        self,
        vendors: &[Vendor],
        customers: &[Customer],
        materials: &[Material],
    ) -> Self {
        self.with_vendors(vendors)
            .with_customers(customers)
            .with_materials(materials)
    }

    /// Check if the generator is using real master data.
    pub fn is_using_real_master_data(&self) -> bool {
        self.using_real_master_data
    }

    /// Determine if this transaction should be fraudulent.
    fn determine_fraud(&mut self) -> Option<FraudType> {
        if !self.fraud_config.enabled {
            return None;
        }

        // Roll for fraud based on fraud rate
        if self.rng.gen::<f64>() >= self.fraud_config.fraud_rate {
            return None;
        }

        // Select fraud type based on distribution
        Some(self.select_fraud_type())
    }

    /// Select a fraud type based on the configured distribution.
    fn select_fraud_type(&mut self) -> FraudType {
        let dist = &self.fraud_config.fraud_type_distribution;
        let roll: f64 = self.rng.gen();

        let mut cumulative = 0.0;

        cumulative += dist.suspense_account_abuse;
        if roll < cumulative {
            return FraudType::SuspenseAccountAbuse;
        }

        cumulative += dist.fictitious_transaction;
        if roll < cumulative {
            return FraudType::FictitiousTransaction;
        }

        cumulative += dist.revenue_manipulation;
        if roll < cumulative {
            return FraudType::RevenueManipulation;
        }

        cumulative += dist.expense_capitalization;
        if roll < cumulative {
            return FraudType::ExpenseCapitalization;
        }

        cumulative += dist.split_transaction;
        if roll < cumulative {
            return FraudType::SplitTransaction;
        }

        cumulative += dist.timing_anomaly;
        if roll < cumulative {
            return FraudType::TimingAnomaly;
        }

        cumulative += dist.unauthorized_access;
        if roll < cumulative {
            return FraudType::UnauthorizedAccess;
        }

        // Default fallback
        FraudType::DuplicatePayment
    }

    /// Map a fraud type to an amount pattern for suspicious amounts.
    fn fraud_type_to_amount_pattern(&self, fraud_type: FraudType) -> FraudAmountPattern {
        match fraud_type {
            FraudType::SplitTransaction | FraudType::JustBelowThreshold => {
                FraudAmountPattern::ThresholdAdjacent
            }
            FraudType::FictitiousTransaction
            | FraudType::FictitiousEntry
            | FraudType::SuspenseAccountAbuse
            | FraudType::RoundDollarManipulation => FraudAmountPattern::ObviousRoundNumbers,
            FraudType::RevenueManipulation
            | FraudType::ExpenseCapitalization
            | FraudType::ImproperCapitalization
            | FraudType::ReserveManipulation
            | FraudType::UnauthorizedAccess
            | FraudType::PrematureRevenue
            | FraudType::UnderstatedLiabilities
            | FraudType::OverstatedAssets
            | FraudType::ChannelStuffing => FraudAmountPattern::StatisticallyImprobable,
            FraudType::DuplicatePayment
            | FraudType::TimingAnomaly
            | FraudType::SelfApproval
            | FraudType::ExceededApprovalLimit
            | FraudType::SegregationOfDutiesViolation
            | FraudType::UnauthorizedApproval
            | FraudType::CollusiveApproval
            | FraudType::FictitiousVendor
            | FraudType::ShellCompanyPayment
            | FraudType::Kickback
            | FraudType::KickbackScheme
            | FraudType::InvoiceManipulation
            | FraudType::AssetMisappropriation
            | FraudType::InventoryTheft
            | FraudType::GhostEmployee => FraudAmountPattern::Normal,
        }
    }

    /// Generate a deterministic UUID from seed and counter.
    fn generate_deterministic_uuid(&self) -> Uuid {
        // Create a deterministic UUID v4-format from seed and counter
        // using the seed and counter to generate the UUID bytes
        let mut bytes = [0u8; 16];
        let seed_bytes = self.seed.to_le_bytes();
        let counter_bytes = self.doc_counter.to_le_bytes();

        // Mix seed and counter into the UUID bytes
        bytes[0..8].copy_from_slice(&seed_bytes);
        bytes[8..16].copy_from_slice(&counter_bytes);

        // Set version to 4 and variant bits properly
        bytes[6] = (bytes[6] & 0x0f) | 0x40; // Version 4
        bytes[8] = (bytes[8] & 0x3f) | 0x80; // Variant 1

        Uuid::from_bytes(bytes)
    }

    /// Generate a single journal entry.
    pub fn generate(&mut self) -> JournalEntry {
        self.count += 1;
        self.doc_counter += 1;

        // Generate deterministic document ID
        let document_id = self.generate_deterministic_uuid();

        // Sample posting date
        let posting_date = self
            .temporal_sampler
            .sample_date(self.start_date, self.end_date);

        // Select company using weighted selector
        let company_code = self.company_selector.select(&mut self.rng).to_string();

        // Sample line item specification
        let line_spec = self.line_sampler.sample();

        // Determine source type and business process
        let is_automated = self.rng.gen::<f64>() < self.config.source_distribution.automated;
        let source = if is_automated {
            TransactionSource::Automated
        } else {
            TransactionSource::Manual
        };

        // Select business process
        let business_process = self.select_business_process();

        // Determine if this is a fraudulent transaction
        let fraud_type = self.determine_fraud();
        let is_fraud = fraud_type.is_some();

        // Sample time based on source
        let time = self.temporal_sampler.sample_time(!is_automated);
        let created_at = posting_date.and_time(time).and_utc();

        // Select user from pool or generate generic
        let (created_by, user_persona) = self.select_user(is_automated);

        // Create header with deterministic UUID
        let mut header =
            JournalEntryHeader::with_deterministic_id(company_code, posting_date, document_id);
        header.created_at = created_at;
        header.source = source;
        header.created_by = created_by;
        header.user_persona = user_persona;
        header.business_process = Some(business_process);
        header.is_fraud = is_fraud;
        header.fraud_type = fraud_type;

        // Generate description context
        let mut context =
            DescriptionContext::with_period(posting_date.month(), posting_date.year());

        // Add vendor/customer context based on business process
        match business_process {
            BusinessProcess::P2P => {
                if let Some(vendor) = self.vendor_pool.random_vendor(&mut self.rng) {
                    context.vendor_name = Some(vendor.name.clone());
                }
            }
            BusinessProcess::O2C => {
                if let Some(customer) = self.customer_pool.random_customer(&mut self.rng) {
                    context.customer_name = Some(customer.name.clone());
                }
            }
            _ => {}
        }

        // Generate header text if enabled
        if self.template_config.descriptions.generate_header_text {
            header.header_text = Some(self.description_generator.generate_header_text(
                business_process,
                &context,
                &mut self.rng,
            ));
        }

        // Generate reference if enabled
        if self.template_config.references.generate_references {
            header.reference = Some(
                self.reference_generator
                    .generate_for_process_year(business_process, posting_date.year()),
            );
        }

        // Generate line items
        let mut entry = JournalEntry::new(header);

        // Generate amount - use fraud pattern if this is a fraudulent transaction
        let total_amount = if let Some(ft) = fraud_type {
            let pattern = self.fraud_type_to_amount_pattern(ft);
            self.amount_sampler.sample_fraud(pattern)
        } else {
            self.amount_sampler.sample()
        };

        // Generate debit lines
        let debit_amounts = self
            .amount_sampler
            .sample_summing_to(line_spec.debit_count, total_amount);
        for (i, amount) in debit_amounts.into_iter().enumerate() {
            let account_number = self.select_debit_account().account_number.clone();
            let mut line = JournalEntryLine::debit(
                entry.header.document_id,
                (i + 1) as u32,
                account_number.clone(),
                amount,
            );

            // Generate line text if enabled
            if self.template_config.descriptions.generate_line_text {
                line.line_text = Some(self.description_generator.generate_line_text(
                    &account_number,
                    &context,
                    &mut self.rng,
                ));
            }

            entry.add_line(line);
        }

        // Generate credit lines - use the SAME amounts to ensure balance
        let credit_amounts = self
            .amount_sampler
            .sample_summing_to(line_spec.credit_count, total_amount);
        for (i, amount) in credit_amounts.into_iter().enumerate() {
            let account_number = self.select_credit_account().account_number.clone();
            let mut line = JournalEntryLine::credit(
                entry.header.document_id,
                (line_spec.debit_count + i + 1) as u32,
                account_number.clone(),
                amount,
            );

            // Generate line text if enabled
            if self.template_config.descriptions.generate_line_text {
                line.line_text = Some(self.description_generator.generate_line_text(
                    &account_number,
                    &context,
                    &mut self.rng,
                ));
            }

            entry.add_line(line);
        }

        // Apply persona-based errors if enabled and it's a human user
        if self.persona_errors_enabled && !is_automated {
            self.maybe_inject_persona_error(&mut entry);
        }

        entry
    }

    /// Enable or disable persona-based error injection.
    ///
    /// When enabled, entries created by human personas have a chance
    /// to contain realistic human errors based on their experience level.
    pub fn with_persona_errors(mut self, enabled: bool) -> Self {
        self.persona_errors_enabled = enabled;
        self
    }

    /// Check if persona errors are enabled.
    pub fn persona_errors_enabled(&self) -> bool {
        self.persona_errors_enabled
    }

    /// Maybe inject a persona-appropriate error based on the persona's error rate.
    fn maybe_inject_persona_error(&mut self, entry: &mut JournalEntry) {
        // Parse persona from the entry header
        let persona_str = &entry.header.user_persona;
        let persona = match persona_str.to_lowercase().as_str() {
            s if s.contains("junior") => UserPersona::JuniorAccountant,
            s if s.contains("senior") => UserPersona::SeniorAccountant,
            s if s.contains("controller") => UserPersona::Controller,
            s if s.contains("manager") => UserPersona::Manager,
            s if s.contains("executive") => UserPersona::Executive,
            _ => return, // Don't inject errors for unknown personas
        };

        // Check if error should occur based on persona's error rate
        let error_rate = persona.error_rate();
        if self.rng.gen::<f64>() >= error_rate {
            return; // No error this time
        }

        // Select and inject persona-appropriate error
        self.inject_human_error(entry, persona);
    }

    /// Inject a human-like error based on the persona.
    ///
    /// Error types 0, 1, and 3 modify amounts and create unbalanced entries.
    /// These entries are marked with [HUMAN_ERROR:*] tags in header_text for ML detection.
    /// Error types 2 and 4 don't affect amounts and entries remain balanced.
    fn inject_human_error(&mut self, entry: &mut JournalEntry, persona: UserPersona) {
        use rust_decimal::Decimal;

        // Different personas make different types of errors
        let error_type: u8 = match persona {
            UserPersona::JuniorAccountant => {
                // Junior accountants make more varied errors
                self.rng.gen_range(0..5)
            }
            UserPersona::SeniorAccountant => {
                // Senior accountants mainly make transposition errors
                self.rng.gen_range(0..3)
            }
            UserPersona::Controller | UserPersona::Manager => {
                // Controllers/managers mainly make rounding or cutoff errors
                self.rng.gen_range(3..5)
            }
            _ => return,
        };

        match error_type {
            0 => {
                // Transposed digits in an amount
                if let Some(line) = entry.lines.get_mut(0) {
                    let amount = if line.debit_amount > Decimal::ZERO {
                        &mut line.debit_amount
                    } else {
                        &mut line.credit_amount
                    };

                    // Simple digit swap in the string representation
                    let s = amount.to_string();
                    if s.len() >= 2 {
                        let chars: Vec<char> = s.chars().collect();
                        let pos = self.rng.gen_range(0..chars.len().saturating_sub(1));
                        if chars[pos].is_ascii_digit() && chars.get(pos + 1).map_or(false, |c| c.is_ascii_digit()) {
                            let mut new_chars = chars;
                            new_chars.swap(pos, pos + 1);
                            if let Ok(new_amount) = new_chars.into_iter().collect::<String>().parse::<Decimal>() {
                                *amount = new_amount;
                                entry.header.header_text = Some(
                                    entry.header.header_text.clone().unwrap_or_default()
                                        + " [HUMAN_ERROR:TRANSPOSITION]"
                                );
                            }
                        }
                    }
                }
            }
            1 => {
                // Wrong decimal place (off by factor of 10)
                if let Some(line) = entry.lines.get_mut(0) {
                    if line.debit_amount > Decimal::ZERO {
                        line.debit_amount = line.debit_amount * Decimal::new(10, 0);
                    } else if line.credit_amount > Decimal::ZERO {
                        line.credit_amount = line.credit_amount * Decimal::new(10, 0);
                    }
                    entry.header.header_text = Some(
                        entry.header.header_text.clone().unwrap_or_default()
                            + " [HUMAN_ERROR:DECIMAL_SHIFT]"
                    );
                }
            }
            2 => {
                // Typo in description (doesn't affect balance)
                if let Some(ref mut text) = entry.header.header_text {
                    let typos = ["teh", "adn", "wiht", "taht", "recieve"];
                    let correct = ["the", "and", "with", "that", "receive"];
                    let idx = self.rng.gen_range(0..typos.len());
                    if text.to_lowercase().contains(correct[idx]) {
                        *text = text.replace(correct[idx], typos[idx]);
                        *text = format!("{} [HUMAN_ERROR:TYPO]", text);
                    }
                }
            }
            3 => {
                // Rounding to round number
                if let Some(line) = entry.lines.get_mut(0) {
                    if line.debit_amount > Decimal::ZERO {
                        line.debit_amount = (line.debit_amount / Decimal::new(100, 0)).round()
                            * Decimal::new(100, 0);
                    } else if line.credit_amount > Decimal::ZERO {
                        line.credit_amount = (line.credit_amount / Decimal::new(100, 0)).round()
                            * Decimal::new(100, 0);
                    }
                    entry.header.header_text = Some(
                        entry.header.header_text.clone().unwrap_or_default()
                            + " [HUMAN_ERROR:ROUNDED]"
                    );
                }
            }
            4 => {
                // Late posting marker (document date much earlier than posting date)
                // This doesn't create an imbalance
                if entry.header.document_date == entry.header.posting_date {
                    let days_late = self.rng.gen_range(5..15);
                    entry.header.document_date =
                        entry.header.posting_date - chrono::Duration::days(days_late);
                    entry.header.header_text = Some(
                        entry.header.header_text.clone().unwrap_or_default()
                            + " [HUMAN_ERROR:LATE_POSTING]"
                    );
                }
            }
            _ => {}
        }
    }

    /// Select a user from the pool or generate a generic user ID.
    fn select_user(&mut self, is_automated: bool) -> (String, String) {
        if let Some(ref pool) = self.user_pool {
            let persona = if is_automated {
                UserPersona::AutomatedSystem
            } else {
                // Random distribution among human personas
                let roll: f64 = self.rng.gen();
                if roll < 0.4 {
                    UserPersona::JuniorAccountant
                } else if roll < 0.7 {
                    UserPersona::SeniorAccountant
                } else if roll < 0.85 {
                    UserPersona::Controller
                } else {
                    UserPersona::Manager
                }
            };

            if let Some(user) = pool.get_random_user(persona, &mut self.rng) {
                return (
                    user.user_id.clone(),
                    format!("{:?}", user.persona).to_lowercase(),
                );
            }
        }

        // Fallback to generic format
        if is_automated {
            (
                format!("BATCH{:04}", self.rng.gen_range(1..=20)),
                "automated_system".to_string(),
            )
        } else {
            (
                format!("USER{:04}", self.rng.gen_range(1..=40)),
                "senior_accountant".to_string(),
            )
        }
    }

    /// Select a business process based on configuration weights.
    fn select_business_process(&mut self) -> BusinessProcess {
        let roll: f64 = self.rng.gen();

        // Default weights: O2C=35%, P2P=30%, R2R=20%, H2R=10%, A2R=5%
        if roll < 0.35 {
            BusinessProcess::O2C
        } else if roll < 0.65 {
            BusinessProcess::P2P
        } else if roll < 0.85 {
            BusinessProcess::R2R
        } else if roll < 0.95 {
            BusinessProcess::H2R
        } else {
            BusinessProcess::A2R
        }
    }

    fn select_debit_account(&mut self) -> &GLAccount {
        let accounts = self.coa.get_accounts_by_type(AccountType::Asset);
        let expense_accounts = self.coa.get_accounts_by_type(AccountType::Expense);

        // 60% asset, 40% expense for debits
        let all: Vec<_> = if self.rng.gen::<f64>() < 0.6 {
            accounts
        } else {
            expense_accounts
        };

        all.choose(&mut self.rng)
            .copied()
            .unwrap_or_else(|| &self.coa.accounts[0])
    }

    fn select_credit_account(&mut self) -> &GLAccount {
        let liability_accounts = self.coa.get_accounts_by_type(AccountType::Liability);
        let revenue_accounts = self.coa.get_accounts_by_type(AccountType::Revenue);

        // 60% liability, 40% revenue for credits
        let all: Vec<_> = if self.rng.gen::<f64>() < 0.6 {
            liability_accounts
        } else {
            revenue_accounts
        };

        all.choose(&mut self.rng)
            .copied()
            .unwrap_or_else(|| &self.coa.accounts[0])
    }
}

impl Generator for JournalEntryGenerator {
    type Item = JournalEntry;
    type Config = (
        TransactionConfig,
        Arc<ChartOfAccounts>,
        Vec<String>,
        NaiveDate,
        NaiveDate,
    );

    fn new(config: Self::Config, seed: u64) -> Self {
        Self::new_with_params(config.0, config.1, config.2, config.3, config.4, seed)
    }

    fn generate_one(&mut self) -> Self::Item {
        self.generate()
    }

    fn reset(&mut self) {
        self.rng = ChaCha8Rng::seed_from_u64(self.seed);
        self.line_sampler.reset(self.seed + 1);
        self.amount_sampler.reset(self.seed + 2);
        self.temporal_sampler.reset(self.seed + 3);
        self.count = 0;
        self.doc_counter = 0;

        // Reset reference generator by recreating it
        let mut ref_gen = ReferenceGenerator::new(
            self.start_date.year(),
            self.companies.first().map(|s| s.as_str()).unwrap_or("1000"),
        );
        ref_gen.set_prefix(
            ReferenceType::Invoice,
            &self.template_config.references.invoice_prefix,
        );
        ref_gen.set_prefix(
            ReferenceType::PurchaseOrder,
            &self.template_config.references.po_prefix,
        );
        ref_gen.set_prefix(
            ReferenceType::SalesOrder,
            &self.template_config.references.so_prefix,
        );
        self.reference_generator = ref_gen;
    }

    fn count(&self) -> u64 {
        self.count
    }

    fn seed(&self) -> u64 {
        self.seed
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ChartOfAccountsGenerator;

    #[test]
    fn test_generate_balanced_entries() {
        let mut coa_gen =
            ChartOfAccountsGenerator::new(CoAComplexity::Small, IndustrySector::Manufacturing, 42);
        let coa = Arc::new(coa_gen.generate());

        let mut je_gen = JournalEntryGenerator::new_with_params(
            TransactionConfig::default(),
            coa,
            vec!["1000".to_string()],
            NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            NaiveDate::from_ymd_opt(2024, 12, 31).unwrap(),
            42,
        );

        for _ in 0..100 {
            let entry = je_gen.generate();
            assert!(
                entry.is_balanced(),
                "Entry {:?} is not balanced",
                entry.header.document_id
            );
            assert!(entry.line_count() >= 2, "Entry has fewer than 2 lines");
        }
    }

    #[test]
    fn test_deterministic_generation() {
        let mut coa_gen =
            ChartOfAccountsGenerator::new(CoAComplexity::Small, IndustrySector::Manufacturing, 42);
        let coa = Arc::new(coa_gen.generate());

        let mut gen1 = JournalEntryGenerator::new_with_params(
            TransactionConfig::default(),
            Arc::clone(&coa),
            vec!["1000".to_string()],
            NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            NaiveDate::from_ymd_opt(2024, 12, 31).unwrap(),
            42,
        );

        let mut gen2 = JournalEntryGenerator::new_with_params(
            TransactionConfig::default(),
            coa,
            vec!["1000".to_string()],
            NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            NaiveDate::from_ymd_opt(2024, 12, 31).unwrap(),
            42,
        );

        for _ in 0..50 {
            let e1 = gen1.generate();
            let e2 = gen2.generate();
            assert_eq!(e1.header.document_id, e2.header.document_id);
            assert_eq!(e1.total_debit(), e2.total_debit());
        }
    }

    #[test]
    fn test_templates_generate_descriptions() {
        let mut coa_gen =
            ChartOfAccountsGenerator::new(CoAComplexity::Small, IndustrySector::Manufacturing, 42);
        let coa = Arc::new(coa_gen.generate());

        // Enable all template features
        let template_config = TemplateConfig {
            names: synth_config::schema::NameTemplateConfig {
                generate_realistic_names: true,
                email_domain: "test.com".to_string(),
                culture_distribution: synth_config::schema::CultureDistribution::default(),
            },
            descriptions: synth_config::schema::DescriptionTemplateConfig {
                generate_header_text: true,
                generate_line_text: true,
            },
            references: synth_config::schema::ReferenceTemplateConfig {
                generate_references: true,
                invoice_prefix: "TEST-INV".to_string(),
                po_prefix: "TEST-PO".to_string(),
                so_prefix: "TEST-SO".to_string(),
            },
        };

        let mut je_gen = JournalEntryGenerator::new_with_full_config(
            TransactionConfig::default(),
            coa,
            vec!["1000".to_string()],
            NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            NaiveDate::from_ymd_opt(2024, 12, 31).unwrap(),
            42,
            template_config,
            None,
        );

        for _ in 0..10 {
            let entry = je_gen.generate();

            // Verify header text is populated
            assert!(
                entry.header.header_text.is_some(),
                "Header text should be populated"
            );

            // Verify reference is populated
            assert!(
                entry.header.reference.is_some(),
                "Reference should be populated"
            );

            // Verify business process is set
            assert!(
                entry.header.business_process.is_some(),
                "Business process should be set"
            );

            // Verify line text is populated
            for line in &entry.lines {
                assert!(line.line_text.is_some(), "Line text should be populated");
            }

            // Entry should still be balanced
            assert!(entry.is_balanced());
        }
    }

    #[test]
    fn test_user_pool_integration() {
        let mut coa_gen =
            ChartOfAccountsGenerator::new(CoAComplexity::Small, IndustrySector::Manufacturing, 42);
        let coa = Arc::new(coa_gen.generate());

        let companies = vec!["1000".to_string()];

        // Generate user pool
        let mut user_gen = crate::UserGenerator::new(42);
        let user_pool = user_gen.generate_standard(&companies);

        let mut je_gen = JournalEntryGenerator::new_with_full_config(
            TransactionConfig::default(),
            coa,
            companies,
            NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            NaiveDate::from_ymd_opt(2024, 12, 31).unwrap(),
            42,
            TemplateConfig::default(),
            Some(user_pool),
        );

        // Generate entries and verify user IDs are from pool
        for _ in 0..20 {
            let entry = je_gen.generate();

            // User ID should not be generic BATCH/USER format when pool is used
            // (though it may still fall back if random selection misses)
            assert!(!entry.header.created_by.is_empty());
        }
    }

    #[test]
    fn test_master_data_connection() {
        let mut coa_gen =
            ChartOfAccountsGenerator::new(CoAComplexity::Small, IndustrySector::Manufacturing, 42);
        let coa = Arc::new(coa_gen.generate());

        // Create test vendors
        let vendors = vec![
            Vendor::new("V-TEST-001", "Test Vendor Alpha", VendorType::Supplier),
            Vendor::new("V-TEST-002", "Test Vendor Beta", VendorType::Technology),
        ];

        // Create test customers
        let customers = vec![
            Customer::new("C-TEST-001", "Test Customer One", CustomerType::Corporate),
            Customer::new("C-TEST-002", "Test Customer Two", CustomerType::SmallBusiness),
        ];

        // Create test materials
        let materials = vec![
            Material::new("MAT-TEST-001", "Test Material A", MaterialType::RawMaterial),
        ];

        // Create generator with master data
        let generator = JournalEntryGenerator::new_with_params(
            TransactionConfig::default(),
            coa,
            vec!["1000".to_string()],
            NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            NaiveDate::from_ymd_opt(2024, 12, 31).unwrap(),
            42,
        );

        // Without master data
        assert!(!generator.is_using_real_master_data());

        // Connect master data
        let generator_with_data = generator
            .with_vendors(&vendors)
            .with_customers(&customers)
            .with_materials(&materials);

        // Should now be using real master data
        assert!(generator_with_data.is_using_real_master_data());
    }

    #[test]
    fn test_with_master_data_convenience_method() {
        let mut coa_gen =
            ChartOfAccountsGenerator::new(CoAComplexity::Small, IndustrySector::Manufacturing, 42);
        let coa = Arc::new(coa_gen.generate());

        let vendors = vec![
            Vendor::new("V-001", "Vendor One", VendorType::Supplier),
        ];
        let customers = vec![
            Customer::new("C-001", "Customer One", CustomerType::Corporate),
        ];
        let materials = vec![
            Material::new("MAT-001", "Material One", MaterialType::RawMaterial),
        ];

        let generator = JournalEntryGenerator::new_with_params(
            TransactionConfig::default(),
            coa,
            vec!["1000".to_string()],
            NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            NaiveDate::from_ymd_opt(2024, 12, 31).unwrap(),
            42,
        )
        .with_master_data(&vendors, &customers, &materials);

        assert!(generator.is_using_real_master_data());
    }
}
