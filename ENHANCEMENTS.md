# Comprehensive Enterprise Simulation Enhancements

## Vision

Transform the synthetic data generator from a transaction producer into a **complete enterprise simulation engine** that generates coherent, interconnected data representing the full lifecycle of a multi-national corporation. The output should be indistinguishable from real enterprise data for testing algorithms, training neural networks, and validating analytics systems.

---

## Current State Analysis

### What We Have
- Benford's Law compliant amounts with fraud patterns
- Internal Controls System (ICS) with SOX 404 markers
- Segregation of Duties (SoD) conflict detection
- Industry-specific seasonality (10 sectors)
- Regional holiday calendars (6 regions)
- Weighted company selection
- Chart of Accounts with hierarchy
- Journal entries with business processes
- ACDOCA SAP-compatible event logs
- User personas and approval workflows

### Critical Gaps for Enterprise Simulation
1. **No master data coherence** - Transactions reference non-existent entities
2. **No intercompany relationships** - Subsidiaries operate in isolation
3. **No document flow** - PO/Invoice/Payment are disconnected
4. **No balance coherence** - Debits may not equal credits over time
5. **No subledger detail** - Missing AR/AP/FA/Inventory granularity
6. **No temporal consistency** - Entities used before they exist
7. **No graph structure** - Missing relationship data for GNNs

---

## Enhancement Architecture

### Conceptual Model

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                        ENTERPRISE SIMULATION ENGINE                          │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                              │
│  ┌──────────────┐    ┌──────────────┐    ┌──────────────┐                   │
│  │   MASTER     │    │  TRANSACTION │    │   PERIOD     │                   │
│  │    DATA      │───▶│    ENGINE    │───▶│    CLOSE     │                   │
│  │  GENERATOR   │    │              │    │   ENGINE     │                   │
│  └──────────────┘    └──────────────┘    └──────────────┘                   │
│         │                   │                   │                            │
│         ▼                   ▼                   ▼                            │
│  ┌──────────────┐    ┌──────────────┐    ┌──────────────┐                   │
│  │   ENTITY     │    │   DOCUMENT   │    │ CONSOLIDATION│                   │
│  │  REGISTRY    │    │    FLOW      │    │    ENGINE    │                   │
│  │              │    │   MANAGER    │    │              │                   │
│  └──────────────┘    └──────────────┘    └──────────────┘                   │
│         │                   │                   │                            │
│         └───────────────────┴───────────────────┘                            │
│                             │                                                │
│                             ▼                                                │
│                    ┌──────────────────┐                                      │
│                    │   GRAPH/NETWORK  │                                      │
│                    │     EXPORTER     │                                      │
│                    └──────────────────┘                                      │
│                                                                              │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## Phase 1: Master Data Foundation

### 1.1 Entity Registry

A central registry tracking all generated entities with temporal validity.

```rust
pub struct EntityRegistry {
    vendors: HashMap<String, VendorMaster>,
    customers: HashMap<String, CustomerMaster>,
    materials: HashMap<String, MaterialMaster>,
    employees: HashMap<String, EmployeeMaster>,
    cost_centers: HashMap<String, CostCenterMaster>,
    profit_centers: HashMap<String, ProfitCenterMaster>,
    assets: HashMap<String, FixedAssetMaster>,

    // Temporal tracking
    entity_timeline: BTreeMap<NaiveDate, Vec<EntityEvent>>,
}

pub enum EntityEvent {
    Created { entity_type: EntityType, id: String },
    Modified { entity_type: EntityType, id: String, field: String },
    Deactivated { entity_type: EntityType, id: String },
}
```

### 1.2 Vendor Master

```rust
pub struct VendorMaster {
    pub vendor_id: String,           // V000001
    pub name: String,
    pub vendor_type: VendorType,     // Supplier, ServiceProvider, Contractor
    pub country: String,
    pub currency: String,
    pub payment_terms: PaymentTerms, // Net30, Net60, 2/10Net30
    pub credit_limit: Decimal,
    pub bank_details: BankDetails,
    pub tax_id: String,
    pub category: VendorCategory,    // Strategic, Preferred, Standard, OneTime
    pub industry: IndustrySector,
    pub created_date: NaiveDate,
    pub is_active: bool,

    // For intercompany
    pub is_intercompany: bool,
    pub related_company_code: Option<String>,

    // Behavioral attributes for simulation
    pub avg_invoice_amount: Decimal,
    pub invoice_frequency: InvoiceFrequency,
    pub typical_gl_accounts: Vec<String>,
}

pub enum PaymentTerms {
    Immediate,
    Net15,
    Net30,
    Net45,
    Net60,
    Net90,
    TwoTenNet30,  // 2% discount if paid in 10 days
}
```

### 1.3 Customer Master

```rust
pub struct CustomerMaster {
    pub customer_id: String,         // C000001
    pub name: String,
    pub customer_type: CustomerType, // B2B, B2C, Government, Internal
    pub country: String,
    pub currency: String,
    pub credit_limit: Decimal,
    pub payment_terms: PaymentTerms,
    pub credit_rating: CreditRating, // AAA, AA, A, BBB, BB, B, CCC, Default
    pub industry: IndustrySector,
    pub sales_region: String,
    pub account_manager: String,     // Employee ID
    pub created_date: NaiveDate,
    pub is_active: bool,

    // For intercompany
    pub is_intercompany: bool,
    pub related_company_code: Option<String>,

    // Behavioral attributes
    pub avg_order_value: Decimal,
    pub order_frequency: OrderFrequency,
    pub seasonality_pattern: Option<String>,
    pub payment_behavior: PaymentBehavior, // OnTime, SlightlyLate, Late, Delinquent
}
```

### 1.4 Material/Product Master

```rust
pub struct MaterialMaster {
    pub material_id: String,         // M0000001
    pub description: String,
    pub material_type: MaterialType, // RawMaterial, SemiFinished, Finished, Service
    pub material_group: String,
    pub unit_of_measure: String,
    pub standard_cost: Decimal,
    pub standard_price: Decimal,
    pub margin_percent: f64,
    pub weight: Option<f64>,
    pub volume: Option<f64>,
    pub is_serialized: bool,
    pub is_batch_managed: bool,
    pub shelf_life_days: Option<u32>,
    pub abc_classification: AbcClass, // A (high value), B (medium), C (low)
    pub xyz_classification: XyzClass, // X (stable demand), Y (variable), Z (sporadic)
    pub created_date: NaiveDate,
    pub is_active: bool,

    // BOM (Bill of Materials) for manufactured items
    pub bom_components: Vec<BomComponent>,

    // GL account determination
    pub inventory_account: String,
    pub cogs_account: String,
    pub revenue_account: String,
}
```

### 1.5 Fixed Asset Master

```rust
pub struct FixedAssetMaster {
    pub asset_id: String,            // A00000001
    pub description: String,
    pub asset_class: AssetClass,     // Buildings, Machinery, Vehicles, IT, Furniture
    pub acquisition_date: NaiveDate,
    pub acquisition_cost: Decimal,
    pub useful_life_months: u32,
    pub depreciation_method: DepreciationMethod, // StraightLine, DecliningBalance, SumOfYears
    pub salvage_value: Decimal,
    pub accumulated_depreciation: Decimal,
    pub net_book_value: Decimal,
    pub location: String,
    pub cost_center: String,
    pub responsible_employee: String,
    pub is_disposed: bool,
    pub disposal_date: Option<NaiveDate>,
    pub disposal_amount: Option<Decimal>,
}
```

### 1.6 Employee Master

```rust
pub struct EmployeeMaster {
    pub employee_id: String,         // E000001
    pub name: String,
    pub email: String,
    pub department: String,
    pub cost_center: String,
    pub job_title: String,
    pub job_level: JobLevel,         // L1-L10, Executive
    pub hire_date: NaiveDate,
    pub termination_date: Option<NaiveDate>,
    pub manager_id: Option<String>,
    pub company_code: String,
    pub country: String,
    pub annual_salary: Decimal,
    pub currency: String,
    pub is_approver: bool,
    pub approval_limit: Decimal,
    pub user_persona: UserPersona,
    pub system_access: Vec<SystemAccess>,
}
```

---

## Phase 2: Document Flow Engine

### 2.1 Procure-to-Pay (P2P) Flow

