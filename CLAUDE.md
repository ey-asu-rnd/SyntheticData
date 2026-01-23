# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Build Commands

```bash
# Build release binary
cargo build --release

# Run all tests
cargo test

# Run tests for a specific crate
cargo test -p datasynth-core
cargo test -p datasynth-generators
cargo test -p datasynth-graph

# Run a single test by name
cargo test test_name

# Check code without building
cargo check

# Format code
cargo fmt

# Run clippy lints
cargo clippy

# Run benchmarks
cargo bench

# Run specific benchmark
cargo bench --bench generation_throughput
```

## CLI Usage

The binary is `datasynth-data` (located at `target/release/datasynth-data` after build).

```bash
# Generate with demo preset
datasynth-data generate --demo --output ./output

# Create industry-specific config
datasynth-data init --industry manufacturing --complexity medium -o config.yaml

# Validate config
datasynth-data validate --config config.yaml

# Generate from config
datasynth-data generate --config config.yaml --output ./output

# Pause/resume during generation (Unix only)
# Send SIGUSR1 to toggle pause state
kill -USR1 $(pgrep datasynth-data)
```

## Server Usage

```bash
# Start REST/gRPC server
cargo run -p datasynth-server -- --port 3000

# With worker threads
cargo run -p datasynth-server -- --port 3000 --worker-threads 4
```

## Architecture

This is a Rust workspace with 15 crates following a layered architecture:

```
datasynth-cli          → Binary entry point (commands: generate, validate, init, info, fingerprint)
datasynth-server       → REST/gRPC/WebSocket server with auth, rate limiting, timeouts
datasynth-ui           → Tauri/SvelteKit desktop UI
    ↓
datasynth-runtime      → Orchestration layer (GenerationOrchestrator coordinates workflow)
    ↓
datasynth-generators   → Data generators (JE, Document Flows, Subledgers, Anomalies, Audit)
datasynth-banking      → KYC/AML banking transaction generator with fraud typologies
datasynth-ocpm         → Object-Centric Process Mining (OCEL 2.0 event logs)
datasynth-fingerprint  → Privacy-preserving fingerprint extraction and synthesis
    ↓
datasynth-graph        → Graph/network export (PyTorch Geometric, Neo4j, DGL)
datasynth-eval         → Evaluation framework with auto-tuning and recommendations
    ↓
datasynth-config       → Configuration schema, validation, industry presets
    ↓
datasynth-core         → Domain models, traits, distributions, templates, resource guards
    ↓
datasynth-output       → Output sinks (CSV, JSON, Parquet, ControlExport)
datasynth-test-utils   → Test utilities, fixtures, mocks
```

### Key Domain Models (datasynth-core/src/models/)

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

### Core Infrastructure (datasynth-core/src/)

**UUID Factory (`uuid_factory.rs`):**

- **DeterministicUuidFactory**: FNV-1a hash-based UUID generation with generator-type discriminators
- **GeneratorType**: Enum discriminators (JournalEntry=0x01, DocumentFlow=0x02, Vendor=0x03, etc.)
- Thread-safe with `AtomicU64` counter for concurrent generation
- Prevents document ID collisions across different generator types

**Resource Guards (memory, disk, CPU):**

- **MemoryGuard** (`memory_guard.rs`): Cross-platform memory tracking and enforcement
  - Soft/hard limits, check intervals, growth rate monitoring
  - Platform support: Linux (/proc/self/statm), macOS (ps), Windows (stubbed)
  - Memory estimation functions for planning generation volumes

- **DiskSpaceGuard** (`disk_guard.rs`): Disk space monitoring and enforcement
  - Hard limit (minimum free space) and soft limit (warning threshold)
  - Pre-write checks with size estimation
  - Platform support: Linux/macOS (statvfs), Windows (GetDiskFreeSpaceExW)
  - `estimate_output_size_mb()` for capacity planning

- **CpuMonitor** (`cpu_monitor.rs`): CPU load tracking with auto-throttling
  - Configurable high (0.85) and critical (0.95) thresholds
  - Sample-based load history with sliding window averaging
  - Auto-throttle with configurable delay when critical threshold exceeded
  - Platform support: Linux (/proc/stat), macOS (top -l 1)

- **ResourceGuard** (`resource_guard.rs`): Unified resource orchestration
  - Combines MemoryGuard, DiskSpaceGuard, and CpuMonitor
  - Single configuration point for all resource constraints
  - `check_all()` method for comprehensive resource validation

