# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Build Commands

```bash
# Build release binary
cargo build --release

# Run all tests
cargo test

# Run tests for a specific crate
cargo test -p synth-core
cargo test -p synth-generators
cargo test -p synth-graph

# Run a single test by name
cargo test test_name

# Check code without building
cargo check

# Format code
cargo fmt

# Run clippy lints
cargo clippy
```

## CLI Usage

The binary is `synth-data` (located at `target/release/synth-data` after build).

```bash
# Generate with demo preset
synth-data generate --demo --output ./output

# Create industry-specific config
synth-data init --industry manufacturing --complexity medium -o config.yaml

# Validate config
synth-data validate --config config.yaml

# Generate from config
synth-data generate --config config.yaml --output ./output
```

## Architecture

This is a Rust workspace with 7 crates following a layered architecture:

```
synth-cli          → Binary entry point (commands: generate, validate, init, info)
    ↓
synth-runtime      → Orchestration layer (GenerationOrchestrator coordinates workflow)
    ↓
synth-generators   → Data generators (JE, Document Flows, Subledgers, Anomalies, etc.)
    ↓
synth-graph        → Graph/network export (PyTorch Geometric, Neo4j, DGL)
    ↓
synth-config       → Configuration schema, validation, industry presets
    ↓
synth-core         → Domain models, traits, statistical distributions, templates
    ↓
synth-output       → Output sinks (CSV, JSON, Parquet, ControlExport)
```

### Key Domain Models (synth-core/src/models/)

**Core Accounting:**

- **JournalEntry**: Header + balanced line items (debits must equal credits)
- **ChartOfAccounts**: Hierarchical GL accounts with AccountType/AccountSubType
- **ACDOCA**: SAP HANA Universal Journal format with ZSIM_ simulation fields

**Master Data:**

- **Vendor**: Enhanced with PaymentTerms, BankAccount, tax_id, intercompany flags, VendorBehavior
- **Customer**: Enhanced with CreditRating, credit_limit, PaymentBehavior, intercompany flags
- **Material**: Material/Product with BOM, valuation method, standard cost, account determination
- **FixedAsset**: Asset with depreciation schedule, useful life, acquisition/disposal tracking
- **Employee**: User with manager hierarchy, approval limits, system roles, transaction codes
- **EntityRegistry**: Central registry with temporal validity for all master data

**Document Flow:**

- **PurchaseOrder**: P2P flow starting document
- **GoodsReceipt**: Inventory receipt with GR/IR clearing
- **VendorInvoice**: AP invoice with three-way match support
- **Payment**: AP/AR payment documents
- **SalesOrder**: O2C flow starting document
- **Delivery**: Inventory issue with COGS recognition
- **CustomerInvoice**: AR invoice
- **CustomerReceipt**: AR receipt/cash application
- **DocumentReference**: Chain linking documents (FollowOn, Payment, Reversal)

**Intercompany:**

- **IntercompanyRelationship**: Parent-subsidiary with ownership percentage
- **ICTransactionType**: GoodsSale, ServiceProvided, Loan, Dividend, ManagementFee, Royalty
- **ICMatchedPair**: Matched IC entry pairs (seller/buyer)
- **TransferPricingMethod**: CostPlus, ResaleMinus, ComparableUncontrolled

**Balance & Subledger:**

- **AccountBalance**: Running balance with period tracking
- **TrialBalance**: Period-end trial balance structure
- **ARInvoice/ARReceipt/ARCreditMemo**: AR subledger records
- **APInvoice/APPayment/APDebitMemo**: AP subledger records
- **AssetRegister/DepreciationSchedule**: FA subledger
- **InventoryPosition/InventoryMovement**: Inventory subledger

**FX & Period Close:**

- **FxRate**: Exchange rate with rate type (Spot, Closing, Average)
- **CurrencyTranslation**: Foreign subsidiary translation
- **FiscalPeriod**: Period close status tracking
- **AccrualEntry**: Month-end accrual entries

**Anomalies & Quality:**

- **AnomalyType**: Fraud, Error, ProcessIssue, Statistical, Relational
- **LabeledAnomaly**: Anomaly with full metadata for ML training
- **QualityIssue**: Data quality issue record (missing, typo, duplicate, format)

**Controls:**