```
Purchase Requisition → Purchase Order → Goods Receipt → Invoice Receipt → Payment
        │                    │                │              │             │
        ▼                    ▼                ▼              ▼             ▼
   PR Document          PO Document      GR Document    IR Document   Payment Doc
   (internal)           (to vendor)      (inventory)    (liability)   (cash out)
```

```rust
pub struct PurchaseOrder {
    pub po_number: String,
    pub company_code: String,
    pub vendor_id: String,
    pub order_date: NaiveDate,
    pub delivery_date: NaiveDate,
    pub payment_terms: PaymentTerms,
    pub currency: String,
    pub total_amount: Decimal,
    pub status: PoStatus,
    pub created_by: String,
    pub approved_by: Option<String>,
    pub approval_date: Option<NaiveDateTime>,
    pub items: Vec<PoItem>,
}

pub struct PoItem {
    pub item_number: u32,
    pub material_id: String,
    pub description: String,
    pub quantity: Decimal,
    pub unit_price: Decimal,
    pub unit_of_measure: String,
    pub gl_account: String,
    pub cost_center: String,
    pub tax_code: String,
}

pub struct GoodsReceipt {
    pub gr_number: String,
    pub po_reference: String,
    pub receipt_date: NaiveDate,
    pub items: Vec<GrItem>,
    pub created_by: String,
}

pub struct VendorInvoice {
    pub invoice_number: String,
    pub vendor_invoice_ref: String,  // External invoice number
    pub po_reference: Option<String>,
    pub gr_reference: Option<String>,
    pub vendor_id: String,
    pub invoice_date: NaiveDate,
    pub due_date: NaiveDate,
    pub total_amount: Decimal,
    pub tax_amount: Decimal,
    pub currency: String,
    pub status: InvoiceStatus,
    pub three_way_match: ThreeWayMatchResult,
    pub items: Vec<InvoiceItem>,
}

pub struct Payment {
    pub payment_id: String,
    pub payment_date: NaiveDate,
    pub vendor_id: String,
    pub payment_method: PaymentMethod, // Check, Wire, ACH, Card
    pub bank_account: String,
    pub total_amount: Decimal,
    pub currency: String,
    pub invoices_paid: Vec<InvoicePayment>,
    pub created_by: String,
    pub approved_by: String,
}
```

### 2.2 Order-to-Cash (O2C) Flow

```
Sales Quotation → Sales Order → Delivery → Customer Invoice → Collection
       │               │            │              │              │
       ▼               ▼            ▼              ▼              ▼
  Quote Doc       SO Document   Delivery Doc   AR Invoice    Receipt Doc
  (optional)      (commitment)  (inventory)    (receivable)  (cash in)
```

```rust
pub struct SalesOrder {
    pub so_number: String,
    pub company_code: String,
    pub customer_id: String,
    pub order_date: NaiveDate,
    pub requested_delivery_date: NaiveDate,
    pub payment_terms: PaymentTerms,
    pub currency: String,
    pub total_amount: Decimal,
    pub margin_percent: f64,
    pub status: SoStatus,
    pub sales_rep: String,
    pub items: Vec<SoItem>,
}

pub struct Delivery {
    pub delivery_number: String,
    pub so_reference: String,
    pub ship_date: NaiveDate,
    pub carrier: String,
    pub tracking_number: String,
    pub items: Vec<DeliveryItem>,
}

pub struct CustomerInvoice {
    pub invoice_number: String,
    pub so_reference: String,
    pub delivery_reference: String,
    pub customer_id: String,
    pub invoice_date: NaiveDate,
    pub due_date: NaiveDate,
    pub total_amount: Decimal,
    pub tax_amount: Decimal,
    pub currency: String,
    pub status: InvoiceStatus,
    pub items: Vec<InvoiceItem>,
}

pub struct CustomerReceipt {
    pub receipt_id: String,
    pub receipt_date: NaiveDate,
    pub customer_id: String,
    pub payment_method: PaymentMethod,
    pub bank_account: String,
    pub total_amount: Decimal,
    pub currency: String,
    pub invoices_applied: Vec<InvoiceApplication>,
    pub days_to_pay: i32,  // For payment behavior analysis
}
```

### 2.3 Document Reference Chain

Every document maintains references to related documents:

```rust
pub struct DocumentReference {
    pub document_type: DocumentType,
    pub document_number: String,
    pub reference_type: ReferenceType, // PrecedingDoc, FollowingDoc, RelatedDoc
    pub reference_document_type: DocumentType,
    pub reference_document_number: String,
}

// Example chain:
// PO-000123 → GR-000456 → IR-000789 → PAY-001234
// Each generates journal entries that cross-reference
```

---

## Phase 3: Intercompany Transactions

### 3.1 Intercompany Relationship Model

```rust
pub struct IntercompanyRelationship {
    pub parent_company: String,
    pub subsidiary_company: String,
    pub ownership_percent: f64,      // 0.0 - 100.0
    pub consolidation_method: ConsolidationMethod, // Full, Proportional, Equity
    pub functional_currency: String,
    pub transfer_pricing_method: TransferPricingMethod,
    pub ic_agreement_date: NaiveDate,
}

pub enum TransferPricingMethod {
    CostPlus { markup_percent: f64 },
    ResaleMinus { margin_percent: f64 },
    ComparableUncontrolled { benchmark_rate: f64 },
    TransactionalNetMargin { profit_indicator: f64 },
    ProfitSplit { split_ratio: f64 },
}

pub enum ConsolidationMethod {
    Full,           // 100% consolidation (>50% ownership)
    Proportional,   // Consolidate by ownership % (joint ventures)
    Equity,         // Book as investment (<50% ownership)
}
```

### 3.2 Intercompany Transaction Types

```rust
pub enum IntercompanyTransactionType {
    // Goods & Services
    GoodsSale { margin_percent: f64 },
    ServiceProvided { rate_type: RateType },

    // Financial
    Loan { interest_rate: f64, term_months: u32 },
    LoanRepayment,
    InterestPayment,
    Dividend,
    CapitalContribution,

    // Allocations
    ManagementFee { basis: AllocationBasis },
    Royalty { rate_percent: f64 },
    CostAllocation { basis: AllocationBasis },
    SharedServiceCharge,

    // Transfers
    AssetTransfer,
    InventoryTransfer,
    CashPooling,
}

pub enum AllocationBasis {
    Revenue,
    Headcount,
    SquareFootage,
    TransactionVolume,
    Custom { formula: String },
}
```

### 3.3 Intercompany Matching Engine

```rust
pub struct IntercompanyMatcher {
    pub tolerance_amount: Decimal,
    pub tolerance_days: i32,
}

impl IntercompanyMatcher {
    /// Generate matching IC transactions for both entities
    pub fn generate_ic_pair(
        &self,
        ic_type: IntercompanyTransactionType,
        from_company: &str,
        to_company: &str,
        amount: Decimal,
        date: NaiveDate,
    ) -> (JournalEntry, JournalEntry) {
        // Seller books: DR IC Receivable, CR Revenue
        // Buyer books:  DR Expense/Asset, CR IC Payable
        // Amounts match exactly for elimination
    }
}
```

### 3.4 Consolidation Eliminations

```rust
pub struct ConsolidationElimination {
    pub elimination_id: String,
    pub period: FiscalPeriod,
    pub elimination_type: EliminationType,
    pub company_pair: (String, String),
    pub original_amount: Decimal,
    pub eliminated_amount: Decimal,
    pub variance: Decimal,
    pub variance_reason: Option<String>,
}

pub enum EliminationType {
    IntercompanyRevenueCost,    // Eliminate IC sales/purchases
    IntercompanyReceivablePayable, // Eliminate IC balances
    IntercompanyProfit,         // Eliminate unrealized IC profit in inventory
    IntercompanyDividend,       // Eliminate IC dividends
    InvestmentEquity,          // Eliminate investment vs. subsidiary equity
}
```

---

## Phase 4: Balance Coherence

### 4.1 Opening Balance Generator

