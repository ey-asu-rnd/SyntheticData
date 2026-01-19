# Domain Models

Core data structures representing enterprise financial concepts.

## Model Categories

| Category | Models |
|----------|--------|
| [Accounting](#accounting) | JournalEntry, ChartOfAccounts, ACDOCA |
| [Master Data](#master-data) | Vendor, Customer, Material, FixedAsset, Employee |
| [Documents](#documents) | PurchaseOrder, Invoice, Payment, etc. |
| [Financial](#financial) | TrialBalance, FxRate, AccountBalance |
| [Compliance](#compliance) | InternalControl, SoDRule, LabeledAnomaly |

---

## Accounting

### JournalEntry

The core accounting record.

```rust
pub struct JournalEntry {
    pub header: JournalEntryHeader,
    pub lines: Vec<JournalEntryLine>,
}

pub struct JournalEntryHeader {
    pub document_id: Uuid,
    pub company_code: String,
    pub fiscal_year: u16,
    pub fiscal_period: u8,
    pub posting_date: NaiveDate,
    pub document_date: NaiveDate,
    pub created_at: DateTime<Utc>,
    pub source: TransactionSource,
    pub business_process: Option<BusinessProcess>,

    // Document references
    pub source_document_type: Option<DocumentType>,
    pub source_document_id: Option<String>,

    // Labels
    pub is_fraud: bool,
    pub fraud_type: Option<FraudType>,
    pub is_anomaly: bool,
    pub anomaly_type: Option<AnomalyType>,

    // Control markers
    pub control_ids: Vec<String>,
    pub sox_relevant: bool,
    pub sod_violation: bool,
}

pub struct JournalEntryLine {
    pub line_number: u32,
    pub account_number: String,
    pub cost_center: Option<String>,
    pub profit_center: Option<String>,
    pub debit_amount: Decimal,
    pub credit_amount: Decimal,
    pub description: String,
    pub tax_code: Option<String>,
}
```

**Invariant:** Sum of debits must equal sum of credits.

### ChartOfAccounts

GL account structure.

```rust
pub struct ChartOfAccounts {
    pub accounts: Vec<Account>,
}

pub struct Account {
    pub account_number: String,
    pub name: String,
    pub account_type: AccountType,
    pub account_subtype: AccountSubType,
    pub is_control_account: bool,
    pub normal_balance: NormalBalance,
    pub is_active: bool,
}

pub enum AccountType {
    Asset,
    Liability,
    Equity,
    Revenue,
    Expense,
}

pub enum AccountSubType {
    // Assets
    Cash, AccountsReceivable, Inventory, FixedAsset,
    // Liabilities
    AccountsPayable, AccruedLiabilities, LongTermDebt,
    // Equity
    CommonStock, RetainedEarnings,
    // Revenue
    SalesRevenue, ServiceRevenue,
    // Expense
    CostOfGoodsSold, OperatingExpense,
    // ...
}
```

### ACDOCA

SAP HANA Universal Journal format.

```rust
pub struct AcdocaEntry {
    pub rclnt: String,           // Client
    pub rldnr: String,           // Ledger
    pub rbukrs: String,          // Company code
    pub gjahr: u16,              // Fiscal year
    pub belnr: String,           // Document number
    pub docln: u32,              // Line item
    pub ryear: u16,              // Year
    pub poper: u8,               // Posting period
    pub racct: String,           // Account
    pub drcrk: DebitCreditIndicator,
    pub hsl: Decimal,            // Amount in local currency
    pub ksl: Decimal,            // Amount in group currency

    // Simulation fields
    pub zsim_fraud: bool,
    pub zsim_anomaly: bool,
    pub zsim_source: String,
}
```

---

## Master Data

### Vendor

Supplier master record.

```rust
pub struct Vendor {
    pub vendor_id: String,
    pub vendor_name: String,
    pub tax_id: Option<String>,
    pub currency: String,
    pub country: String,
    pub payment_terms: PaymentTerms,
    pub bank_account: Option<BankAccount>,
    pub is_intercompany: bool,
    pub behavior: VendorBehavior,
    pub valid_from: NaiveDate,
    pub valid_to: Option<NaiveDate>,
}

pub struct VendorBehavior {
    pub late_payment_tendency: f64,
    pub discount_usage_rate: f64,
}
```

### Customer

Customer master record.

```rust
pub struct Customer {
    pub customer_id: String,
    pub customer_name: String,
    pub currency: String,
    pub country: String,
    pub credit_limit: Decimal,
    pub credit_rating: CreditRating,
    pub payment_behavior: PaymentBehavior,
    pub is_intercompany: bool,
    pub valid_from: NaiveDate,
}

pub struct PaymentBehavior {
    pub on_time_rate: f64,
    pub early_payment_rate: f64,
    pub late_payment_rate: f64,
    pub average_days_late: u32,
}
```

### Material

Product/material master.

```rust
pub struct Material {
    pub material_id: String,
    pub description: String,
    pub material_type: MaterialType,
    pub unit_of_measure: String,
    pub valuation_method: ValuationMethod,
    pub standard_cost: Decimal,
    pub gl_account: String,
}

pub enum MaterialType {
    RawMaterial,
    WorkInProgress,
    FinishedGoods,
    Service,
}

pub enum ValuationMethod {
    Fifo,
    Lifo,
    WeightedAverage,
    StandardCost,
}
```

### FixedAsset

Capital asset record.

```rust
pub struct FixedAsset {
    pub asset_id: String,
    pub description: String,
    pub asset_class: AssetClass,
    pub acquisition_date: NaiveDate,
    pub acquisition_cost: Decimal,
    pub useful_life_years: u32,
    pub depreciation_method: DepreciationMethod,
    pub salvage_value: Decimal,
    pub accumulated_depreciation: Decimal,
    pub disposal_date: Option<NaiveDate>,
}
```

### Employee

User/employee record.

```rust
pub struct Employee {
    pub employee_id: String,
    pub name: String,
    pub department: String,
    pub role: String,
    pub manager_id: Option<String>,
    pub approval_limit: Decimal,
    pub transaction_codes: Vec<String>,
    pub hire_date: NaiveDate,
}
```

---

## Documents

### PurchaseOrder

P2P initiating document.

```rust
pub struct PurchaseOrder {
    pub po_number: String,
    pub vendor_id: String,
    pub company_code: String,
    pub order_date: NaiveDate,
    pub items: Vec<PoLineItem>,
    pub total_amount: Decimal,
    pub currency: String,
    pub status: PoStatus,
}

pub struct PoLineItem {
    pub line_number: u32,
    pub material_id: String,
    pub quantity: Decimal,
    pub unit_price: Decimal,
    pub gl_account: String,
}
```

### VendorInvoice

AP invoice with three-way match.

```rust
pub struct VendorInvoice {
    pub invoice_number: String,
    pub vendor_id: String,
    pub po_number: Option<String>,
    pub gr_number: Option<String>,
    pub invoice_date: NaiveDate,
    pub due_date: NaiveDate,
    pub total_amount: Decimal,
    pub match_status: MatchStatus,
}

pub enum MatchStatus {
    Matched,
    QuantityVariance,
    PriceVariance,
    Blocked,
}
```

### DocumentReference

Links documents in flows.

```rust
pub struct DocumentReference {
    pub from_document_type: DocumentType,
    pub from_document_id: String,
    pub to_document_type: DocumentType,
    pub to_document_id: String,
    pub reference_type: ReferenceType,
}

pub enum ReferenceType {
    FollowsFrom,     // Normal flow
    PaymentFor,      // Payment → Invoice
    ReversalOf,      // Reversal/credit memo
}
```

---

## Financial

### TrialBalance

Period-end balances.

```rust
pub struct TrialBalance {
    pub company_code: String,
    pub fiscal_year: u16,
    pub fiscal_period: u8,
    pub accounts: Vec<TrialBalanceRow>,
}

pub struct TrialBalanceRow {
    pub account_number: String,
    pub account_name: String,
    pub opening_balance: Decimal,
    pub period_debits: Decimal,
    pub period_credits: Decimal,
    pub closing_balance: Decimal,
}
```

### FxRate

Exchange rate record.

```rust
pub struct FxRate {
    pub from_currency: String,
    pub to_currency: String,
    pub rate_date: NaiveDate,
    pub rate_type: RateType,
    pub rate: Decimal,
}

pub enum RateType {
    Spot,
    Closing,
    Average,
}
```

---

## Compliance

### LabeledAnomaly

ML training label.

```rust
pub struct LabeledAnomaly {
    pub document_id: Uuid,
    pub anomaly_id: String,
    pub anomaly_type: AnomalyType,
    pub category: AnomalyCategory,
    pub severity: Severity,
    pub description: String,
    pub detection_difficulty: DetectionDifficulty,
}

pub enum AnomalyType {
    Fraud,
    Error,
    ProcessIssue,
    Statistical,
    Relational,
}
```

### InternalControl

SOX control definition.

```rust
pub struct InternalControl {
    pub control_id: String,
    pub name: String,
    pub description: String,
    pub control_type: ControlType,
    pub frequency: ControlFrequency,
    pub assertions: Vec<Assertion>,
}
```

---

## Decimal Handling

All monetary amounts use `rust_decimal::Decimal`:

```rust
use rust_decimal::Decimal;
use rust_decimal_macros::dec;

let amount = dec!(1234.56);
let tax = amount * dec!(0.077);
```

Serialized as strings to prevent IEEE 754 issues:

```json
{"amount": "1234.56"}
```

## See Also

- [synth-core Crate](../crates/synth-core.md)
- [Data Flow](data-flow.md)
- [Generation Pipeline](generation-pipeline.md)