- **InternalControl**: SOX 404 control definitions with control types and assertions
- **ControlMapping**: Control-to-entity mappings (accounts, processes, thresholds)
- **SoD**: Segregation of Duties conflict types and violation records

### Generator Modules (synth-generators/src/)

**Core Generators:**

- `je_generator.rs`: Journal Entry generator
- `coa_generator.rs`: Chart of Accounts generator
- `company_selector.rs`: Weighted company selection
- `user_generator.rs`: User/persona generator
- `control_generator.rs`: Internal controls generator

**Master Data (master_data/):**

- `vendor_generator.rs`: Enhanced vendor generation
- `customer_generator.rs`: Enhanced customer generation
- `material_generator.rs`: Material/product generation
- `asset_generator.rs`: Fixed asset generation
- `employee_generator.rs`: Employee with hierarchy
- `entity_registry_manager.rs`: Central entity registry

**Document Flow (document_flow/):**

- `p2p_generator.rs`: Procure-to-Pay flow (PO → GR → Invoice → Payment)
- `o2c_generator.rs`: Order-to-Cash flow (SO → Delivery → Invoice → Receipt)
- `document_chain_manager.rs`: Document reference chain management
- `three_way_match.rs`: PO/GR/Invoice matching engine

**Intercompany (intercompany/):**

- `ic_generator.rs`: Generate matched IC JE pairs
- `matching_engine.rs`: IC matching and reconciliation
- `elimination_generator.rs`: Consolidation elimination entries

**Balance (balance/):**

- `opening_balance_generator.rs`: Coherent opening balance sheet
- `balance_tracker.rs`: Running balance tracker
- `trial_balance_generator.rs`: Period-end trial balance

**Subledger (subledger/):**

- `ar_generator.rs`: AR invoices, receipts, credit memos, aging
- `ap_generator.rs`: AP invoices, payments, debit memos
- `fa_generator.rs`: Fixed assets, depreciation, disposals
- `inventory_generator.rs`: Inventory positions, movements, valuation
- `reconciliation.rs`: GL-to-subledger reconciliation

**FX (fx/):**

- `fx_rate_service.rs`: FX rate generation (Ornstein-Uhlenbeck process)
- `currency_translator.rs`: Trial balance translation
- `cta_generator.rs`: Currency Translation Adjustment entries

**Period Close (period_close/):**

- `close_engine.rs`: Main orchestration
- `accruals.rs`: Accrual entry generation
- `depreciation.rs`: Monthly depreciation runs
- `year_end.rs`: Year-end closing entries

**Anomaly (anomaly/):**

- `injector.rs`: Main anomaly injection engine
- `types.rs`: Weighted anomaly type configurations
- `strategies.rs`: Injection strategies (amount, date, duplication, approval)
- `patterns.rs`: Temporal patterns, clustering, entity targeting

**Data Quality (data_quality/):**

- `injector.rs`: Main data quality injector
- `missing_values.rs`: MCAR, MAR, MNAR, Systematic missing patterns
- `format_variations.rs`: Date, amount, identifier format variations
- `duplicates.rs`: Exact, near, fuzzy duplicate generation
- `typos.rs`: Keyboard-aware typos, OCR errors, homophones

### Graph Module (synth-graph/src/)

**Models:**

- `nodes.rs`: Node types (Account, Entity, User, Transaction)
- `edges.rs`: Edge types (Transaction, Approval, Ownership)
- `graph.rs`: Graph container with node/edge collections

**Builders:**

- `transaction_graph.rs`: Accounts/entities as nodes, transactions as edges
- `approval_graph.rs`: Users as nodes, approvals as edges
- `entity_graph.rs`: Legal entities with ownership edges

**Exporters:**

- `pytorch_geometric.rs`: .pt files (node_features, edge_index, edge_attr, masks)
- `neo4j.rs`: CSV files with Cypher import scripts
- `dgl.rs`: Deep Graph Library format

**ML:**

- `features.rs`: Feature computation (temporal, amount, structural, categorical)
- `splits.rs`: Train/validation/test split generation

### Statistical Distributions (synth-core/src/distributions/)

- **LineItemSampler**: Empirical distribution (60.68% two-line entries, 88% even line counts)
- **AmountSampler**: Log-normal with round-number bias, Benford's Law compliance
- **TemporalSampler**: Seasonality patterns with industry and holiday integration
- **BenfordSampler**: First-digit distribution following Benford's Law P(d) = log10(1 + 1/d)
- **FraudAmountGenerator**: Suspicious amount patterns (threshold-adjacent, round numbers)
- **IndustrySeasonality**: Industry-specific volume patterns for 10 sectors
- **HolidayCalendar**: Regional holidays for US, DE, GB, CN, JP, IN