```rust
pub struct OpeningBalanceGenerator {
    config: BalanceConfig,
}

impl OpeningBalanceGenerator {
    /// Generate coherent opening balance sheet
    pub fn generate(&self, company: &CompanyConfig, start_date: NaiveDate) -> BalanceSheet {
        // Assets = Liabilities + Equity (always balanced)

        let total_assets = self.calculate_total_assets(company);

        // Distribute across asset accounts based on industry
        let assets = self.distribute_assets(total_assets, company.industry);

        // Calculate liabilities (typically 40-70% of assets depending on industry)
        let debt_ratio = self.industry_debt_ratio(company.industry);
        let total_liabilities = total_assets * debt_ratio;
        let liabilities = self.distribute_liabilities(total_liabilities, company.industry);

        // Equity is the plug
        let total_equity = total_assets - total_liabilities;
        let equity = self.distribute_equity(total_equity);

        BalanceSheet { assets, liabilities, equity }
    }
}
```

### 4.2 Running Balance Tracker

```rust
pub struct BalanceTracker {
    // Account -> (Debit Balance, Credit Balance)
    balances: HashMap<String, (Decimal, Decimal)>,

    // Period snapshots for trial balance
    period_snapshots: BTreeMap<FiscalPeriod, TrialBalance>,
}

impl BalanceTracker {
    pub fn post_entry(&mut self, entry: &JournalEntry) {
        for line in &entry.lines {
            let (debit, credit) = self.balances
                .entry(line.gl_account.clone())
                .or_insert((Decimal::ZERO, Decimal::ZERO));

            *debit += line.debit_amount;
            *credit += line.credit_amount;
        }
    }

    pub fn validate_entry(&self, entry: &JournalEntry) -> Result<(), BalanceError> {
        let total_debit: Decimal = entry.lines.iter().map(|l| l.debit_amount).sum();
        let total_credit: Decimal = entry.lines.iter().map(|l| l.credit_amount).sum();

        if total_debit != total_credit {
            return Err(BalanceError::Unbalanced { debit: total_debit, credit: total_credit });
        }
        Ok(())
    }

    pub fn get_trial_balance(&self, period: FiscalPeriod) -> TrialBalance {
        // Generate trial balance as of period end
    }
}
```

### 4.3 Account Balance Relationships

```rust
/// Define expected relationships between accounts for coherence checking
pub struct BalanceRelationship {
    pub relationship_type: RelationshipType,
    pub source_account: String,
    pub target_account: String,
    pub expected_ratio: Option<f64>,
    pub tolerance: f64,
}

pub enum RelationshipType {
    /// Revenue should correlate with COGS (gross margin check)
    RevenueToCoGs { expected_margin: f64 },

    /// AR should correlate with Revenue (DSO check)
    ReceivablesToRevenue { expected_dso: f64 },

    /// AP should correlate with COGS (DPO check)
    PayablesToCogs { expected_dpo: f64 },

    /// Inventory should correlate with COGS (DIO check)
    InventoryToCogs { expected_dio: f64 },

    /// Depreciation should correlate with Fixed Assets
    DepreciationToAssets { expected_rate: f64 },

    /// Interest expense should correlate with Debt
    InterestToDebt { expected_rate: f64 },

    /// Tax expense should correlate with Pre-tax income
    TaxToIncome { effective_rate: f64 },

    /// Payroll should correlate with headcount
    PayrollToHeadcount { avg_salary: Decimal },
}
```

---

## Phase 5: Subledger Simulation

### 5.1 Accounts Receivable Subledger

```rust
pub struct ArSubledger {
    pub invoices: HashMap<String, ArInvoice>,
    pub receipts: HashMap<String, CustomerReceipt>,
    pub credit_memos: HashMap<String, CreditMemo>,

    // Aging buckets
    pub aging: ArAging,
}

pub struct ArAging {
    pub current: Decimal,        // 0-30 days
    pub days_31_60: Decimal,
    pub days_61_90: Decimal,
    pub days_91_120: Decimal,
    pub over_120: Decimal,
}

impl ArSubledger {
    pub fn reconcile_to_gl(&self, gl_balance: Decimal) -> ReconciliationResult {
        let subledger_total = self.calculate_total_outstanding();
        ReconciliationResult {
            gl_balance,
            subledger_balance: subledger_total,
            difference: gl_balance - subledger_total,
            is_reconciled: (gl_balance - subledger_total).abs() < Decimal::new(1, 2),
        }
    }

    pub fn generate_aging_report(&self, as_of_date: NaiveDate) -> AgingReport {
        // Calculate aging buckets based on invoice due dates
    }
}
```

### 5.2 Accounts Payable Subledger

```rust
pub struct ApSubledger {
    pub invoices: HashMap<String, ApInvoice>,
    pub payments: HashMap<String, Payment>,
    pub debit_memos: HashMap<String, DebitMemo>,

    // Payment scheduling
    pub payment_schedule: Vec<ScheduledPayment>,
}

impl ApSubledger {
    pub fn get_cash_requirements(&self, from: NaiveDate, to: NaiveDate) -> CashRequirements {
        // Calculate upcoming payment obligations
    }

    pub fn apply_early_payment_discount(&mut self, invoice_id: &str) -> Option<Decimal> {
        // Calculate and apply 2/10 net 30 discounts
    }
}
```

### 5.3 Fixed Asset Subledger

```rust
pub struct FixedAssetSubledger {
    pub assets: HashMap<String, FixedAssetMaster>,
    pub depreciation_schedule: Vec<DepreciationEntry>,
    pub disposals: Vec<AssetDisposal>,
}

impl FixedAssetSubledger {
    pub fn run_depreciation(&mut self, period: FiscalPeriod) -> Vec<JournalEntry> {
        // Calculate and post depreciation for all active assets
        self.assets.values()
            .filter(|a| !a.is_disposed)
            .map(|asset| self.calculate_period_depreciation(asset, period))
            .collect()
    }

    pub fn calculate_period_depreciation(
        &self,
        asset: &FixedAssetMaster,
        period: FiscalPeriod,
    ) -> JournalEntry {
        let monthly_depreciation = match asset.depreciation_method {
            DepreciationMethod::StraightLine => {
                (asset.acquisition_cost - asset.salvage_value)
                    / Decimal::from(asset.useful_life_months)
            }
            DepreciationMethod::DecliningBalance { rate } => {
                asset.net_book_value * Decimal::from_f64(rate / 12.0).unwrap()
            }
            // ... other methods
        };

        // DR Depreciation Expense, CR Accumulated Depreciation
        self.create_depreciation_entry(asset, monthly_depreciation, period)
    }
}
```

### 5.4 Inventory Subledger

```rust
pub struct InventorySubledger {
    pub materials: HashMap<String, InventoryPosition>,
    pub movements: Vec<InventoryMovement>,
    pub valuations: HashMap<String, InventoryValuation>,
}

pub struct InventoryPosition {
    pub material_id: String,
    pub location: String,
    pub quantity_on_hand: Decimal,
    pub quantity_reserved: Decimal,
    pub quantity_available: Decimal,
    pub standard_cost: Decimal,
    pub total_value: Decimal,
}

pub enum InventoryMovement {
    GoodsReceipt { po_reference: String, quantity: Decimal, value: Decimal },
    GoodsIssue { so_reference: String, quantity: Decimal, value: Decimal },
    TransferPosting { from_location: String, to_location: String, quantity: Decimal },
    PhysicalInventoryAdjustment { quantity_diff: Decimal, reason: String },
    Scrap { quantity: Decimal, reason: String },
}

impl InventorySubledger {
    pub fn post_goods_receipt(&mut self, gr: &GoodsReceipt) -> JournalEntry {
        // DR Inventory, CR GR/IR Clearing
    }

    pub fn post_goods_issue(&mut self, delivery: &Delivery) -> JournalEntry {
        // DR COGS, CR Inventory (at standard cost)
    }

    pub fn calculate_cogs(&self, period: FiscalPeriod) -> Decimal {
        // Sum of all goods issues at standard cost
    }
}
```

---

## Phase 6: Currency & FX

### 6.1 FX Rate Service