**Graceful Degradation (`degradation.rs`):**

- **DegradationLevel**: Normal → Reduced → Minimal → Emergency
- **DegradationConfig**: Configurable thresholds for memory, disk, CPU
- **DegradationController**: Thread-safe degradation state management
- **DegradationActions**: Actions per level (batch size, skip injections, flush)
- Auto-recovery with hysteresis to prevent oscillation

| Level | Memory | Disk | Batch Size | Actions |
|-------|--------|------|------------|---------|
| Normal | <70% | >1GB | 100% | All features enabled |
| Reduced | 70-85% | 500MB-1GB | 50% | Skip data quality, 50% anomaly rate |
| Minimal | 85-95% | 100-500MB | 25% | Essential data only, no injections |
| Emergency | >95% | <100MB | 0% | Flush and terminate gracefully |

**GL Account Constants (`accounts.rs`):**

- Centralized control account numbers (AR_CONTROL="1100", AP_CONTROL="2000", etc.)
- **AccountCategory** enum with debit/credit normal classification
- Used by all generators for consistent account references

**Template System (`templates/`):**

- **TemplateLoader** (`loader.rs`): Load YAML/JSON template files
- **TemplateProvider** trait (`provider.rs`): Interface for generators to access templates
- **MergeStrategy**: Replace, Extend, MergePreferFile for combining templates
- Categories: person_names, vendor_names, customer_names, material_descriptions, line_item_descriptions

### Generator Modules (datasynth-generators/src/)

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
- `document_flow_je_generator.rs`: Generate JEs from document flows
- `three_way_match.rs`: PO/GR/Invoice matching with quantity/price validation and configurable tolerances

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
- `labels.rs`: ML training labels for data quality issues
  - `LabeledIssueType`: MissingValue, Typo, FormatVariation, Duplicate, EncodingIssue, etc.
  - `QualityIssueSubtype`: Detailed subtypes with severity levels (1-5)
  - `QualityIssueLabel`: Complete label with original/modified values and metadata

**Audit (audit/):**

- `engagement_generator.rs`: Audit engagement with phases (Planning, Fieldwork, Completion)
- `workpaper_generator.rs`: Audit workpapers per ISA 230
- `evidence_generator.rs`: Audit evidence per ISA 500
- `risk_generator.rs`: Risk assessment per ISA 315/330
- `finding_generator.rs`: Audit findings per ISA 265
- `judgment_generator.rs`: Professional judgment documentation per ISA 200

### Server Module (datasynth-server/src/)

**REST API (rest/):**

- `routes.rs`: Axum REST endpoints for config, streaming, pattern triggers
- `auth.rs`: API key authentication middleware
- `rate_limit.rs`: Sliding window rate limiter with per-client tracking
- `websocket.rs`: WebSocket handler for real-time event streaming

**gRPC API (grpc/):**

- `service.rs`: Tonic gRPC service implementation
- `synth.proto`: Protocol buffer definitions

**Key Endpoints:**

- `GET/POST /api/config`: Configuration management
- `POST /api/stream/start|stop|pause|resume`: Stream control
- `POST /api/stream/trigger/{pattern}`: Trigger patterns (month_end, quarter_end, year_end)
- `WS /ws/events`: Real-time event streaming

**Production Features:**

- Authentication: API key validation via `X-API-Key` header
- Rate Limiting: Configurable max requests per time window
- Timeout: Request timeout with `TimeoutLayer`
- Memory Limits: Enforced via `/proc/self/statm` on Linux

### Desktop UI Module (datasynth-ui/)

**Technology Stack:**
- Tauri (Rust backend for desktop)
- SvelteKit (TypeScript/Svelte 5 frontend)
- TailwindCSS (styling)

**Route Structure:**
```
src/routes/
├── +page.svelte           # Dashboard
├── generate/
│   └── stream/            # Real-time streaming viewer
└── config/
    ├── +page.svelte       # Config overview (15 sections)
    ├── global/            # Industry, dates, seed, performance
    ├── transactions/      # Line items, amounts, sources
    ├── master-data/       # Vendors, customers, materials
    ├── document-flows/    # P2P, O2C configuration
    ├── financial/         # Balance, subledger, FX, period close
    ├── compliance/        # Fraud, controls, approval
    ├── analytics/         # Graph export, anomaly, data quality
    ├── output/            # Formats, compression
    ├── chart-of-accounts/ # COA complexity and structure
    ├── business-processes/# Process weight distribution
    ├── user-personas/     # Persona and culture distribution
    ├── templates/         # Template usage rates
    ├── approval/          # Approval thresholds
    ├── departments/       # Department distribution
    └── intercompany/      # IC transaction types
```

