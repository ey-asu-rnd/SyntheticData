//! Intercompany transaction generator.
//!
//! Generates matched pairs of intercompany journal entries that offset
//! between related entities.

use chrono::{Datelike, NaiveDate, NaiveDateTime};
use rand::prelude::*;
use rand_chacha::ChaCha8Rng;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use std::collections::HashMap;

use synth_core::models::intercompany::{
    ConsolidationMethod, ICLoan, ICMatchedPair, ICSettlementStatus, ICTransactionType,
    IntercompanyRelationship, OwnershipStructure, RecurringFrequency, TransferPriceCalculation,
    TransferPricingMethod, TransferPricingPolicy,
};
use synth_core::models::{JournalEntry, JournalEntryLine};

/// Configuration for IC transaction generation.
#[derive(Debug, Clone)]
pub struct ICGeneratorConfig {
    /// Probability of generating an IC transaction (0.0 to 1.0).
    pub ic_transaction_rate: f64,
    /// Transfer pricing method to use.
    pub transfer_pricing_method: TransferPricingMethod,
    /// Markup percentage for cost-plus method.
    pub markup_percent: Decimal,
    /// Generate matched pairs (both sides of IC transaction).
    pub generate_matched_pairs: bool,
    /// Transaction type distribution.
    pub transaction_type_weights: HashMap<ICTransactionType, f64>,
    /// Generate netting settlements.
    pub generate_netting: bool,
    /// Netting frequency (if enabled).
    pub netting_frequency: RecurringFrequency,
    /// Generate IC loans.
    pub generate_loans: bool,
    /// Typical loan amount range.
    pub loan_amount_range: (Decimal, Decimal),
    /// Loan interest rate range.
    pub loan_interest_rate_range: (Decimal, Decimal),
}

impl Default for ICGeneratorConfig {
    fn default() -> Self {
        let mut weights = HashMap::new();
        weights.insert(ICTransactionType::GoodsSale, 0.35);
        weights.insert(ICTransactionType::ServiceProvided, 0.20);
        weights.insert(ICTransactionType::ManagementFee, 0.15);
        weights.insert(ICTransactionType::Royalty, 0.10);
        weights.insert(ICTransactionType::CostSharing, 0.10);
        weights.insert(ICTransactionType::LoanInterest, 0.05);
        weights.insert(ICTransactionType::ExpenseRecharge, 0.05);

        Self {
            ic_transaction_rate: 0.15,
            transfer_pricing_method: TransferPricingMethod::CostPlus,
            markup_percent: dec!(5),
            generate_matched_pairs: true,
            transaction_type_weights: weights,
            generate_netting: true,
            netting_frequency: RecurringFrequency::Monthly,
            generate_loans: true,
            loan_amount_range: (dec!(100000), dec!(10000000)),
            loan_interest_rate_range: (dec!(2), dec!(8)),
        }
    }
}

/// Generator for intercompany transactions.
pub struct ICGenerator {
    /// Configuration.
    config: ICGeneratorConfig,
    /// Random number generator.
    rng: ChaCha8Rng,
    /// Ownership structure.
    ownership_structure: OwnershipStructure,
    /// Transfer pricing policies by relationship.
    transfer_pricing_policies: HashMap<String, TransferPricingPolicy>,
    /// Active IC loans.
    active_loans: Vec<ICLoan>,
    /// Generated IC matched pairs.
    matched_pairs: Vec<ICMatchedPair>,
    /// IC reference counter.
    ic_counter: u64,
    /// Document counter.
    doc_counter: u64,
}

impl ICGenerator {
    /// Create a new IC generator.
    pub fn new(
        config: ICGeneratorConfig,
        ownership_structure: OwnershipStructure,
        seed: u64,
    ) -> Self {
        Self {
            config,
            rng: ChaCha8Rng::seed_from_u64(seed),
            ownership_structure,
            transfer_pricing_policies: HashMap::new(),
            active_loans: Vec::new(),
            matched_pairs: Vec::new(),
            ic_counter: 0,
            doc_counter: 0,
        }
    }

    /// Add a transfer pricing policy.
    pub fn add_transfer_pricing_policy(&mut self, relationship_id: String, policy: TransferPricingPolicy) {
        self.transfer_pricing_policies.insert(relationship_id, policy);
    }