### Core Traits (synth-core/src/traits/)

- **Generator**: `generate_batch()` and `generate_stream()` for data generation
- **Sink**: Output destination interface (CSV, JSON, Parquet implementations)

## Key Design Decisions

1. **Deterministic RNG**: Uses ChaCha8 with configurable seed for reproducible output
2. **Precise Decimals**: `rust_decimal` for financial calculations (no floating point)
3. **Balanced Entries**: JournalEntry enforces debits = credits at construction time
4. **Empirical Distributions**: Based on academic research on real GL data patterns
5. **Benford's Law**: Amount distribution follows first-digit law with fraud pattern exceptions
6. **Weighted Company Selection**: Companies selected based on volume_weight
7. **Document Chain Integrity**: All documents maintain proper reference chains
8. **Balance Coherence**: Running balance tracker validates Assets = Liabilities + Equity
9. **Subledger Reconciliation**: Automatic GL-to-subledger control account reconciliation
10. **ML-Ready Output**: Graph exports with train/val/test splits and computed features

## Configuration Schema

Config files use YAML with sections:

**Core:** `global`, `companies`, `chart_of_accounts`, `transactions`, `output`

**Compliance:** `fraud`, `internal_controls`

**Enterprise:** `enterprise`, `master_data`, `document_flows`, `intercompany`

**Financial:** `balance`, `subledger`, `fx`, `period_close`

**ML/Analytics:** `graph_export`, `anomaly_injection`, `data_quality`

**Supporting:** `business_processes`, `templates`, `approval`, `departments`

Industry presets: manufacturing, retail, financial_services, healthcare, technology
Complexity levels: small (~100 accounts), medium (~400), large (~2500)

## Anomaly Injection Framework

### Anomaly Categories

**Fraud Types (20+):**

- FictitiousTransaction, RevenueManipulation, ExpenseCapitalization
- SplitTransaction, RoundTripping, KickbackScheme, GhostEmployee
- DuplicatePayment, SuspenseAccountAbuse, UnauthorizedDiscount

**Error Types:**

- DuplicateEntry, ReversedAmount, WrongPeriod, WrongAccount
- MissingReference, IncorrectTaxCode, Misclassification

**Process Issues:**

- LatePosting, SkippedApproval, ThresholdManipulation
- MissingDocumentation, OutOfSequence

**Statistical Anomalies:**

- UnusualAmount, TrendBreak, BenfordViolation, OutlierValue

**Relational Anomalies:**

- CircularTransaction, DormantAccountActivity, UnusualCounterparty

### Injection Features

- Configurable rates per category
- Temporal patterns (year-end spikes)
- Anomaly clustering (realistic batch patterns)
- Entity targeting (random, repeat offender, volume-weighted)
- Full labeling for supervised learning

## Data Quality Variations

### Missing Value Strategies

- **MCAR**: Missing Completely At Random (equal probability)
- **MAR**: Missing At Random (depends on other observed values)
- **MNAR**: Missing Not At Random (depends on value itself)
- **Systematic**: Entire field groups missing together

### Format Variations

- **Dates**: ISO (2024-01-15), US (01/15/2024), EU (15.01.2024), Long (January 15, 2024)
- **Amounts**: Plain, US comma (1,234.56), EU format (1.234,56), Currency prefix/suffix
- **Identifiers**: Case variations, padding, separator variations

### Typo Generation

- Keyboard-aware substitution (QWERTY layout)
- Transposition, insertion, deletion
- OCR errors (0/O, 1/l, 5/S confusion)
- Homophones (their/there, affect/effect)

### Encoding Issues

- Mojibake (UTF-8/Latin-1 confusion)
- Missing characters, BOM issues
- HTML entity corruption

## Graph Export Formats

### PyTorch Geometric

```
output/graphs/transaction_network/pytorch_geometric/
├── node_features.pt    # [num_nodes, num_features]
├── edge_index.pt       # [2, num_edges]
├── edge_attr.pt        # [num_edges, num_edge_features]
├── labels.pt           # [num_nodes] or [num_edges]
├── train_mask.pt       # Boolean mask
├── val_mask.pt
└── test_mask.pt
```