**Key Components:**
- `src/lib/stores/config.ts`: Central config state with dirty tracking
- `src/lib/components/forms/`: Reusable form components (FormSection, FormGroup, InputNumber, DistributionEditor)
- `src/lib/components/config/`: Config-specific components (ValidationBanner, PresetSelector)

**Running the UI:**
```bash
cd crates/datasynth-ui
npm install
npm run dev       # Development server
npm run build     # Production build
npm run tauri dev # Desktop app development
```

### Graph Module (datasynth-graph/src/)

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

### Banking Module (datasynth-banking/src/)

KYC/AML banking transaction generator for compliance testing and fraud detection ML.

**Architecture:**
```
BankingOrchestrator → Generators → Typologies → Labels → Models
```

**Models:**

- **BankingCustomer**: Retail, Business, Trust customer personas
- **BankAccount**: Multiple account types with feature sets
- **BankTransaction**: Complete transaction records with direction/channel
- **KycProfile**: Expected activity envelope (declared turnover, transaction frequency, source of funds, geographic exposure, cash intensity, beneficial owner complexity)
- **CounterpartyPool**: Transaction counterparty management
- **CaseNarrative**: Investigation and compliance narratives

**Generators (`generators/`):**

- `customer_generator.rs`: Customer with KYC profile generation
- `account_generator.rs`: Account creation with proper features
- `transaction_generator.rs`: Persona-based transaction generation with causal drivers
- `counterparty_generator.rs`: Counterparty pool management

**AML Typologies (`typologies/`):**

- `structuring.rs`: Structuring below reporting thresholds
- `funnel.rs`: Funnel account patterns for layering
- `layering.rs`: Complex transaction layering schemes
- `mule.rs`: Money mule network patterns
- `round_tripping.rs`: Round-tripping schemes
- `fraud.rs`: Credit card fraud, synthetic identity fraud
- `spoofing.rs`: Adversarial transaction generation for robustness testing
- `injector.rs`: Pattern injection orchestration

**Personas (`personas/`):**

- `retail.rs`: Individual customer behavioral patterns
- `business.rs`: Business account patterns
- `trust.rs`: Trust/corporate patterns

**Labels (`labels/`):**

- `entity_labels.rs`: Entity-level ML labels
- `relationship_labels.rs`: Relationship risk labels
- `transaction_labels.rs`: Transaction classification labels
- `narrative_generator.rs`: Investigation narrative generation

### Process Mining Module (datasynth-ocpm/src/)

Object-Centric Process Mining (OCEL 2.0) event log generation.

**Models:**

- **EventLog**: OCEL 2.0 event log structure
- **Event**: Activity with many-to-many object relationships
- **ObjectInstance**: Business entity evolving through processes
- **ObjectType**: Type definitions (Order, Invoice, PaymentRequest, etc.)
- **ActivityType**: Activity definitions with allowed transitions
- **ObjectRelationship**: Many-to-many links between objects
- **ProcessVariant**: Distinct execution patterns

**Generators (`generator/`):**

- `event_generator.rs`: Core event generation logic
- `p2p_generator.rs`: P2P process events (PO → GR → Invoice → Payment)
- `o2c_generator.rs`: O2C process events (SO → Delivery → Invoice → Receipt)

**Export (`export/`):**

- `Ocel2Exporter`: OCEL 2.0 JSON export functionality

### Fingerprinting Module (datasynth-fingerprint/src/)

Privacy-preserving fingerprint extraction from real data and synthesis of matching synthetic data.

**Architecture:**

```
Real Data → Extract → .dsf File → Generate → Synthetic Data → Evaluate
```

**Models (`models/`):**