    /// Generate IC reference number.
    fn generate_ic_reference(&mut self, date: NaiveDate) -> String {
        self.ic_counter += 1;
        format!("IC{}{:06}", date.format("%Y%m"), self.ic_counter)
    }

    /// Generate document number.
    fn generate_doc_number(&mut self, prefix: &str) -> String {
        self.doc_counter += 1;
        format!("{}{:08}", prefix, self.doc_counter)
    }

    /// Select a random IC transaction type based on weights.
    fn select_transaction_type(&mut self) -> ICTransactionType {
        let total_weight: f64 = self.config.transaction_type_weights.values().sum();
        let mut roll: f64 = self.rng.gen::<f64>() * total_weight;

        for (tx_type, weight) in &self.config.transaction_type_weights {
            roll -= weight;
            if roll <= 0.0 {
                return *tx_type;
            }
        }

        ICTransactionType::GoodsSale
    }

    /// Select a random pair of related companies.
    fn select_company_pair(&mut self) -> Option<(String, String)> {
        let relationships = self.ownership_structure.relationships.clone();
        if relationships.is_empty() {
            return None;
        }

        let rel = relationships.choose(&mut self.rng)?;

        // Randomly decide direction (parent sells to sub, or sub sells to parent)
        if self.rng.gen_bool(0.5) {
            Some((rel.parent_company.clone(), rel.subsidiary_company.clone()))
        } else {
            Some((rel.subsidiary_company.clone(), rel.parent_company.clone()))
        }
    }

    /// Generate a base amount for IC transaction.
    fn generate_base_amount(&mut self, tx_type: ICTransactionType) -> Decimal {
        let (min, max) = match tx_type {
            ICTransactionType::GoodsSale => (dec!(1000), dec!(500000)),
            ICTransactionType::ServiceProvided => (dec!(5000), dec!(200000)),
            ICTransactionType::ManagementFee => (dec!(10000), dec!(100000)),
            ICTransactionType::Royalty => (dec!(5000), dec!(150000)),
            ICTransactionType::CostSharing => (dec!(2000), dec!(50000)),
            ICTransactionType::LoanInterest => (dec!(1000), dec!(50000)),
            ICTransactionType::ExpenseRecharge => (dec!(500), dec!(20000)),
            ICTransactionType::Dividend => (dec!(50000), dec!(1000000)),
            _ => (dec!(1000), dec!(100000)),
        };

        let range = max - min;
        let random_factor = Decimal::from_f64_retain(self.rng.gen::<f64>()).unwrap_or(dec!(0.5));
        (min + range * random_factor).round_dp(2)
    }

    /// Apply transfer pricing markup to base amount.
    fn apply_transfer_pricing(&self, base_amount: Decimal, relationship_id: &str) -> Decimal {
        if let Some(policy) = self.transfer_pricing_policies.get(relationship_id) {
            policy.calculate_transfer_price(base_amount)
        } else {
            // Use default config markup
            base_amount * (Decimal::ONE + self.config.markup_percent / dec!(100))
        }
    }

    /// Generate a single IC matched pair.
    pub fn generate_ic_transaction(
        &mut self,
        date: NaiveDate,
        fiscal_period: &str,
    ) -> Option<ICMatchedPair> {
        // Check if we should generate an IC transaction
        if !self.rng.gen_bool(self.config.ic_transaction_rate) {
            return None;
        }

        let (seller, buyer) = self.select_company_pair()?;
        let tx_type = self.select_transaction_type();
        let base_amount = self.generate_base_amount(tx_type);

        // Find relationship for transfer pricing
        let relationship_id = format!("{}-{}", seller, buyer);
        let transfer_price = self.apply_transfer_pricing(base_amount, &relationship_id);

        let ic_reference = self.generate_ic_reference(date);

        let mut pair = ICMatchedPair::new(
            ic_reference,
            tx_type,
            seller.clone(),
            buyer.clone(),
            transfer_price,
            "USD".to_string(), // Could be parameterized
            date,
        );

        // Assign document numbers
        pair.seller_document = self.generate_doc_number("ICS");
        pair.buyer_document = self.generate_doc_number("ICB");

        // Calculate withholding tax if applicable
        if tx_type.has_withholding_tax() {
            pair.calculate_withholding_tax();
        }

        self.matched_pairs.push(pair.clone());
        Some(pair)
    }