```rust
pub struct FxRateService {
    // Date -> (FromCurrency, ToCurrency) -> Rate
    rates: BTreeMap<NaiveDate, HashMap<(String, String), Decimal>>,

    // Historical volatility by currency pair
    volatility: HashMap<(String, String), f64>,
}

impl FxRateService {
    pub fn generate_rates(&mut self, start: NaiveDate, end: NaiveDate, base_currency: &str) {
        // Generate realistic FX rates with:
        // - Random walk with mean reversion
        // - Appropriate volatility per currency pair
        // - Occasional larger moves (fat tails)
    }

    pub fn get_rate(&self, date: NaiveDate, from: &str, to: &str) -> Decimal {
        // Return rate, triangulating through USD if needed
    }

    pub fn get_average_rate(&self, period: FiscalPeriod, from: &str, to: &str) -> Decimal {
        // For P&L translation
    }

    pub fn get_closing_rate(&self, period: FiscalPeriod, from: &str, to: &str) -> Decimal {
        // For balance sheet translation
    }
}
```

### 6.2 Currency Translation

```rust
pub struct CurrencyTranslator {
    fx_service: FxRateService,
    group_currency: String,
}

impl CurrencyTranslator {
    pub fn translate_trial_balance(
        &self,
        local_tb: &TrialBalance,
        local_currency: &str,
        period: FiscalPeriod,
    ) -> (TrialBalance, CurrencyTranslationAdjustment) {
        let mut translated = TrialBalance::new();
        let mut cta = Decimal::ZERO;

        for (account, balance) in &local_tb.balances {
            let account_type = self.get_account_type(account);

            let rate = match account_type {
                AccountType::Asset | AccountType::Liability => {
                    self.fx_service.get_closing_rate(period, local_currency, &self.group_currency)
                }
                AccountType::Equity => {
                    // Historical rate (simplified: use average)
                    self.fx_service.get_average_rate(period, local_currency, &self.group_currency)
                }
                AccountType::Revenue | AccountType::Expense => {
                    self.fx_service.get_average_rate(period, local_currency, &self.group_currency)
                }
            };

            translated.balances.insert(account.clone(), balance * rate);
        }

        // CTA is the plug to balance
        cta = translated.total_assets() - translated.total_liabilities() - translated.total_equity();

        (translated, CurrencyTranslationAdjustment { amount: cta, period })
    }
}
```

---

## Phase 7: Period Close Engine

### 7.1 Period Close Checklist

```rust
pub struct PeriodCloseEngine {
    company_code: String,
    period: FiscalPeriod,
    checklist: Vec<CloseTask>,
}

pub enum CloseTask {
    // Subledger closes
    RunDepreciation,
    PostInventoryRevaluation,
    ReconcileArToGl,
    ReconcileApToGl,
    ReconcileInventoryToGl,

    // Accruals
    PostAccruedExpenses,
    PostAccruedRevenue,
    PostPrepaidAmortization,
    PostDeferredRevenueRecognition,

    // Allocations
    AllocateCorporateOverhead,
    AllocateSharedServices,

    // Intercompany
    ReconcileIntercompany,
    PostIntercompanySettlements,

    // Tax
    CalculateAndPostTaxProvision,
    PostDeferredTaxAdjustments,

    // Consolidation (parent only)
    TranslateForeignSubsidiaries,
    EliminateIntercompany,
    CalculateMinorityInterest,

    // Year-end only
    CloseRevenueAndExpense,
    PostRetainedEarningsRollforward,
}

impl PeriodCloseEngine {
    pub fn execute(&mut self) -> Vec<JournalEntry> {
        let mut entries = Vec::new();

        for task in &self.checklist {
            match task {
                CloseTask::RunDepreciation => {
                    entries.extend(self.run_depreciation());
                }
                CloseTask::PostAccruedExpenses => {
                    entries.extend(self.generate_accruals());
                }
                // ... etc
            }
        }

        entries
    }

    fn generate_accruals(&self) -> Vec<JournalEntry> {
        // Generate realistic accrual entries:
        // - Utility accruals (based on historical average)
        // - Payroll accruals (days worked * daily rate)
        // - Interest accruals (principal * rate * days / 365)
        // - Professional fees accruals
    }
}
```

### 7.2 Year-End Close

```rust
pub struct YearEndClose {
    fiscal_year: i32,
    company_code: String,
}

impl YearEndClose {
    pub fn close_books(&self, trial_balance: &TrialBalance) -> Vec<JournalEntry> {
        let mut entries = Vec::new();

        // Close revenue accounts to Income Summary
        for (account, balance) in &trial_balance.balances {
            if self.is_revenue_account(account) {
                entries.push(self.create_closing_entry(
                    account,
                    "INCOME_SUMMARY",
                    balance.credit_balance,
                ));
            }
        }

        // Close expense accounts to Income Summary
        for (account, balance) in &trial_balance.balances {
            if self.is_expense_account(account) {
                entries.push(self.create_closing_entry(
                    "INCOME_SUMMARY",
                    account,
                    balance.debit_balance,
                ));
            }
        }

        // Close Income Summary to Retained Earnings
        let net_income = self.calculate_net_income(trial_balance);
        entries.push(self.create_closing_entry(
            "INCOME_SUMMARY",
            "RETAINED_EARNINGS",
            net_income,
        ));

        entries
    }
}
```

---

## Phase 8: Graph/Network Export

### 8.1 Transaction Network

For GNN-based fraud detection:

```rust
pub struct TransactionGraph {
    // Nodes
    pub accounts: Vec<AccountNode>,
    pub entities: Vec<EntityNode>,  // Vendors, Customers, Employees
    pub documents: Vec<DocumentNode>,

    // Edges
    pub transaction_edges: Vec<TransactionEdge>,
    pub approval_edges: Vec<ApprovalEdge>,
    pub reference_edges: Vec<ReferenceEdge>,
}

pub struct AccountNode {
    pub id: String,
    pub account_type: AccountType,
    pub features: Vec<f64>,  // Embedding features
}

pub struct TransactionEdge {
    pub source: String,      // Account or Entity
    pub target: String,      // Account or Entity
    pub amount: f64,
    pub timestamp: i64,      // Unix timestamp
    pub edge_type: EdgeType,
    pub features: Vec<f64>,  // Edge features for ML
    pub label: Option<String>, // For supervised learning (fraud type)
}

impl TransactionGraph {
    pub fn export_for_pytorch_geometric(&self, path: &Path) -> Result<()> {
        // Export node features: nodes.csv
        // Export edge index: edge_index.csv
        // Export edge attributes: edge_attr.csv
        // Export labels: labels.csv
    }

    pub fn export_for_dgl(&self, path: &Path) -> Result<()> {
        // Export in DGL format
    }

    pub fn export_for_neo4j(&self, path: &Path) -> Result<()> {
        // Export Cypher import statements
    }
}
```

### 8.2 Approval Network

```rust
pub struct ApprovalGraph {
    pub users: Vec<UserNode>,
    pub documents: Vec<DocumentNode>,
    pub approval_edges: Vec<ApprovalEdge>,
}

pub struct ApprovalEdge {
    pub approver_id: String,
    pub document_id: String,
    pub approval_action: ApprovalAction,
    pub timestamp: i64,
    pub amount: f64,
    pub is_sod_violation: bool,
    pub within_limit: bool,
}

// Useful for:
// - SoD violation detection
// - Rubber-stamping detection
// - Approval collusion networks
```

### 8.3 Entity Relationship Graph

```rust
pub struct EntityRelationshipGraph {
    pub legal_entities: Vec<LegalEntityNode>,
    pub ownership_edges: Vec<OwnershipEdge>,
    pub trading_edges: Vec<TradingRelationshipEdge>,
}

pub struct OwnershipEdge {
    pub parent: String,
    pub subsidiary: String,
    pub ownership_percent: f64,
    pub effective_date: NaiveDate,
}

// Useful for:
// - Consolidation path analysis
// - Related party identification
// - Ultimate beneficial owner tracing
```

---

## Phase 9: Anomaly Injection Framework

### 9.1 Labeled Anomaly Types

```rust
pub enum AnomalyType {
    // Fraud patterns
    Fraud(FraudType),

    // Errors
    DuplicateEntry,
    ReversedAmount,
    WrongPeriod,
    MissingApproval,
    WrongAccount,

    // Process issues
    LatePosting { days_late: i32 },
    SkippedApproval,
    UnusualTiming,

    // Statistical anomalies
    UnusualAmount { z_score: f64 },
    UnusualFrequency,
    SeasonalityDeviation,
    TrendBreak,

    // Relational anomalies
    NewVendorLargePayment,
    DormantAccountActivity,
    UnusualAccountCombination,
    CircularTransaction,
}

pub struct LabeledAnomaly {
    pub document_id: String,
    pub anomaly_type: AnomalyType,
    pub severity: AnomalySeverity,
    pub confidence: f64,
    pub explanation: String,
    pub detection_features: HashMap<String, f64>,
}
```

