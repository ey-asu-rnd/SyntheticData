//! AP (Accounts Payable) generator.

use chrono::NaiveDate;
use rand::Rng;
use rand_chacha::ChaCha8Rng;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;

use synth_core::models::subledger::ap::{
    APDebitMemo, APDebitMemoLine, APInvoice, APInvoiceLine, APPayment, APPaymentMethod,
    DebitMemoReason, MatchStatus,
};
use synth_core::models::subledger::PaymentTerms;
use synth_core::models::{JournalEntry, JournalEntryLine};

/// Configuration for AP generation.
#[derive(Debug, Clone)]
pub struct APGeneratorConfig {
    /// Average invoice amount.
    pub avg_invoice_amount: Decimal,
    /// Invoice amount variation.
    pub amount_variation: Decimal,
    /// Percentage of invoices paid on time.
    pub on_time_payment_rate: Decimal,
    /// Average days to payment.
    pub avg_days_to_payment: u32,
    /// Debit memo rate.
    pub debit_memo_rate: Decimal,
    /// Default tax rate.
    pub tax_rate: Decimal,
    /// Three-way match rate.
    pub three_way_match_rate: Decimal,
    /// Default payment terms.
    pub default_terms: PaymentTerms,
}

impl Default for APGeneratorConfig {
    fn default() -> Self {
        Self {
            avg_invoice_amount: dec!(10000),
            amount_variation: dec!(0.6),
            avg_days_to_payment: 30,
            on_time_payment_rate: dec!(0.85),
            debit_memo_rate: dec!(0.03),
            tax_rate: dec!(10),
            three_way_match_rate: dec!(0.95),
            default_terms: PaymentTerms::net_30(),
        }
    }
}

/// Generator for AP transactions.
pub struct APGenerator {
    config: APGeneratorConfig,
    rng: ChaCha8Rng,
    invoice_counter: u64,
    payment_counter: u64,
    debit_memo_counter: u64,
}

impl APGenerator {
    /// Creates a new AP generator.
    pub fn new(config: APGeneratorConfig, rng: ChaCha8Rng) -> Self {
        Self {
            config,
            rng,
            invoice_counter: 0,
            payment_counter: 0,
            debit_memo_counter: 0,
        }
    }

    /// Generates an AP invoice.
    pub fn generate_invoice(
        &mut self,
        company_code: &str,
        vendor_id: &str,
        vendor_name: &str,
        vendor_invoice_number: &str,
        invoice_date: NaiveDate,
        currency: &str,
        line_count: usize,
        po_number: Option<&str>,
    ) -> (APInvoice, JournalEntry) {
        self.invoice_counter += 1;
        let invoice_number = format!("APINV{:08}", self.invoice_counter);

        let mut invoice = APInvoice::new(
            invoice_number.clone(),
            vendor_invoice_number.to_string(),
            company_code.to_string(),
            vendor_id.to_string(),
            vendor_name.to_string(),
            invoice_date,
            self.config.default_terms.clone(),
            currency.to_string(),
        );

        if let Some(po) = po_number {
            invoice.reference_po = Some(po.to_string());
            invoice.match_status = if self.rng.gen::<f64>() < 0.95 {
                MatchStatus::Matched
            } else {
                MatchStatus::MatchedWithVariance {
                    price_variance: self.generate_variance(),
                    quantity_variance: Decimal::ZERO,
                }
            };
        } else {
            invoice.match_status = MatchStatus::NotRequired;
        }

        for line_num in 1..=line_count {
            let amount = self.generate_line_amount();
            let line = APInvoiceLine::new(
                line_num as u32,
                format!("Item/Service {}", line_num),
                dec!(1),
                "EA".to_string(),
                amount,
                "5000".to_string(),
            )
            .with_tax("VAT".to_string(), self.config.tax_rate);

            invoice.add_line(line);
        }

        let je = self.generate_invoice_je(&invoice);
        (invoice, je)
    }

    /// Generates a payment.
    pub fn generate_payment(
        &mut self,
        invoices: &[&APInvoice],
        payment_date: NaiveDate,
        house_bank: &str,
        bank_account: &str,
    ) -> (APPayment, JournalEntry) {
        self.payment_counter += 1;
        let payment_number = format!("APPAY{:08}", self.payment_counter);

        let vendor = invoices.first().expect("At least one invoice required");
        let total_amount: Decimal = invoices.iter().map(|i| i.amount_remaining).sum();
        let total_discount: Decimal = invoices
            .iter()
            .map(|i| i.available_discount(payment_date))
            .sum();

        let mut payment = APPayment::new(
            payment_number.clone(),
            vendor.company_code.clone(),
            vendor.vendor_id.clone(),
            vendor.vendor_name.clone(),
            payment_date,
            total_amount - total_discount,
            vendor.gross_amount.document_currency.clone(),
            self.random_payment_method(),
            house_bank.to_string(),
            bank_account.to_string(),
        );

        for invoice in invoices {
            let discount = invoice.available_discount(payment_date);
            payment.allocate_to_invoice(
                invoice.invoice_number.clone(),
                invoice.amount_remaining,
                discount,
                Decimal::ZERO,
            );
        }

        let je = self.generate_payment_je(&payment);
        (payment, je)
    }