    /// Generate IC journal entries from a matched pair.
    pub fn generate_journal_entries(
        &mut self,
        pair: &ICMatchedPair,
        fiscal_year: i32,
        fiscal_period: u32,
    ) -> (JournalEntry, JournalEntry) {
        let (seller_dr_desc, seller_cr_desc) = pair.transaction_type.seller_accounts();
        let (buyer_dr_desc, buyer_cr_desc) = pair.transaction_type.buyer_accounts();

        // Seller entry: DR IC Receivable, CR Revenue/Income
        let seller_entry = self.create_seller_entry(pair, fiscal_year, fiscal_period, seller_dr_desc, seller_cr_desc);

        // Buyer entry: DR Expense/Asset, CR IC Payable
        let buyer_entry = self.create_buyer_entry(pair, fiscal_year, fiscal_period, buyer_dr_desc, buyer_cr_desc);

        (seller_entry, buyer_entry)
    }

    /// Create seller-side journal entry.
    fn create_seller_entry(
        &mut self,
        pair: &ICMatchedPair,
        fiscal_year: i32,
        fiscal_period: u32,
        dr_desc: &str,
        cr_desc: &str,
    ) -> JournalEntry {
        let doc_number = pair.seller_document.clone();
        let posting_time = NaiveDateTime::new(
            pair.posting_date,
            chrono::NaiveTime::from_hms_opt(12, 0, 0).unwrap(),
        );

        let mut lines = vec![
            JournalEntryLine {
                line_number: 1,
                account_code: self.get_seller_receivable_account(&pair.buyer_company),
                account_description: Some(format!("IC Receivable - {}", pair.buyer_company)),
                debit_amount: Some(pair.amount),
                credit_amount: None,
                cost_center: None,
                profit_center: None,
                segment: None,
                project: None,
                customer: None,
                vendor: None,
                material: None,
                quantity: None,
                uom: None,
                text: Some(format!("{} - {}", dr_desc, pair.description)),
                assignment: Some(pair.ic_reference.clone()),
                reference: Some(pair.buyer_document.clone()),
                clearing_document: None,
                clearing_date: None,
            },
            JournalEntryLine {
                line_number: 2,
                account_code: self.get_seller_revenue_account(pair.transaction_type),
                account_description: Some(cr_desc.to_string()),
                debit_amount: None,
                credit_amount: Some(pair.amount),
                cost_center: None,
                profit_center: None,
                segment: None,
                project: None,
                customer: None,
                vendor: None,
                material: None,
                quantity: None,
                uom: None,
                text: Some(format!("{} - {}", cr_desc, pair.description)),
                assignment: Some(pair.ic_reference.clone()),
                reference: None,
                clearing_document: None,
                clearing_date: None,
            },
        ];

        // Add withholding tax line if applicable
        if let Some(wht) = pair.withholding_tax {
            lines.push(JournalEntryLine {
                line_number: 3,
                account_code: "2180".to_string(), // WHT payable
                account_description: Some("Withholding Tax Payable".to_string()),
                debit_amount: None,
                credit_amount: Some(wht),
                cost_center: None,
                profit_center: None,
                segment: None,
                project: None,
                customer: None,
                vendor: None,
                material: None,
                quantity: None,
                uom: None,
                text: Some("Withholding tax on IC transaction".to_string()),
                assignment: Some(pair.ic_reference.clone()),
                reference: None,
                clearing_document: None,
                clearing_date: None,
            });

            // Adjust receivable for net amount
            lines[0].debit_amount = Some(pair.net_amount());
        }

        JournalEntry {
            document_number: doc_number,
            company_code: pair.seller_company.clone(),
            fiscal_year,
            fiscal_period,
            document_type: "IC".to_string(),
            posting_date: pair.posting_date,
            document_date: pair.transaction_date,
            entry_date: pair.posting_date,
            posting_time,
            reference: Some(pair.ic_reference.clone()),
            header_text: Some(format!(
                "IC {} to {}",
                pair.transaction_type.seller_accounts().1,
                pair.buyer_company
            )),
            currency: pair.currency.clone(),
            exchange_rate: Some(Decimal::ONE),
            total_debit: pair.net_amount(),
            total_credit: pair.amount,
            line_count: lines.len() as u32,
            lines,
            created_by: "IC_GENERATOR".to_string(),
            created_at: chrono::Utc::now().naive_utc(),
            is_posted: true,
            is_reversed: false,
            reversal_document: None,
            reversal_reason: None,
            source_system: Some("IC".to_string()),
            business_process: Some("IC_TRANSACTION".to_string()),
            approval_status: Some("APPROVED".to_string()),
            approved_by: None,
            approved_at: None,
        }
    }