### 9.2 Anomaly Injection Strategy

```rust
pub struct AnomalyInjector {
    config: AnomalyConfig,
    rng: ChaCha8Rng,
}

pub struct AnomalyConfig {
    // Overall anomaly rate
    pub total_anomaly_rate: f64,  // e.g., 0.02 = 2%

    // Distribution across types
    pub fraud_rate: f64,
    pub error_rate: f64,
    pub process_issue_rate: f64,
    pub statistical_anomaly_rate: f64,

    // Clustering (anomalies often come in batches)
    pub cluster_probability: f64,
    pub cluster_size_range: (usize, usize),

    // Temporal patterns
    pub year_end_spike: f64,  // Higher anomaly rate at year-end
    pub weekend_spike: f64,
}

impl AnomalyInjector {
    pub fn inject(&mut self, entries: &mut Vec<JournalEntry>) -> Vec<LabeledAnomaly> {
        let mut labels = Vec::new();

        for entry in entries {
            if self.should_inject_anomaly(&entry) {
                let anomaly_type = self.select_anomaly_type(&entry);
                let label = self.apply_anomaly(entry, anomaly_type);
                labels.push(label);
            }
        }

        labels
    }
}
```

---

## Phase 10: Data Quality Variations

### 10.1 Realistic Data Quality Issues

```rust
pub struct DataQualityInjector {
    config: DataQualityConfig,
}

pub struct DataQualityConfig {
    // Missing values
    pub missing_rate: f64,
    pub missing_by_field: HashMap<String, f64>,

    // Format variations
    pub date_format_variations: bool,
    pub amount_format_variations: bool,
    pub name_spelling_variations: bool,

    // Duplicates
    pub exact_duplicate_rate: f64,
    pub near_duplicate_rate: f64,

    // Typos
    pub typo_rate: f64,

    // Encoding issues
    pub encoding_issue_rate: f64,
}

impl DataQualityInjector {
    pub fn inject_quality_issues(&self, data: &mut Vec<JournalEntry>) {
        for entry in data {
            // Randomly null out optional fields
            if self.rng.gen::<f64>() < self.config.missing_rate {
                self.remove_random_optional_field(entry);
            }

            // Introduce typos in text fields
            if self.rng.gen::<f64>() < self.config.typo_rate {
                self.introduce_typo(entry);
            }

            // Create near-duplicates
            if self.rng.gen::<f64>() < self.config.near_duplicate_rate {
                self.create_near_duplicate(entry);
            }
        }
    }
}
```

---

## Implementation Roadmap

### Milestone 1: Master Data Foundation (2-3 weeks effort)
1. Entity Registry implementation
2. Vendor Master generator
3. Customer Master generator
4. Material Master generator
5. Employee Master generator
6. Temporal validity tracking

### Milestone 2: Document Flow Engine (3-4 weeks effort)
1. P2P document flow (PO → GR → IR → Payment)
2. O2C document flow (SO → Delivery → Invoice → Receipt)
3. Document reference chain
4. Three-way matching logic
5. Payment term calculations

### Milestone 3: Balance Coherence (2 weeks effort)
1. Opening balance generator
2. Running balance tracker
3. Trial balance generation
4. Balance relationship validation
5. Account balance correlations

### Milestone 4: Intercompany Engine (2-3 weeks effort)
1. Intercompany relationship model
2. IC transaction generator
3. Transfer pricing logic
4. IC matching and reconciliation
5. Consolidation eliminations

### Milestone 5: Subledgers (3-4 weeks effort)
1. AR subledger with aging
2. AP subledger with scheduling
3. Fixed Asset register with depreciation
4. Inventory subledger with movements
5. GL-to-subledger reconciliation

### Milestone 6: Currency & Period Close (2 weeks effort)
1. FX rate service with realistic rates
2. Currency translation logic
3. Period close engine
4. Year-end closing entries
5. Accrual generation

### Milestone 7: Graph Export & Analytics (2 weeks effort)
1. Transaction network export
2. Approval network export
3. Entity relationship graph
4. PyTorch Geometric format
5. Neo4j export

### Milestone 8: Anomaly & Quality Framework (1-2 weeks effort)
1. Anomaly injection framework
2. Labeled anomaly dataset
3. Data quality variations
4. Validation and testing

---

## Configuration Example

```yaml
global:
  seed: 12345
  start_date: 2020-01-01
  period_months: 36  # 3 years of data
  group_currency: USD

enterprise:
  name: "Global Manufacturing Corp"
  industry: manufacturing

  # Legal entity hierarchy
  legal_entities:
    - code: "1000"
      name: "GMC Holdings (Parent)"
      country: US
      currency: USD
      is_parent: true

    - code: "1100"
      name: "GMC Americas"
      country: US
      currency: USD
      parent: "1000"
      ownership_percent: 100

    - code: "1200"
      name: "GMC Europe"
      country: DE
      currency: EUR
      parent: "1000"
      ownership_percent: 100

    - code: "1210"
      name: "GMC UK"
      country: GB
      currency: GBP
      parent: "1200"
      ownership_percent: 100

    - code: "1300"
      name: "GMC Asia Pacific"
      country: SG
      currency: SGD
      parent: "1000"
      ownership_percent: 100

    - code: "1310"
      name: "GMC Japan"
      country: JP
      currency: JPY
      parent: "1300"
      ownership_percent: 80  # Minority interest

master_data:
  vendors:
    count: 500
    intercompany_percent: 0.05
    distribution:
      strategic: 0.10
      preferred: 0.25
      standard: 0.50
      one_time: 0.15

  customers:
    count: 2000
    intercompany_percent: 0.05
    b2b_percent: 0.70

  materials:
    count: 5000
    raw_materials_percent: 0.40
    finished_goods_percent: 0.35
    services_percent: 0.25

  employees:
    count: 1500

  fixed_assets:
    count: 800

transactions:
  annual_volume_per_company: hundred_k

  document_flows:
    p2p_enabled: true
    o2c_enabled: true

  intercompany:
    enabled: true
    ic_revenue_percent: 0.15  # 15% of revenue is intercompany
    transfer_pricing_method: cost_plus
    markup_percent: 0.05

period_close:
  run_depreciation: true
  generate_accruals: true
  intercompany_settlement: true

consolidation:
  enabled: true
  eliminate_intercompany: true
  translate_currencies: true

anomalies:
  enabled: true
  total_rate: 0.02
  fraud_rate: 0.005
  error_rate: 0.01
  generate_labels: true

export:
  formats: [parquet, csv]
  graph_export:
    enabled: true
    formats: [pytorch_geometric, neo4j]
  subledger_detail: true
  labels_file: true
```

---

## Output Files

```
output/
├── master_data/
│   ├── vendors.parquet
│   ├── customers.parquet
│   ├── materials.parquet
│   ├── employees.parquet
│   ├── fixed_assets.parquet
│   └── cost_centers.parquet
│
├── transactions/
│   ├── journal_entries.parquet
│   ├── purchase_orders.parquet
│   ├── goods_receipts.parquet
│   ├── vendor_invoices.parquet
│   ├── payments.parquet
│   ├── sales_orders.parquet
│   ├── deliveries.parquet
│   ├── customer_invoices.parquet
│   ├── customer_receipts.parquet
│   └── document_references.parquet
│
├── subledgers/
│   ├── ar_open_items.parquet
│   ├── ar_aging.parquet
│   ├── ap_open_items.parquet
│   ├── ap_aging.parquet
│   ├── fa_register.parquet
│   ├── fa_depreciation.parquet
│   ├── inventory_positions.parquet
│   └── inventory_movements.parquet
│
├── period_close/
│   ├── trial_balances/
│   │   ├── 2020_01.parquet
│   │   ├── 2020_02.parquet
│   │   └── ...
│   ├── accruals.parquet
│   └── closing_entries.parquet
│
├── consolidation/
│   ├── eliminations.parquet
│   ├── currency_translation.parquet
│   ├── minority_interest.parquet
│   └── consolidated_trial_balance.parquet
│
├── controls/
│   ├── internal_controls.csv
│   ├── control_mappings.csv
│   ├── sod_rules.csv
│   └── control_test_results.parquet
│
├── graphs/
│   ├── pytorch_geometric/
│   │   ├── transaction_graph/
│   │   │   ├── node_features.pt
│   │   │   ├── edge_index.pt
│   │   │   ├── edge_attr.pt
│   │   │   └── labels.pt
│   │   └── approval_graph/
│   │       └── ...
│   └── neo4j/
│       ├── nodes.csv
│       ├── relationships.csv
│       └── import.cypher
│
└── labels/
    ├── anomaly_labels.parquet
    ├── fraud_labels.parquet
    └── quality_issues.parquet
```