    /// Generates a debit memo.
    pub fn generate_debit_memo(
        &mut self,
        invoice: &APInvoice,
        memo_date: NaiveDate,
        reason: DebitMemoReason,
        percent: Decimal,
    ) -> (APDebitMemo, JournalEntry) {
        self.debit_memo_counter += 1;
        let memo_number = format!("APDM{:08}", self.debit_memo_counter);

        let mut memo = APDebitMemo::for_invoice(
            memo_number.clone(),
            invoice.company_code.clone(),
            invoice.vendor_id.clone(),
            invoice.vendor_name.clone(),
            memo_date,
            invoice.invoice_number.clone(),
            reason,
            format!("{:?}", reason),
            invoice.gross_amount.document_currency.clone(),
        );

        for (idx, inv_line) in invoice.lines.iter().enumerate() {
            let line = APDebitMemoLine::new(
                (idx + 1) as u32,
                inv_line.description.clone(),
                inv_line.quantity * percent,
                inv_line.unit.clone(),
                inv_line.unit_price,
                inv_line.gl_account.clone(),
            )
            .with_tax(
                inv_line.tax_code.clone().unwrap_or_default(),
                inv_line.tax_rate,
            );
            memo.add_line(line);
        }

        let je = self.generate_debit_memo_je(&memo);
        (memo, je)
    }

    fn generate_line_amount(&mut self) -> Decimal {
        let base = self.config.avg_invoice_amount;
        let variation = base * self.config.amount_variation;
        let random: f64 = self.rng.gen_range(-1.0..1.0);
        (base + variation * Decimal::try_from(random).unwrap_or_default())
            .max(dec!(100))
            .round_dp(2)
    }

    fn generate_variance(&mut self) -> Decimal {
        let random: f64 = self.rng.gen_range(-100.0..100.0);
        Decimal::try_from(random).unwrap_or_default().round_dp(2)
    }

    fn random_payment_method(&mut self) -> APPaymentMethod {
        match self.rng.gen_range(0..4) {
            0 => APPaymentMethod::WireTransfer,
            1 => APPaymentMethod::Check,
            2 => APPaymentMethod::ACH,
            _ => APPaymentMethod::SEPA,
        }
    }

    fn generate_invoice_je(&self, invoice: &APInvoice) -> JournalEntry {
        let mut je = JournalEntry::new(
            format!("JE-{}", invoice.invoice_number),
            invoice.company_code.clone(),
            invoice.posting_date,
            format!("AP Invoice {}", invoice.invoice_number),
        );

        // Debit Expense
        je.add_line(JournalEntryLine {
            line_number: 1,
            account_code: "5000".to_string(),
            account_description: Some("Expense".to_string()),
            debit_amount: invoice.net_amount.document_amount,
            credit_amount: Decimal::ZERO,
            cost_center: None,
            profit_center: None,
            project_code: None,
            reference: Some(invoice.invoice_number.clone()),
            assignment: None,
            text: None,
            quantity: None,
            unit: None,
            tax_code: None,
            trading_partner: None,
            value_date: None,
        });

        // Debit Tax Receivable
        if invoice.tax_amount.document_amount > Decimal::ZERO {
            je.add_line(JournalEntryLine {
                line_number: 2,
                account_code: "1400".to_string(),
                account_description: Some("Input Tax".to_string()),
                debit_amount: invoice.tax_amount.document_amount,
                credit_amount: Decimal::ZERO,
                cost_center: None,
                profit_center: None,
                project_code: None,
                reference: Some(invoice.invoice_number.clone()),
                assignment: None,
                text: None,
                quantity: None,
                unit: None,
                tax_code: Some("VAT".to_string()),
                trading_partner: None,
                value_date: None,
            });
        }

        // Credit AP
        je.add_line(JournalEntryLine {
            line_number: 3,
            account_code: "2000".to_string(),
            account_description: Some("Accounts Payable".to_string()),
            debit_amount: Decimal::ZERO,
            credit_amount: invoice.gross_amount.document_amount,
            cost_center: None,
            profit_center: None,
            project_code: None,
            reference: Some(invoice.invoice_number.clone()),
            assignment: Some(invoice.vendor_id.clone()),
            text: None,
            quantity: None,
            unit: None,
            tax_code: None,
            trading_partner: None,
            value_date: None,
        });

        je
    }