    /// Create buyer-side journal entry.
    fn create_buyer_entry(
        &mut self,
        pair: &ICMatchedPair,
        fiscal_year: i32,
        fiscal_period: u32,
        dr_desc: &str,
        cr_desc: &str,
    ) -> JournalEntry {
        let doc_number = pair.buyer_document.clone();
        let posting_time = NaiveDateTime::new(
            pair.posting_date,
            chrono::NaiveTime::from_hms_opt(12, 0, 0).unwrap(),
        );

        let lines = vec![
            JournalEntryLine {
                line_number: 1,
                account_code: self.get_buyer_expense_account(pair.transaction_type),
                account_description: Some(dr_desc.to_string()),
                debit_amount: Some(pair.amount),
                credit_amount: None,
                cost_center: Some("CC100".to_string()),
                profit_center: None,
                segment: None,
                project: None,
                customer: None,
                vendor: None,
                material: None,
                quantity: None,
                uom: None,
                text: Some(format!("{} - {}", dr_desc, pair.description)),
                assignment: Some(pair.ic_reference.clone()),
                reference: Some(pair.seller_document.clone()),
                clearing_document: None,
                clearing_date: None,
            },
            JournalEntryLine {
                line_number: 2,
                account_code: self.get_buyer_payable_account(&pair.seller_company),
                account_description: Some(format!("IC Payable - {}", pair.seller_company)),
                debit_amount: None,
                credit_amount: Some(pair.amount),
                cost_center: None,
                profit_center: None,
                segment: None,
                project: None,
                customer: None,
                vendor: None,
                material: None,
                quantity: None,
                uom: None,
                text: Some(format!("{} - {}", cr_desc, pair.description)),
                assignment: Some(pair.ic_reference.clone()),
                reference: None,
                clearing_document: None,
                clearing_date: None,
            },
        ];

        JournalEntry {
            document_number: doc_number,
            company_code: pair.buyer_company.clone(),
            fiscal_year,
            fiscal_period,
            document_type: "IC".to_string(),
            posting_date: pair.posting_date,
            document_date: pair.transaction_date,
            entry_date: pair.posting_date,
            posting_time,
            reference: Some(pair.ic_reference.clone()),
            header_text: Some(format!(
                "IC {} from {}",
                pair.transaction_type.buyer_accounts().0,
                pair.seller_company
            )),
            currency: pair.currency.clone(),
            exchange_rate: Some(Decimal::ONE),
            total_debit: pair.amount,
            total_credit: pair.amount,
            line_count: lines.len() as u32,
            lines,
            created_by: "IC_GENERATOR".to_string(),
            created_at: chrono::Utc::now().naive_utc(),
            is_posted: true,
            is_reversed: false,
            reversal_document: None,
            reversal_reason: None,
            source_system: Some("IC".to_string()),
            business_process: Some("IC_TRANSACTION".to_string()),
            approval_status: Some("APPROVED".to_string()),
            approved_by: None,
            approved_at: None,
        }
    }

    /// Get IC receivable account for seller.
    fn get_seller_receivable_account(&self, buyer_company: &str) -> String {
        format!("1310{}", &buyer_company[..buyer_company.len().min(2)])
    }

    /// Get IC revenue account for seller.
    fn get_seller_revenue_account(&self, tx_type: ICTransactionType) -> String {
        match tx_type {
            ICTransactionType::GoodsSale => "4100".to_string(),
            ICTransactionType::ServiceProvided => "4200".to_string(),
            ICTransactionType::ManagementFee => "4300".to_string(),
            ICTransactionType::Royalty => "4400".to_string(),
            ICTransactionType::LoanInterest => "4500".to_string(),
            ICTransactionType::Dividend => "4600".to_string(),
            _ => "4900".to_string(),
        }
    }