---

## Validation Criteria

### Coherence Checks
- [ ] All transactions reference existing master data entities
- [ ] Document references form valid chains
- [ ] Trial balance balances (debits = credits)
- [ ] Subledgers reconcile to GL
- [ ] Intercompany balances match between entities
- [ ] FX rates are consistent across transactions

### Statistical Checks
- [ ] Amount distributions match Benford's Law (where applicable)
- [ ] Seasonal patterns visible in time series
- [ ] Account balance ratios within industry norms
- [ ] Payment terms reflected in aging

### Business Logic Checks
- [ ] Depreciation correctly calculated
- [ ] Tax provisions reasonable
- [ ] Intercompany margins within transfer pricing rules
- [ ] Approval limits respected

### ML Readiness Checks
- [ ] Labels correctly applied
- [ ] Graph structure is valid
- [ ] Features are properly normalized
- [ ] Train/validation/test splits are temporally separated

---

## Conclusion

This enhancement plan transforms the synthetic data generator into a comprehensive enterprise simulation engine. The resulting dataset will be:

1. **Coherent**: All entities, documents, and transactions are interconnected
2. **Complete**: Full document lifecycle from order to payment/collection
3. **Realistic**: Statistical properties match real enterprise data
4. **Labeled**: Ground truth available for supervised learning
5. **Multi-format**: Ready for analytics, ML, and graph analysis

The implementation follows a modular architecture where each phase builds upon the previous, ensuring testability and incremental value delivery.

---

# Part 2: RustAssureTwin Integration Gap Analysis

## Executive Summary

RustAssureTwin is an AI-assisted audit intelligence platform that requires comprehensive synthetic data for development and testing. This analysis identifies gaps between current SyntheticData capabilities and RustAssureTwin requirements based on the 22 specification documents.

### RustAssureTwin Three-Layer Architecture

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                    RUSTASSURETWIN KNOWLEDGE GRAPH                            │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                              │
│  LAYER 3: INTERNAL CONTROL SYSTEM (ICS) - Audit & Compliance                │
│  ┌─────────────────────────────────────────────────────────────────────┐    │
│  │ Framework, ControlObjective, ControlActivity, Risk, Policy,         │    │
│  │ TestResult, Issue, RemediationPlan, Evidence, AuditEngagement,     │    │
│  │ Workpaper, Finding, ProfessionalJudgment                           │    │
│  └─────────────────────────────────────────────────────────────────────┘    │
│                              ▲                                              │
│                              │ Controls monitor                             │
│                              │                                              │
│  LAYER 2: OBJECT-CENTRIC PROCESS MINING (OCPM) - Process Intelligence      │
│  ┌─────────────────────────────────────────────────────────────────────┐    │
│  │ ObjectType, Object, ActivityType, Event, Process, Variant,         │    │
│  │ Resource, ProcessModel, Perspective                                │    │
│  └─────────────────────────────────────────────────────────────────────┘    │
│                              ▲                                              │
│                              │ Events create                                │
│                              │                                              │
│  LAYER 1: ACCOUNTING NETWORKS - Transaction Intelligence                    │
│  ┌─────────────────────────────────────────────────────────────────────┐    │
│  │ Account, Transaction, GLEntry, Balance, Document, LegalEntity,     │    │
│  │ CostCenter, Segment, Period, Currency, ExchangeRate                │    │
│  └─────────────────────────────────────────────────────────────────────┘    │
│                                                                              │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## Gap Analysis Summary

### Coverage by Layer

| Layer | Current Coverage | Gap Level | Priority |
|-------|-----------------|-----------|----------|
| Layer 1: Accounting | ✅ ~90% | Low | Minor enhancements |
| Layer 2: OCPM | ⚠️ ~40% | Medium | Complete synth-ocpm |
| Layer 3: ICS | ⚠️ ~30% | High | Major new development |
| Bi-temporal Support | ❌ 0% | High | Critical foundation |
| Predictive Analytics | ❌ 0% | Medium | New entity types |

---

## Layer 1: Accounting Networks - Detailed Gap Analysis

### Currently Supported ✅

| Entity | Status | Notes |
|--------|--------|-------|
| Account (ChartOfAccounts) | ✅ Complete | Full hierarchy with account types |
| Transaction (JournalEntry) | ✅ Complete | Balanced entries with all metadata |
| GLEntry (LineItem) | ✅ Complete | Debit/credit with cost objects |
| Balance | ✅ Complete | BalanceTracker, TrialBalance |
| Document | ✅ Complete | P2P, O2C document flows |
| LegalEntity (Company) | ✅ Complete | Multi-company with IC relationships |
| CostCenter | ✅ Complete | Master data generation |
| Period | ✅ Complete | FiscalPeriod with close status |
| Currency | ✅ Complete | FxRateService with rates |
| ExchangeRate | ✅ Complete | Spot, average, closing rates |

### Gaps to Address

| Gap | Description | Priority |
|-----|-------------|----------|
| Segment | Business segment dimension for reporting | Medium |
| Bi-temporal timestamps | valid_from/valid_to, recorded_at/superseded_at | High |
| Enhanced Document metadata | Reliability assessment per ISA 500 | Medium |

---

## Layer 2: OCPM - Detailed Gap Analysis

### Current State: synth-ocpm Crate (Started)

The `crates/synth-ocpm/` crate has been created with model definitions:

| Model | Status | Notes |
|-------|--------|-------|
| ObjectType | ✅ Defined | PurchaseOrder, SalesOrder, Invoice, etc. |
| ObjectInstance | ✅ Defined | Object with lifecycle state |
| OcpmEvent | ✅ Defined | Event with timestamp, activity, objects |
| EventLog | ✅ Defined | OCEL 2.0 compatible structure |
| ActivityType | ✅ Defined | P2P and O2C activities defined |
| ProcessVariant | ✅ Defined | Case traces with statistics |
| Resource | ✅ Defined | Human and system resources |

### Gaps to Address

| Gap | Description | Priority |
|-----|-------------|----------|
| OcpmEventGenerator | Generate events from document flows | High |
| ProcessModel | Process model definitions (BPMN-like) | Medium |
| Perspective | Different analytical views | Low |
| Variant Generator | Generate realistic process variants | High |
| Resource Assignment | Link events to resources | Medium |
| OCEL 2.0 Export | Export in standard OCEL format | High |

---

## Layer 3: ICS - Detailed Gap Analysis

### Current State

| Entity | Status | Notes |
|--------|--------|-------|
| InternalControl | ✅ Partial | Basic SOX controls defined |
| ControlMapping | ✅ Partial | Account/process mappings |
| SoDConflict | ✅ Partial | Conflict types defined |

### Major Gaps (New Development Required)

#### 3.1 Audit Engagement Entities

```rust
// NEW: Audit engagement structure
pub struct AuditEngagement {
    pub engagement_id: String,
    pub client_entity: String,
    pub engagement_type: EngagementType,  // Annual, Interim, SOX, SpecialPurpose
    pub fiscal_year: u16,
    pub planning_start: NaiveDate,
    pub fieldwork_start: NaiveDate,
    pub report_date: NaiveDate,
    pub materiality: Decimal,
    pub performance_materiality: Decimal,
    pub clearly_trivial: Decimal,
    pub engagement_partner: String,
    pub engagement_manager: String,
    pub team_members: Vec<String>,
    pub status: EngagementStatus,
}

pub enum EngagementType {
    AnnualAudit,
    InterimAudit,
    Sox404,
    ReviewEngagement,
    CompilatonEngagement,
    AgreedUponProcedures,
}
```

#### 3.2 Evidence Management

