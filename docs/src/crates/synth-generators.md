# synth-generators

Data generators for journal entries, master data, document flows, and anomalies.

## Overview

`synth-generators` contains all data generation logic for SyntheticData:

- **Core Generators**: Journal entries, chart of accounts, users
- **Master Data**: Vendors, customers, materials, assets, employees
- **Document Flows**: P2P (Procure-to-Pay), O2C (Order-to-Cash)
- **Financial**: Intercompany, balance tracking, subledgers, FX, period close
- **Quality**: Anomaly injection, data quality variations

## Module Structure

### Core Generators

| Generator | Description |
|-----------|-------------|
| `je_generator` | Journal entry generation with statistical distributions |
| `coa_generator` | Chart of accounts with industry-specific structures |
| `company_selector` | Weighted company selection for transactions |
| `user_generator` | User/persona generation with roles |
| `control_generator` | Internal controls and SoD rules |

### Master Data (`master_data/`)

| Generator | Description |
|-----------|-------------|
| `vendor_generator` | Vendors with payment terms, bank accounts, behaviors |
| `customer_generator` | Customers with credit ratings, payment patterns |
| `material_generator` | Materials/products with BOM, valuations |
| `asset_generator` | Fixed assets with depreciation schedules |
| `employee_generator` | Employees with manager hierarchy |
| `entity_registry_manager` | Central entity registry with temporal validity |

### Document Flow (`document_flow/`)

| Generator | Description |
|-----------|-------------|
| `p2p_generator` | PO → GR → Invoice → Payment flow |
| `o2c_generator` | SO → Delivery → Invoice → Receipt flow |
| `document_chain_manager` | Reference chain management |
| `document_flow_je_generator` | Generate JEs from document flows |
| `three_way_match` | PO/GR/Invoice matching validation |

### Intercompany (`intercompany/`)

| Generator | Description |
|-----------|-------------|
| `ic_generator` | Matched intercompany entry pairs |
| `matching_engine` | IC matching and reconciliation |
| `elimination_generator` | Consolidation elimination entries |

### Balance (`balance/`)

| Generator | Description |
|-----------|-------------|
| `opening_balance_generator` | Coherent opening balance sheet |
| `balance_tracker` | Running balance validation |
| `trial_balance_generator` | Period-end trial balance |

### Subledger (`subledger/`)

| Generator | Description |
|-----------|-------------|
| `ar_generator` | AR invoices, receipts, credit memos, aging |
| `ap_generator` | AP invoices, payments, debit memos |
| `fa_generator` | Fixed assets, depreciation, disposals |
| `inventory_generator` | Inventory positions, movements, valuation |
| `reconciliation` | GL-to-subledger reconciliation |

### FX (`fx/`)

| Generator | Description |
|-----------|-------------|
| `fx_rate_service` | FX rate generation (Ornstein-Uhlenbeck process) |
| `currency_translator` | Trial balance translation |
| `cta_generator` | Currency Translation Adjustment entries |

### Period Close (`period_close/`)

| Generator | Description |
|-----------|-------------|
| `close_engine` | Main orchestration |
| `accruals` | Accrual entry generation |
| `depreciation` | Monthly depreciation runs |
| `year_end` | Year-end closing entries |

### Anomaly (`anomaly/`)

| Generator | Description |
|-----------|-------------|
| `injector` | Main anomaly injection engine |
| `types` | Weighted anomaly type configurations |
| `strategies` | Injection strategies (amount, date, duplication) |
| `patterns` | Temporal patterns, clustering, entity targeting |

### Data Quality (`data_quality/`)

| Generator | Description |
|-----------|-------------|
| `injector` | Main data quality injector |
| `missing_values` | MCAR, MAR, MNAR, Systematic patterns |
| `format_variations` | Date, amount, identifier formats |
| `duplicates` | Exact, near, fuzzy duplicates |
| `typos` | Keyboard-aware typos, OCR errors |

## Usage Examples

### Journal Entry Generation

```rust
use synth_generators::je_generator::JournalEntryGenerator;

let mut generator = JournalEntryGenerator::new(config, seed);

// Generate batch
let entries = generator.generate_batch(1000)?;

// Stream generation
for entry in generator.generate_stream().take(1000) {
    process(entry?);
}
```

### Master Data Generation

```rust
use synth_generators::master_data::{VendorGenerator, CustomerGenerator};

let mut vendor_gen = VendorGenerator::new(seed);
let vendors = vendor_gen.generate(100);

let mut customer_gen = CustomerGenerator::new(seed);
let customers = customer_gen.generate(200);
```

### Document Flow Generation

```rust
use synth_generators::document_flow::{P2pGenerator, O2cGenerator};

let mut p2p = P2pGenerator::new(config, seed);
let p2p_flows = p2p.generate_batch(500)?;

let mut o2c = O2cGenerator::new(config, seed);
let o2c_flows = o2c.generate_batch(500)?;
```

### Anomaly Injection

```rust
use synth_generators::anomaly::AnomalyInjector;

let mut injector = AnomalyInjector::new(config.anomaly_injection, seed);

// Inject into existing entries
let (modified_entries, labels) = injector.inject(&entries)?;
```

## Three-Way Match

The P2P generator validates document matching:

```rust
use synth_generators::document_flow::ThreeWayMatch;

let match_result = ThreeWayMatch::validate(
    &purchase_order,
    &goods_receipt,
    &vendor_invoice,
    tolerance_config,
);

match match_result {
    MatchResult::Passed => { /* Process normally */ }
    MatchResult::QuantityVariance(var) => { /* Handle variance */ }
    MatchResult::PriceVariance(var) => { /* Handle variance */ }
}
```

## Balance Coherence

The balance tracker maintains accounting equation:

```rust
use synth_generators::balance::BalanceTracker;

let mut tracker = BalanceTracker::new();

for entry in &entries {
    tracker.post(&entry)?;
}

// Verify Assets = Liabilities + Equity
assert!(tracker.is_balanced());
```

## FX Rate Generation

Uses Ornstein-Uhlenbeck process for realistic rate movements:

```rust
use synth_generators::fx::FxRateService;

let mut fx_service = FxRateService::new(config.fx, seed);

// Get rate for date
let rate = fx_service.get_rate("EUR", "USD", date)?;

// Generate daily rates
let rates = fx_service.generate_daily_rates(start, end)?;
```

## Anomaly Types

### Fraud Types
- FictitiousTransaction, RevenueManipulation, ExpenseCapitalization
- SplitTransaction, RoundTripping, KickbackScheme
- GhostEmployee, DuplicatePayment, UnauthorizedDiscount

### Error Types
- DuplicateEntry, ReversedAmount, WrongPeriod
- WrongAccount, MissingReference, IncorrectTaxCode

### Process Issues
- LatePosting, SkippedApproval, ThresholdManipulation
- MissingDocumentation, OutOfSequence

### Statistical Anomalies
- UnusualAmount, TrendBreak, BenfordViolation, OutlierValue

### Relational Anomalies
- CircularTransaction, DormantAccountActivity, UnusualCounterparty

## See Also

- [synth-core](synth-core.md)
- [Anomaly Injection](../advanced/anomaly-injection.md)
- [Document Flows](../configuration/document-flows.md)