### Neo4j

```
output/graphs/entity_relationship/neo4j/
├── nodes_account.csv
├── nodes_entity.csv
├── edges_transaction.csv
├── edges_ownership.csv
└── import.cypher
```

### ML Features

**Temporal:** weekday, period, is_month_end, is_quarter_end, is_year_end

**Amount:** log(amount), benford_probability, is_round_number

**Structural:** line_count, unique_accounts, has_intercompany

**Categorical:** business_process (one-hot), source_type (one-hot)

## Export Files

### Transaction Data

- `journal_entries.parquet` / `.csv` / `.json`
- `acdoca.parquet` - SAP HANA Universal Journal format

### Master Data

- `vendors.parquet`, `customers.parquet`
- `materials.parquet`, `fixed_assets.parquet`
- `employees.parquet`, `cost_centers.parquet`

### Document Flow

- `purchase_orders.parquet`, `goods_receipts.parquet`
- `vendor_invoices.parquet`, `payments.parquet`
- `sales_orders.parquet`, `deliveries.parquet`
- `customer_invoices.parquet`, `customer_receipts.parquet`
- `document_references.parquet`

### Subledgers

- `ar_open_items.parquet`, `ar_aging.parquet`
- `ap_open_items.parquet`, `ap_aging.parquet`
- `fa_register.parquet`, `fa_depreciation.parquet`
- `inventory_positions.parquet`, `inventory_movements.parquet`

### Period Close

- `trial_balances/*.parquet`
- `accruals.parquet`, `depreciation.parquet`
- `closing_entries.parquet`

### Consolidation

- `eliminations.parquet`
- `currency_translation.parquet`
- `consolidated_trial_balance.parquet`

### FX

- `daily_rates.parquet`, `period_rates.parquet`
- `cta_adjustments.parquet`

### Labels (for ML)

- `anomaly_labels.parquet`
- `fraud_labels.parquet`
- `quality_issues.parquet`

### Control Master Data

- `internal_controls.csv`
- `control_account_mappings.csv`
- `control_process_mappings.csv`
- `sod_conflict_pairs.csv`
- `sod_rules.csv`

## Journal Entry Header Fields

```rust
pub struct JournalEntryHeader {
    // Standard fields
    pub document_id: Uuid,
    pub company_code: String,
    pub fiscal_year: u16,
    pub fiscal_period: u8,
    pub posting_date: NaiveDate,
    pub document_date: NaiveDate,
    pub created_at: DateTime<Utc>,
    pub source: TransactionSource,
    pub business_process: Option<BusinessProcess>,

    // Document flow references
    pub source_document_type: Option<DocumentType>,
    pub source_document_id: Option<String>,

    // Fraud markers
    pub is_fraud: bool,
    pub fraud_type: Option<FraudType>,

    // Control markers
    pub control_ids: Vec<String>,
    pub sox_relevant: bool,
    pub control_status: ControlStatus,
    pub sod_violation: bool,
    pub sod_conflict_type: Option<SodConflictType>,

    // Anomaly markers
    pub is_anomaly: bool,
    pub anomaly_type: Option<AnomalyType>,
    pub anomaly_id: Option<String>,
}
```

## Implementation Phases (Completed)

The enterprise simulation was implemented in 10 phases:

| Phase | Description | Status |
|-------|-------------|--------|
| 1 | Master Data Foundation | Complete |
| 2 | Document Flow Engine | Complete |
| 3 | Intercompany Transactions | Complete |
| 4 | Balance Coherence | Complete |
| 5 | Subledger Simulation | Complete |
| 6 | Currency & FX | Complete |
| 7 | Period Close Engine | Complete |
| 8 | Graph/Network Export | Complete |
| 9 | Anomaly Injection | Complete |
| 10 | Data Quality Variations | Complete |

## Coherence Validation

The generator validates:

- All transactions reference existing master data entities
- Document references form valid chains (PO→GR→Invoice→Payment)
- Trial balance always balanced (debits = credits)
- Subledgers reconcile to GL control accounts
- IC balances match between entities
- FX rates consistent across transactions
- Amounts follow Benford's Law (where applicable)

## Performance

- Single-threaded: ~100K+ entries/second
- Parallel: Scales with available cores
- Memory-efficient streaming for large volumes