```rust
// NEW: Evidence per ISA 500
pub struct AuditEvidence {
    pub evidence_id: String,
    pub engagement_id: String,
    pub evidence_type: EvidenceType,
    pub source_type: EvidenceSource,
    pub title: String,
    pub description: String,
    pub obtained_date: NaiveDate,
    pub obtained_by: String,
    pub file_hash: String,

    // Reliability assessment per ISA 500.A31
    pub reliability_assessment: ReliabilityAssessment,

    // Relevance mapping
    pub assertions_addressed: Vec<Assertion>,
    pub accounts_impacted: Vec<String>,
    pub linked_workpapers: Vec<String>,

    // AI extraction (optional)
    pub ai_extracted_terms: Option<HashMap<String, String>>,
    pub ai_confidence: Option<f64>,
}

pub struct ReliabilityAssessment {
    pub independence_of_source: ReliabilityLevel,
    pub effectiveness_of_controls: ReliabilityLevel,
    pub qualifications_of_provider: ReliabilityLevel,
    pub objectivity_of_provider: ReliabilityLevel,
    pub overall_reliability: ReliabilityLevel,
    pub notes: String,
}

pub enum EvidenceSource {
    ExternalThirdParty,
    ExternalClientProvided,
    InternalClientPrepared,
    AuditorPrepared,
}
```

#### 3.3 Workpaper Structure

```rust
// NEW: Workpaper per ISA 230
pub struct Workpaper {
    pub workpaper_id: String,
    pub engagement_id: String,
    pub title: String,
    pub section: WorkpaperSection,
    pub objective: String,
    pub assertions_tested: Vec<Assertion>,
    pub procedure_performed: String,
    pub scope: WorkpaperScope,
    pub population_size: u64,
    pub sample_size: u32,
    pub results_summary: String,
    pub exceptions_found: u32,
    pub conclusion: WorkpaperConclusion,
    pub preparer: String,
    pub preparer_date: NaiveDate,
    pub reviewer: Option<String>,
    pub reviewer_date: Option<NaiveDate>,
    pub status: WorkpaperStatus,
    pub evidence_refs: Vec<String>,
    pub cross_references: Vec<String>,
    pub version: u32,
}

pub enum WorkpaperSection {
    Planning,
    RiskAssessment,
    ControlTesting,
    SubstantiveTesting,
    Completion,
    Reporting,
}
```

#### 3.4 Professional Judgment

```rust
// NEW: Judgment documentation per ISA 200
pub struct ProfessionalJudgment {
    pub judgment_id: String,
    pub engagement_id: String,
    pub judgment_type: JudgmentType,
    pub subject: String,
    pub applicable_standards: Vec<String>,

    // Structured documentation
    pub issue_description: String,
    pub information_considered: Vec<InformationItem>,
    pub alternatives_evaluated: Vec<AlternativeEvaluation>,
    pub skepticism_applied: SkepticismDocumentation,
    pub conclusion: String,
    pub rationale: String,
    pub residual_risk: String,

    // Sign-offs
    pub preparer: String,
    pub preparer_date: NaiveDate,
    pub reviewer: Option<String>,
    pub reviewer_date: Option<NaiveDate>,
    pub partner_concurrence: Option<String>,
}

pub enum JudgmentType {
    MaterialityDetermination,
    RiskAssessment,
    ControlEvaluation,
    EstimateEvaluation,
    GoingConcern,
    MisstatementEvaluation,
    ReportingDecision,
    SamplingDesign,
}
```

#### 3.5 Risk Assessment

```rust
// NEW: Risk entities per ISA 315/330
pub struct RiskAssessment {
    pub risk_id: String,
    pub engagement_id: String,
    pub risk_category: RiskCategory,
    pub account_or_process: String,
    pub assertion: Option<Assertion>,
    pub inherent_risk: RiskLevel,
    pub control_risk: RiskLevel,
    pub risk_of_material_misstatement: RiskLevel,
    pub is_significant_risk: bool,
    pub fraud_risk_factors: Vec<FraudRiskFactor>,
    pub planned_response: Vec<String>,
    pub assessed_by: String,
    pub assessed_date: NaiveDate,
}

pub enum RiskCategory {
    FinancialStatementLevel,
    AssertionLevel,
    FraudRisk,
    GoingConcern,
    RelatedParty,
    EstimateRisk,
    ItGeneralControl,
}

pub struct FraudRiskFactor {
    pub factor_type: FraudTriangleElement,  // Opportunity, Pressure, Rationalization
    pub indicator: String,
    pub score: u8,  // 0-100
    pub trend: Trend,
    pub source: String,
}
```

#### 3.6 Finding and Issue Management

```rust
// NEW: Audit findings
pub struct AuditFinding {
    pub finding_id: String,
    pub engagement_id: String,
    pub finding_type: FindingType,
    pub severity: FindingSeverity,
    pub title: String,
    pub condition: String,      // What we found
    pub criteria: String,       // What it should be
    pub cause: String,          // Why it happened
    pub effect: String,         // What's the impact
    pub recommendation: String,
    pub management_response: Option<String>,
    pub remediation_plan: Option<RemediationPlan>,
    pub status: FindingStatus,
    pub workpaper_refs: Vec<String>,
    pub evidence_refs: Vec<String>,
}

pub enum FindingType {
    MaterialWeakness,
    SignificantDeficiency,
    ControlDeficiency,
    MaterialMisstatement,
    OtherMatter,
    ComplianceException,
}

pub struct RemediationPlan {
    pub plan_id: String,
    pub finding_id: String,
    pub description: String,
    pub responsible_party: String,
    pub target_date: NaiveDate,
    pub status: RemediationStatus,
    pub validation_approach: String,
    pub validated_by: Option<String>,
    pub validated_date: Option<NaiveDate>,
}
```

---

## Bi-Temporal Data Model - Critical Foundation

### Requirement

RustAssureTwin requires bi-temporal data for complete audit trails:

```rust
// NEW: Bi-temporal wrapper for all auditable entities
pub struct BiTemporal<T> {
    pub data: T,

    // Business validity (when the fact is true in the real world)
    pub valid_from: NaiveDateTime,
    pub valid_to: Option<NaiveDateTime>,  // None = current

    // System recording (when we recorded this in the system)
    pub recorded_at: NaiveDateTime,
    pub superseded_at: Option<NaiveDateTime>,  // None = current version

    // Audit metadata
    pub recorded_by: String,
    pub change_reason: Option<String>,
}

// Example: BiTemporal<JournalEntry> allows:
// - Point-in-time queries: "What was balance as of Dec 31?"
// - Audit trail queries: "What did we know on Jan 15 about Dec 31?"
// - Correction tracking: "Show me all versions of this entry"
```

### Implementation Impact

| Area | Change Required |
|------|-----------------|
| synth-core models | Add temporal fields to all entities |
| Generators | Generate temporal histories |
| Output sinks | Include temporal columns |
| Query support | Temporal query functions |

---

## Predictive Analytics Entities

### New Entity Types Required

```rust
// NEW: Forecast and prediction support
pub struct Forecast {
    pub forecast_id: String,
    pub model_id: String,
    pub target_entity: String,
    pub metric_type: MetricType,
    pub forecast_date: NaiveDate,
    pub horizon_days: u32,
    pub point_estimate: Decimal,
    pub confidence_interval_lower: Decimal,
    pub confidence_interval_upper: Decimal,
    pub confidence_level: f64,  // e.g., 0.95
    pub assumptions: Vec<ForecastAssumption>,
    pub actual_value: Option<Decimal>,  // Filled when period closes
}

pub struct RiskAlert {
    pub alert_id: String,
    pub alert_type: RiskAlertType,
    pub severity: AlertSeverity,
    pub probability: f64,
    pub potential_impact_low: Decimal,
    pub potential_impact_high: Decimal,
    pub contributing_signals: Vec<RiskSignal>,
    pub generated_at: NaiveDateTime,
    pub status: AlertStatus,
    pub recommended_actions: Vec<String>,
}

pub struct RiskSignal {
    pub signal_id: String,
    pub signal_type: SignalType,  // Leading, Concurrent, Lagging
    pub category: SignalCategory,
    pub metric_id: String,
    pub current_value: f64,
    pub baseline_value: f64,
    pub deviation_sigma: f64,
    pub confidence: f64,
    pub detected_at: NaiveDateTime,
}

pub struct ControlHealth {
    pub health_id: String,
    pub control_id: String,
    pub current_score: u8,  // 0-100
    pub trend: HealthTrend,
    pub failure_probability: f64,
    pub predicted_failure_date: Option<NaiveDate>,
    pub degradation_factors: Vec<String>,
    pub last_assessed: NaiveDateTime,
}
```