- **Fingerprint**: Root container with manifest, schema, statistics, correlations, integrity, rules, anomalies, privacy_audit
- **Manifest**: Version, format, created_at, source metadata, privacy metadata, checksums, optional signature
- **SchemaFingerprint**: Tables with columns, data types, cardinalities, relationships
- **StatisticsFingerprint**: Numeric stats (distribution, percentiles, Benford), categorical stats (frequencies, entropy)
- **CorrelationFingerprint**: Correlation matrices with copula parameters
- **IntegrityFingerprint**: Foreign key definitions, cardinality rules
- **RulesFingerprint**: Balance rules, approval thresholds
- **AnomalyFingerprint**: Anomaly rates, type distributions, temporal patterns
- **PrivacyAudit**: Actions log, epsilon spent, k-anonymity, warnings

**Privacy Engine (`privacy/`):**

- **LaplaceMechanism**: Differential privacy with configurable epsilon
- **GaussianMechanism**: Alternative DP mechanism for (ε,δ)-privacy
- **KAnonymity**: Suppression of rare categorical values below k threshold
- **PrivacyEngine**: Unified interface combining DP, k-anonymity, winsorization
- **PrivacyAuditBuilder**: Build privacy audit with actions and warnings

**Privacy Levels:**

| Level | Epsilon | k | Outlier % | Use Case |
|-------|---------|---|-----------|----------|
| Minimal | 5.0 | 3 | 99% | Low privacy, high utility |
| Standard | 1.0 | 5 | 95% | Balanced (default) |
| High | 0.5 | 10 | 90% | Higher privacy |
| Maximum | 0.1 | 20 | 85% | Maximum privacy |

**Extraction Engine (`extraction/`):**

- **FingerprintExtractor**: Main coordinator for all extraction
- **SchemaExtractor**: Infer data types, cardinalities, relationships
- **StatsExtractor**: Compute distributions, percentiles, Benford analysis
- **CorrelationExtractor**: Pearson correlations, copula fitting
- **IntegrityExtractor**: Detect foreign key relationships
- **RulesExtractor**: Detect balance rules, approval patterns
- **AnomalyExtractor**: Analyze anomaly rates and patterns

**I/O (`io/`):**

- **FingerprintWriter**: Write .dsf files (ZIP with YAML/JSON components)
- **FingerprintReader**: Read .dsf files with checksum verification
- **FingerprintValidator**: Validate DSF structure and integrity
- **validate_dsf()**: Convenience function for CLI validation

**DSF File Format:** ZIP archive containing:
- `manifest.json` - Version, checksums, privacy config
- `schema.yaml` - Tables, columns, relationships
- `statistics.yaml` - Distributions, percentiles, Benford
- `correlations.yaml` - Correlation matrices, copulas
- `integrity.yaml` - FK relationships, cardinality
- `rules.yaml` - Balance constraints, approval thresholds
- `anomalies.yaml` - Anomaly rates, type distribution
- `privacy_audit.json` - Privacy decisions, epsilon spent

**Synthesis (`synthesis/`):**

- **ConfigSynthesizer**: Convert fingerprint to GeneratorConfig
- **DistributionFitter**: Fit AmountSampler parameters from statistics
- **GaussianCopula**: Generate correlated values preserving multivariate structure

**Evaluation (`evaluation/`):**

- **FidelityEvaluator**: Compare synthetic data against fingerprint
- **FidelityReport**: Overall score, component scores, pass/fail status
- **FidelityConfig**: Thresholds and weights for evaluation

**Fidelity Metrics:**
- Statistical: KS statistic, Wasserstein distance, Benford MAD
- Correlation: Correlation matrix RMSE
- Schema: Column type match, row count ratio
- Rules: Balance equation compliance rate

**CLI Commands:**

```bash
# Extract fingerprint
datasynth-data fingerprint extract --input ./data.csv --output ./fp.dsf --privacy-level standard

# Validate
datasynth-data fingerprint validate ./fp.dsf

# Show info
datasynth-data fingerprint info ./fp.dsf --detailed

# Compare
datasynth-data fingerprint diff ./fp1.dsf ./fp2.dsf

# Evaluate fidelity
datasynth-data fingerprint evaluate --fingerprint ./fp.dsf --synthetic ./synthetic/ --threshold 0.8
```

### Evaluation Module (datasynth-eval/src/)

Comprehensive evaluation framework for validating generated data quality.

**Core Modules:**

- `statistical/`: Benford's Law, amount distributions, temporal patterns, line items
- `coherence/`: Balance sheet validation, IC matching, document chains, subledger reconciliation
- `quality/`: Completeness, consistency, duplicates, format validation, uniqueness
- `ml/`: Feature distributions, label quality, graph structure, train/val/test splits
- `report/`: HTML and JSON report generation with baseline comparisons
- `tuning/`: Configuration optimization recommendations