    /// Get IC expense account for buyer.
    fn get_buyer_expense_account(&self, tx_type: ICTransactionType) -> String {
        match tx_type {
            ICTransactionType::GoodsSale => "5100".to_string(),
            ICTransactionType::ServiceProvided => "5200".to_string(),
            ICTransactionType::ManagementFee => "5300".to_string(),
            ICTransactionType::Royalty => "5400".to_string(),
            ICTransactionType::LoanInterest => "5500".to_string(),
            ICTransactionType::Dividend => "3100".to_string(), // Retained earnings
            _ => "5900".to_string(),
        }
    }

    /// Get IC payable account for buyer.
    fn get_buyer_payable_account(&self, seller_company: &str) -> String {
        format!("2110{}", &seller_company[..seller_company.len().min(2)])
    }

    /// Generate an IC loan.
    pub fn generate_ic_loan(
        &mut self,
        lender: String,
        borrower: String,
        start_date: NaiveDate,
        term_months: u32,
    ) -> ICLoan {
        let (min_amount, max_amount) = self.config.loan_amount_range;
        let range = max_amount - min_amount;
        let random_factor = Decimal::from_f64_retain(self.rng.gen::<f64>()).unwrap_or(dec!(0.5));
        let principal = (min_amount + range * random_factor).round_dp(0);

        let (min_rate, max_rate) = self.config.loan_interest_rate_range;
        let rate_range = max_rate - min_rate;
        let rate_factor = Decimal::from_f64_retain(self.rng.gen::<f64>()).unwrap_or(dec!(0.5));
        let interest_rate = (min_rate + rate_range * rate_factor).round_dp(2);

        let maturity_date = start_date
            .checked_add_months(chrono::Months::new(term_months))
            .unwrap_or(start_date);

        let loan_id = format!("LOAN{}{:04}", start_date.format("%Y"), self.active_loans.len() + 1);

        let loan = ICLoan::new(
            loan_id,
            lender,
            borrower,
            principal,
            "USD".to_string(),
            interest_rate,
            start_date,
            maturity_date,
        );

        self.active_loans.push(loan.clone());
        loan
    }

    /// Generate interest entries for active loans.
    pub fn generate_loan_interest_entries(
        &mut self,
        as_of_date: NaiveDate,
        fiscal_year: i32,
        fiscal_period: u32,
    ) -> Vec<(JournalEntry, JournalEntry)> {
        let mut entries = Vec::new();

        for loan in &self.active_loans {
            if loan.is_repaid() {
                continue;
            }

            // Calculate interest for the period
            let period_start = NaiveDate::from_ymd_opt(
                if fiscal_period == 1 { fiscal_year - 1 } else { fiscal_year },
                if fiscal_period == 1 { 12 } else { fiscal_period - 1 },
                1,
            ).unwrap_or(as_of_date);

            let interest = loan.calculate_interest(period_start, as_of_date);

            if interest > Decimal::ZERO {
                let mut pair = ICMatchedPair::new(
                    self.generate_ic_reference(as_of_date),
                    ICTransactionType::LoanInterest,
                    loan.lender_company.clone(),
                    loan.borrower_company.clone(),
                    interest,
                    loan.currency.clone(),
                    as_of_date,
                );
                pair.seller_document = self.generate_doc_number("INT");
                pair.buyer_document = self.generate_doc_number("INT");
                pair.description = format!("Interest on loan {}", loan.loan_id);

                let (seller_je, buyer_je) = self.generate_journal_entries(&pair, fiscal_year, fiscal_period);
                entries.push((seller_je, buyer_je));
            }
        }

        entries
    }

    /// Get all generated matched pairs.
    pub fn get_matched_pairs(&self) -> &[ICMatchedPair] {
        &self.matched_pairs
    }

    /// Get open (unsettled) matched pairs.
    pub fn get_open_pairs(&self) -> Vec<&ICMatchedPair> {
        self.matched_pairs
            .iter()
            .filter(|p| p.is_open())
            .collect()
    }

    /// Get active loans.
    pub fn get_active_loans(&self) -> &[ICLoan] {
        &self.active_loans
    }