---

## Implementation Plan for RustAssureTwin Integration

### Phase 11: Bi-Temporal Foundation

**Duration**: 1-2 weeks

| Task | Description |
|------|-------------|
| 11.1 | Define `BiTemporal<T>` wrapper |
| 11.2 | Add temporal fields to JournalEntry |
| 11.3 | Add temporal fields to master data |
| 11.4 | Create temporal history generator |
| 11.5 | Update output sinks for temporal columns |

### Phase 12: OCPM Event Generation

**Duration**: 2-3 weeks

| Task | Description |
|------|-------------|
| 12.1 | Create OcpmEventGenerator |
| 12.2 | Generate events from P2P document flow |
| 12.3 | Generate events from O2C document flow |
| 12.4 | Create process variant generator |
| 12.5 | Implement OCEL 2.0 export format |
| 12.6 | Link events to resources |

### Phase 13: Audit Engagement Framework

**Duration**: 3-4 weeks

| Task | Description |
|------|-------------|
| 13.1 | Define engagement models |
| 13.2 | Create engagement generator |
| 13.3 | Define workpaper structure |
| 13.4 | Create workpaper generator |
| 13.5 | Define evidence models |
| 13.6 | Create evidence generator |
| 13.7 | Implement cross-referencing |

### Phase 14: Risk and Judgment Framework

**Duration**: 2-3 weeks

| Task | Description |
|------|-------------|
| 14.1 | Define risk assessment models |
| 14.2 | Create risk assessment generator |
| 14.3 | Define judgment models |
| 14.4 | Create judgment generator |
| 14.5 | Define finding models |
| 14.6 | Create finding generator |
| 14.7 | Link findings to workpapers/evidence |

### Phase 15: Predictive Analytics Entities

**Duration**: 2 weeks

| Task | Description |
|------|-------------|
| 15.1 | Define forecast models |
| 15.2 | Create forecast generator |
| 15.3 | Define risk alert models |
| 15.4 | Create alert generator |
| 15.5 | Define control health models |
| 15.6 | Create control health generator |

### Phase 16: Integration Testing

**Duration**: 1-2 weeks

| Task | Description |
|------|-------------|
| 16.1 | Cross-layer relationship validation |
| 16.2 | Temporal consistency checks |
| 16.3 | OCEL 2.0 format validation |
| 16.4 | ISA compliance checks |
| 16.5 | Performance benchmarking |

---

## Enhanced Configuration Schema

```yaml
# New configuration sections for RustAssureTwin

audit_engagement:
  enabled: true
  engagement_type: annual_audit
  fiscal_year: 2025
  materiality:
    basis: total_revenue
    percentage: 0.005  # 0.5%
    performance_materiality_factor: 0.75
    clearly_trivial_factor: 0.05

  team:
    engagement_partner: "J. Williams"
    engagement_manager: "S. Patel"
    senior_count: 3
    staff_count: 5

  timeline:
    planning_weeks: 4
    fieldwork_weeks: 6
    completion_weeks: 2

workpapers:
  enabled: true
  sections:
    planning:
      count_range: [15, 25]
    risk_assessment:
      count_range: [20, 35]
    control_testing:
      count_range: [30, 50]
    substantive_testing:
      count_range: [40, 70]
    completion:
      count_range: [10, 20]

  review_rates:
    first_review_complete: 0.95
    second_review_complete: 0.85

  exception_rate: 0.08  # 8% of tests find exceptions

evidence:
  enabled: true
  categories:
    external_third_party: 0.25
    external_client: 0.35
    internal_client: 0.30
    auditor_prepared: 0.10

  ai_extraction_rate: 0.40  # 40% have AI-extracted terms
  reliability_distribution:
    high: 0.40
    medium: 0.45
    low: 0.15

professional_judgment:
  enabled: true
  types:
    materiality: 1
    risk_assessment: 5
    control_evaluation: 8
    estimate_evaluation: 4
    going_concern: 1
    misstatement_evaluation: 2

  consultation_rate: 0.15  # 15% require consultation

risk_assessment:
  enabled: true
  significant_risk_accounts: 5
  fraud_risk_factors:
    opportunity_indicators: 8
    pressure_indicators: 6
    rationalization_indicators: 4

  fraud_triangle_scoring: true

findings:
  enabled: true
  distribution:
    material_weakness: 0.01
    significant_deficiency: 0.05
    control_deficiency: 0.15
    compliance_exception: 0.10
    other_matter: 0.05

  remediation_rate: 0.70  # 70% have remediation plans

ocpm:
  enabled: true
  event_generation:
    from_p2p: true
    from_o2c: true
    from_period_close: true

  process_variants:
    happy_path_rate: 0.75
    exception_path_rate: 0.20
    error_path_rate: 0.05

  export_format: ocel2  # OCEL 2.0 standard

temporal:
  enabled: true
  bi_temporal: true
  correction_rate: 0.02  # 2% of entries have corrections
  late_posting_rate: 0.05  # 5% posted in subsequent period

predictive:
  enabled: true
  forecasts:
    revenue: true
    expense: true
    cash_flow: true

  risk_alerts:
    fraud_risk: true
    control_failure: true
    going_concern: true

  control_health:
    monitor_all_key_controls: true
    degradation_simulation: true
```

---

## Extended Output Files

```
output/
├── ... (existing structure)
│
├── audit/
│   ├── engagements.csv
│   ├── workpapers.csv
│   ├── evidence.csv
│   ├── evidence_reliability.csv
│   ├── cross_references.csv
│   ├── professional_judgments.csv
│   ├── risk_assessments.csv
│   ├── fraud_risk_factors.csv
│   ├── findings.csv
│   ├── remediation_plans.csv
│   └── sign_offs.csv
│
├── ocpm/
│   ├── ocel2/
│   │   ├── object_types.json
│   │   ├── objects.json
│   │   ├── events.json
│   │   └── ocel2_complete.jsonocel
│   ├── process_variants.csv
│   ├── case_traces.csv
│   └── resources.csv
│
├── temporal/
│   ├── entity_history/
│   │   ├── journal_entry_history.csv
│   │   ├── vendor_history.csv
│   │   └── ...
│   ├── corrections.csv
│   └── temporal_index.csv
│
└── predictive/
    ├── forecasts.csv
    ├── risk_alerts.csv
    ├── risk_signals.csv
    ├── control_health.csv
    └── trend_analysis.csv
```

---

## Validation Criteria for RustAssureTwin

### Layer Integration Checks
- [ ] Events link to documents (Layer 2 → Layer 1)
- [ ] Controls link to accounts and processes (Layer 3 → Layers 1,2)
- [ ] Evidence links to workpapers and findings
- [ ] Risk assessments link to accounts and assertions
- [ ] Judgments link to engagements and workpapers

### ISA Compliance Checks
- [ ] Evidence meets ISA 500 sufficiency criteria
- [ ] Workpapers meet ISA 230 documentation requirements
- [ ] Risk assessments align with ISA 315
- [ ] Findings follow ISA 265 classification
- [ ] Materiality follows ISA 320 guidance

### Temporal Consistency Checks
- [ ] valid_from < valid_to for all temporal records
- [ ] recorded_at is consistent with business timeline
- [ ] Superseded records have complete chains
- [ ] No orphaned temporal records

### OCEL 2.0 Compliance
- [ ] Valid JSON structure per OCEL 2.0 spec
- [ ] All events have timestamps
- [ ] Object references are valid
- [ ] Activity types are consistent

---

## Conclusion

This gap analysis identifies the extensions needed to make SyntheticData a comprehensive data source for RustAssureTwin. The key additions are:

1. **Bi-temporal foundation** - Critical for audit trail requirements
2. **OCPM event generation** - Complete the Layer 2 model
3. **Audit engagement framework** - New Layer 3 entities
4. **Risk and judgment documentation** - ISA-compliant structures
5. **Predictive analytics entities** - Support for AI features

The implementation builds upon the solid foundation of Phases 1-10, adding audit-specific capabilities while maintaining coherence with existing transaction data.