**Enhancement Module (`enhancement/`):**

Auto-tuning engine for deriving optimal configuration from evaluation results.

```
Evaluation Results → Threshold Check → Gap Analysis → Root Cause → Config Suggestion
```

- **AutoTuner** (`auto_tuner.rs`): Analyzes evaluation results and generates config patches
  - `ConfigPatch`: Suggested configuration change with confidence level
  - `MetricGap`: Gap between current and target metric values
  - Metric-to-config mappings for automatic derivation

- **RecommendationEngine** (`recommendation_engine.rs`): Prioritized recommendations
  - `RecommendationPriority`: Critical, High, Medium, Low, Info
  - `RecommendationCategory`: Statistical, Coherence, DataQuality, MLReadiness, Performance
  - `RootCause`: Evidence-based root cause analysis
  - `Recommendation`: Actionable suggestion with expected improvement

**Key Types:**

```rust
pub struct AutoTuneResult {
    pub patches: Vec<ConfigPatch>,
    pub expected_improvement: f64,
    pub addressed_metrics: Vec<String>,
    pub unaddressable_metrics: Vec<String>,
    pub summary: String,
}
```

### Statistical Distributions (datasynth-core/src/distributions/)

- **LineItemSampler**: Empirical distribution (60.68% two-line entries, 88% even line counts)
- **AmountSampler**: Log-normal with round-number bias, Benford's Law compliance
- **TemporalSampler**: Seasonality patterns with industry and holiday integration
- **BenfordSampler**: First-digit distribution following Benford's Law P(d) = log10(1 + 1/d)
- **FraudAmountGenerator**: Suspicious amount patterns (threshold-adjacent, round numbers)
- **IndustrySeasonality**: Industry-specific volume patterns for 10 sectors
- **HolidayCalendar**: Regional holidays for US, DE, GB, CN, JP, IN

### Core Traits (datasynth-core/src/traits/)

- **Generator**: `generate_batch()` and `generate_stream()` for data generation
- **Sink**: Output destination interface (CSV, JSON, Parquet implementations)
- **PostProcessor**: Post-generation transformation interface for data quality variations
  - `ProcessContext`: Record-level context (index, batch size, output format, metadata)
  - `ProcessorStats`: Modification tracking (records processed, modified, labels generated)

## Key Design Decisions

1. **Deterministic RNG**: Uses ChaCha8 with configurable seed for reproducible output
2. **Precise Decimals**: `rust_decimal` for financial calculations (no floating point); serialized as strings to prevent IEEE 754 artifacts
3. **Balanced Entries**: JournalEntry enforces debits = credits at construction time
4. **Empirical Distributions**: Based on academic research on real GL data patterns
5. **Benford's Law**: Amount distribution follows first-digit law with fraud pattern exceptions
6. **Weighted Company Selection**: Companies selected based on volume_weight
7. **Document Chain Integrity**: All documents maintain proper reference chains with payment→invoice links
8. **Balance Coherence**: Running balance tracker validates Assets = Liabilities + Equity
9. **Subledger Reconciliation**: Automatic GL-to-subledger control account reconciliation
10. **ML-Ready Output**: Graph exports with train/val/test splits and computed features
11. **Collision-Free UUIDs**: FNV-1a hash-based UUID generation with generator-type discriminators prevents document ID collisions
12. **Memory Safety**: MemoryGuard with configurable soft/hard limits prevents OOM conditions during large generations
13. **Three-Way Match Validation**: Actual PO/GR/Invoice matching with configurable quantity and price tolerances
14. **Graceful Degradation**: Progressive feature reduction under resource pressure (Normal→Reduced→Minimal→Emergency)
15. **Multi-Resource Guards**: Unified CPU, memory, and disk monitoring with automatic throttling
16. **Evaluation-Driven Enhancement**: Auto-tuner derives config improvements from evaluation metric gaps
17. **KYC Profile Coherence**: Banking transactions driven by declared activity envelopes with ground-truth labeling
18. **OCEL 2.0 Compliance**: Process mining exports follow Object-Centric Event Log standard

## Configuration Schema

Config files use YAML with sections:

**Core:** `global`, `companies`, `chart_of_accounts`, `transactions`, `output`