    fn generate_payment_je(&self, payment: &APPayment) -> JournalEntry {
        let mut je = JournalEntry::new(
            format!("JE-{}", payment.payment_number),
            payment.company_code.clone(),
            payment.posting_date,
            format!("AP Payment {}", payment.payment_number),
        );

        // Debit AP
        let ap_debit = payment.net_payment + payment.discount_taken;
        je.add_line(JournalEntryLine {
            line_number: 1,
            account_code: "2000".to_string(),
            account_description: Some("Accounts Payable".to_string()),
            debit_amount: ap_debit,
            credit_amount: Decimal::ZERO,
            cost_center: None,
            profit_center: None,
            project_code: None,
            reference: Some(payment.payment_number.clone()),
            assignment: Some(payment.vendor_id.clone()),
            text: None,
            quantity: None,
            unit: None,
            tax_code: None,
            trading_partner: None,
            value_date: None,
        });

        // Credit Cash
        je.add_line(JournalEntryLine {
            line_number: 2,
            account_code: "1000".to_string(),
            account_description: Some("Cash".to_string()),
            debit_amount: Decimal::ZERO,
            credit_amount: payment.net_payment,
            cost_center: None,
            profit_center: None,
            project_code: None,
            reference: Some(payment.payment_number.clone()),
            assignment: None,
            text: None,
            quantity: None,
            unit: None,
            tax_code: None,
            trading_partner: None,
            value_date: None,
        });

        // Credit Discount Income
        if payment.discount_taken > Decimal::ZERO {
            je.add_line(JournalEntryLine {
                line_number: 3,
                account_code: "4800".to_string(),
                account_description: Some("Purchase Discounts".to_string()),
                debit_amount: Decimal::ZERO,
                credit_amount: payment.discount_taken,
                cost_center: None,
                profit_center: None,
                project_code: None,
                reference: Some(payment.payment_number.clone()),
                assignment: None,
                text: None,
                quantity: None,
                unit: None,
                tax_code: None,
                trading_partner: None,
                value_date: None,
            });
        }

        je
    }

    fn generate_debit_memo_je(&self, memo: &APDebitMemo) -> JournalEntry {
        let mut je = JournalEntry::new(
            format!("JE-{}", memo.debit_memo_number),
            memo.company_code.clone(),
            memo.posting_date,
            format!("AP Debit Memo {}", memo.debit_memo_number),
        );

        // Debit AP
        je.add_line(JournalEntryLine {
            line_number: 1,
            account_code: "2000".to_string(),
            account_description: Some("Accounts Payable".to_string()),
            debit_amount: memo.gross_amount.document_amount,
            credit_amount: Decimal::ZERO,
            cost_center: None,
            profit_center: None,
            project_code: None,
            reference: Some(memo.debit_memo_number.clone()),
            assignment: Some(memo.vendor_id.clone()),
            text: None,
            quantity: None,
            unit: None,
            tax_code: None,
            trading_partner: None,
            value_date: None,
        });

        // Credit Expense
        je.add_line(JournalEntryLine {
            line_number: 2,
            account_code: "5000".to_string(),
            account_description: Some("Expense".to_string()),
            debit_amount: Decimal::ZERO,
            credit_amount: memo.net_amount.document_amount,
            cost_center: None,
            profit_center: None,
            project_code: None,
            reference: Some(memo.debit_memo_number.clone()),
            assignment: None,
            text: None,
            quantity: None,
            unit: None,
            tax_code: None,
            trading_partner: None,
            value_date: None,
        });

        // Credit Tax
        if memo.tax_amount.document_amount > Decimal::ZERO {
            je.add_line(JournalEntryLine {
                line_number: 3,
                account_code: "1400".to_string(),
                account_description: Some("Input Tax".to_string()),
                debit_amount: Decimal::ZERO,
                credit_amount: memo.tax_amount.document_amount,
                cost_center: None,
                profit_center: None,
                project_code: None,
                reference: Some(memo.debit_memo_number.clone()),
                assignment: None,
                text: None,
                quantity: None,
                unit: None,
                tax_code: Some("VAT".to_string()),
                trading_partner: None,
                value_date: None,
            });
        }

        je
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::SeedableRng;

    #[test]
    fn test_generate_invoice() {
        let rng = ChaCha8Rng::seed_from_u64(12345);
        let mut generator = APGenerator::new(APGeneratorConfig::default(), rng);

        let (invoice, je) = generator.generate_invoice(
            "1000",
            "VEND001",
            "Test Vendor",
            "V-INV-001",
            NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(),
            "USD",
            2,
            Some("PO001"),
        );

        assert_eq!(invoice.lines.len(), 2);
        assert!(invoice.gross_amount.document_amount > Decimal::ZERO);
        assert!(je.is_balanced());
    }
}