    /// Generate multiple IC transactions for a date range.
    pub fn generate_transactions_for_period(
        &mut self,
        start_date: NaiveDate,
        end_date: NaiveDate,
        transactions_per_day: usize,
    ) -> Vec<ICMatchedPair> {
        let mut pairs = Vec::new();
        let mut current_date = start_date;

        while current_date <= end_date {
            let fiscal_period = format!("{}{:02}", current_date.year(), current_date.month());

            for _ in 0..transactions_per_day {
                if let Some(pair) = self.generate_ic_transaction(current_date, &fiscal_period) {
                    pairs.push(pair);
                }
            }

            current_date = current_date.succ_opt().unwrap_or(current_date);
        }

        pairs
    }

    /// Reset counters (for testing).
    pub fn reset_counters(&mut self) {
        self.ic_counter = 0;
        self.doc_counter = 0;
        self.matched_pairs.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_ownership_structure() -> OwnershipStructure {
        let mut structure = OwnershipStructure::new("1000".to_string());
        structure.add_relationship(IntercompanyRelationship::new(
            "REL001".to_string(),
            "1000".to_string(),
            "1100".to_string(),
            dec!(100),
            NaiveDate::from_ymd_opt(2022, 1, 1).unwrap(),
        ));
        structure.add_relationship(IntercompanyRelationship::new(
            "REL002".to_string(),
            "1000".to_string(),
            "1200".to_string(),
            dec!(100),
            NaiveDate::from_ymd_opt(2022, 1, 1).unwrap(),
        ));
        structure
    }

    #[test]
    fn test_ic_generator_creation() {
        let config = ICGeneratorConfig::default();
        let structure = create_test_ownership_structure();
        let generator = ICGenerator::new(config, structure, 12345);

        assert!(generator.matched_pairs.is_empty());
        assert!(generator.active_loans.is_empty());
    }

    #[test]
    fn test_generate_ic_transaction() {
        let mut config = ICGeneratorConfig::default();
        config.ic_transaction_rate = 1.0; // Always generate

        let structure = create_test_ownership_structure();
        let mut generator = ICGenerator::new(config, structure, 12345);

        let date = NaiveDate::from_ymd_opt(2022, 6, 15).unwrap();
        let pair = generator.generate_ic_transaction(date, "202206");

        assert!(pair.is_some());
        let pair = pair.unwrap();
        assert!(!pair.ic_reference.is_empty());
        assert!(pair.amount > Decimal::ZERO);
    }

    #[test]
    fn test_generate_journal_entries() {
        let mut config = ICGeneratorConfig::default();
        config.ic_transaction_rate = 1.0;

        let structure = create_test_ownership_structure();
        let mut generator = ICGenerator::new(config, structure, 12345);

        let date = NaiveDate::from_ymd_opt(2022, 6, 15).unwrap();
        let pair = generator.generate_ic_transaction(date, "202206").unwrap();

        let (seller_je, buyer_je) = generator.generate_journal_entries(&pair, 2022, 6);

        assert_eq!(seller_je.company_code, pair.seller_company);
        assert_eq!(buyer_je.company_code, pair.buyer_company);
        assert_eq!(seller_je.reference, Some(pair.ic_reference.clone()));
        assert_eq!(buyer_je.reference, Some(pair.ic_reference));
    }

    #[test]
    fn test_generate_ic_loan() {
        let config = ICGeneratorConfig::default();
        let structure = create_test_ownership_structure();
        let mut generator = ICGenerator::new(config, structure, 12345);

        let loan = generator.generate_ic_loan(
            "1000".to_string(),
            "1100".to_string(),
            NaiveDate::from_ymd_opt(2022, 1, 1).unwrap(),
            24,
        );

        assert!(!loan.loan_id.is_empty());
        assert!(loan.principal > Decimal::ZERO);
        assert!(loan.interest_rate > Decimal::ZERO);
        assert_eq!(generator.active_loans.len(), 1);
    }

    #[test]
    fn test_generate_transactions_for_period() {
        let mut config = ICGeneratorConfig::default();
        config.ic_transaction_rate = 1.0;

        let structure = create_test_ownership_structure();
        let mut generator = ICGenerator::new(config, structure, 12345);

        let start = NaiveDate::from_ymd_opt(2022, 6, 1).unwrap();
        let end = NaiveDate::from_ymd_opt(2022, 6, 5).unwrap();

        let pairs = generator.generate_transactions_for_period(start, end, 2);

        // 5 days * 2 transactions per day = 10 transactions
        assert_eq!(pairs.len(), 10);
    }
}