**Compliance:** `fraud`, `internal_controls`

**Enterprise:** `enterprise`, `master_data`, `document_flows`, `intercompany`

**Financial:** `balance`, `subledger`, `fx`, `period_close`

**ML/Analytics:** `graph_export`, `anomaly_injection`, `data_quality`

**Supporting:** `business_processes`, `templates`, `approval`, `departments`

**Templates:** File-based template loading for regional/sector customization:
- `template_path`: Path to external YAML/JSON template file
- `merge_strategy`: Replace, Extend, or MergePreferFile
- Categories: person_names, vendor_names, customer_names, material_descriptions, line_item_descriptions

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

- `journal_entries.csv` / `.json`
- `acdoca.csv` - SAP HANA Universal Journal format

### Master Data

- `vendors.csv`, `customers.csv`
- `materials.csv`, `fixed_assets.csv`
- `employees.csv`, `cost_centers.csv`

### Document Flow

- `purchase_orders.csv`, `goods_receipts.csv`
- `vendor_invoices.csv`, `payments.csv`
- `sales_orders.csv`, `deliveries.csv`
- `customer_invoices.csv`, `customer_receipts.csv`
- `document_references.csv`

### Subledgers

- `ar_open_items.csv`, `ar_aging.csv`
- `ap_open_items.csv`, `ap_aging.csv`
- `fa_register.csv`, `fa_depreciation.csv`
- `inventory_positions.csv`, `inventory_movements.csv`

### Period Close

- `trial_balances/*.csv`
- `accruals.csv`, `depreciation.csv`
- `closing_entries.csv`

### Consolidation

- `eliminations.csv`
- `currency_translation.csv`
- `consolidated_trial_balance.csv`

### FX

- `daily_rates.csv`, `period_rates.csv`
- `cta_adjustments.csv`

### Labels (for ML)

- `anomaly_labels.csv`
- `fraud_labels.csv`
- `quality_issues.csv`
- `quality_labels.csv` - Data quality issue labels with original/modified values

### Control Master Data

- `internal_controls.csv`
- `control_account_mappings.csv`
- `control_process_mappings.csv`
- `sod_conflict_pairs.csv`
- `sod_rules.csv`

### Banking/KYC/AML

- `banking_customers.csv` - Customer profiles with KYC data
- `bank_accounts.csv` - Account records with features
- `bank_transactions.csv` - Transaction records with channels/categories
- `kyc_profiles.csv` - Expected activity envelopes
- `counterparties.csv` - Counterparty pool
- `aml_typology_labels.csv` - Structuring, funnel, mule, layering labels
- `entity_risk_labels.csv` - Entity-level risk classifications
- `transaction_risk_labels.csv` - Transaction-level classifications

### Process Mining (OCEL 2.0)

- `event_log.json` - OCEL 2.0 format event log
- `objects.json` - Object instances and types
- `events.json` - Event records with object relationships
- `process_variants.csv` - Distinct execution patterns

### Audit Data

- `audit_engagements.csv` - Engagement metadata with materiality
- `audit_workpapers.csv` - Workpaper records per ISA 230
- `audit_evidence.csv` - Evidence per ISA 500
- `audit_risks.csv` - Risk assessments per ISA 315
- `audit_findings.csv` - Findings per ISA 265
- `audit_judgments.csv` - Professional judgments

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

The enterprise simulation was implemented in 14 phases:

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
| 11 | Banking/KYC/AML Module | Complete |
| 12 | Process Mining (OCEL 2.0) | Complete |
| 13 | Audit Data Generation | Complete |
| 14 | Resource Guards & Evaluation | Complete |

## Coherence Validation

The generator validates:

- All transactions reference existing master data entities
- Document references form valid chains (PO→GR→Invoice→Payment) with proper payment→invoice links
- Document IDs are unique across all generators (no collisions)
- Trial balance always balanced (debits = credits)
- Subledgers reconcile to GL control accounts
- IC balances match between entities
- FX rates consistent across transactions
- Amounts follow Benford's Law (where applicable)
- Three-way match validates actual PO/GR/Invoice quantities and prices

## Performance

- Single-threaded: ~100K+ entries/second
- Parallel: Scales with available cores
- Memory-efficient streaming for large volumes

## Production Readiness

The codebase has been hardened for production use with the following features:

### Data Integrity
- CSV and JSON output formats fully implemented
- All transfer pricing methods produce correct calculations
- Custom close tasks return errors instead of silently skipping
- Comprehensive config validation with bounds checking
- Zero document ID collisions (FNV-1a hash-based UUID factory with generator-type discriminators)
- Decimal values serialized as strings (no IEEE 754 floating point artifacts)
- Complete payment→invoice document reference chains

### API Robustness
- REST/gRPC pattern triggers fully implemented
- REST set_config actually applies configuration changes
- Proper error responses for invalid requests

### Resource Management
- **MemoryGuard**: Memory limit enforcement (Linux: /proc/self/statm, macOS: ps command)
  - Configurable soft/hard limits with growth rate monitoring
  - Memory estimation functions for planning generation volumes
- **DiskSpaceGuard**: Disk space monitoring and enforcement
  - Hard/soft limits with pre-write capacity checks
  - Platform support: Linux/macOS (statvfs), Windows (GetDiskFreeSpaceExW)
- **CpuMonitor**: CPU load tracking with auto-throttling
  - High (0.85) and critical (0.95) thresholds
  - Auto-throttle delay when critical threshold exceeded
- **ResourceGuard**: Unified resource orchestration combining all guards
- **DegradationController**: Graceful degradation under resource pressure
  - Normal → Reduced → Minimal → Emergency levels
  - Auto-recovery with hysteresis to prevent oscillation
- Configurable request timeouts
- Worker thread configuration support
- CLI pause/resume via SIGUSR1 signal

### Security
- API key authentication middleware
- Rate limiting with sliding window algorithm
- Configurable exempt paths for health checks

### Observability
- Comprehensive logging throughout generation pipeline
- Progress bar with pause state indication
- Detailed error messages with context

### Configuration Validation
- period_months: 1-120 (max 10 years)
- compression level: 1-9 when enabled
- All rate/percentage fields: 0.0-1.0
- Approval thresholds: strictly ascending order
- Distribution sums: must equal 1.0 (±0.01 tolerance)

### Desktop UI (datasynth-ui)
- Tauri + SvelteKit cross-platform desktop application
- 15+ configuration pages covering all config sections
- Real-time WebSocket streaming viewer
- Industry preset management
- Real-time validation with error feedback

### Benchmarks
- Criterion-based benchmark suite in `benches/`
- Generation throughput benchmarks (JE, master data, document flows)
- Distribution sampling benchmarks
- Output sink benchmarks (CSV, JSON)
- Scalability benchmarks (memory, parallel scaling)
- Correctness benchmarks (Benford's Law, balance coherence)

## Python Wrapper (python/datasynth_py)

A Python wrapper is available for programmatic access to DataSynth.

### Installation

```bash
cd python
pip install -e ".[all]"
```

Optional dependency groups:
- `cli`: PyYAML for config serialization
- `memory`: pandas for in-memory table loading
- `streaming`: websockets for server streaming
- `all`: All optional dependencies
- `dev`: Development dependencies (pytest, mypy, ruff)

### Usage

```python
from datasynth_py import DataSynth, CompanyConfig, Config, GlobalSettings, ChartOfAccountsSettings

config = Config(
    global_settings=GlobalSettings(
        industry="retail",
        start_date="2024-01-01",
        period_months=12,
    ),
    companies=[
        CompanyConfig(code="C001", name="Retail Corp", currency="USD", country="US"),
    ],
    chart_of_accounts=ChartOfAccountsSettings(complexity="small"),
)

synth = DataSynth()
result = synth.generate(config=config, output={"format": "csv", "sink": "temp_dir"})
```

### Blueprints

```python
from datasynth_py.config import blueprints

# Available: retail_small, banking_medium, manufacturing_large
config = blueprints.retail_small(companies=4, transactions=10000)
```

### Key Classes

| Class | Description |
|-------|-------------|
| `DataSynth` | Main client for generation (CLI batch or server streaming) |
| `Config` | Root configuration container |
| `GlobalSettings` | Industry, start_date, period_months, seed, group_currency |
| `CompanyConfig` | Company code, name, currency, country, volume |
| `ChartOfAccountsSettings` | Complexity level (small/medium/large) |
| `FraudSettings` | Fraud injection (enabled, rate) |
| `OutputSpec` | Output format, sink, path |
| `GenerationResult` | Result with output_dir or tables |

### Running Tests

```bash
cd python
python -m unittest discover -s tests -v
```
