# CLAUDE.md

Guidance for Claude Code working with this repository.

## Build Commands

```bash
cargo build --release          # Build binary
cargo test                     # All tests
cargo test -p datasynth-core   # Specific crate
cargo test test_name           # Single test
cargo check                    # Check only
cargo fmt && cargo clippy      # Format + lint
cargo bench                    # Benchmarks
```

## CLI Usage

Binary: `datasynth-data` (at `target/release/datasynth-data`)

```bash
datasynth-data generate --demo --output ./output
datasynth-data init --industry manufacturing --complexity medium -o config.yaml
datasynth-data validate --config config.yaml
datasynth-data generate --config config.yaml --output ./output
kill -USR1 $(pgrep datasynth-data)  # Pause/resume (Unix)
```

## Server

```bash
cargo run -p datasynth-server -- --port 3000 --worker-threads 4
```

## Architecture

Rust workspace with 15 crates:

```
datasynth-cli          → Binary (generate, validate, init, info, fingerprint)
datasynth-server       → REST/gRPC/WebSocket server
datasynth-ui           → Tauri/SvelteKit desktop UI
datasynth-runtime      → GenerationOrchestrator coordinates workflow
datasynth-generators   → Data generators (JE, Document Flows, Subledgers, Anomalies, Audit)
datasynth-banking      → KYC/AML banking with fraud typologies
datasynth-ocpm         → OCEL 2.0 process mining
datasynth-fingerprint  → Privacy-preserving fingerprint extraction/synthesis
datasynth-graph        → Graph export (PyTorch Geometric, Neo4j, DGL)
datasynth-eval         → Evaluation framework with auto-tuning
datasynth-config       → Configuration schema, validation, presets
datasynth-core         → Domain models, traits, distributions, resource guards
datasynth-output       → Output sinks (CSV, JSON, Parquet)
datasynth-test-utils   → Test utilities
```

### Key Models (datasynth-core/src/models/)

| Category | Models |
|----------|--------|
| Accounting | JournalEntry, ChartOfAccounts, ACDOCA |
| Master Data | Vendor, Customer, Material, FixedAsset, Employee, EntityRegistry |
| Document Flow | PurchaseOrder, GoodsReceipt, VendorInvoice, Payment, SalesOrder, Delivery, CustomerInvoice, CustomerReceipt, DocumentReference |
| Intercompany | IntercompanyRelationship, ICTransactionType, ICMatchedPair, TransferPricingMethod |
| Subledger | AccountBalance, TrialBalance, AR*/AP*/FA*/Inventory* records |
| FX/Close | FxRate, CurrencyTranslation, FiscalPeriod, AccrualEntry |
| Anomalies | AnomalyType, LabeledAnomaly, QualityIssue |
| Controls | InternalControl, ControlMapping, SoD |
| COSO Framework | CosoComponent, CosoPrinciple, ControlScope, CosoMaturityLevel |

### Core Infrastructure (datasynth-core/src/)

- **uuid_factory.rs**: FNV-1a hash-based deterministic UUIDs with generator-type discriminators
- **memory_guard.rs**: Memory limits (Linux /proc/self/statm, macOS ps)
- **disk_guard.rs**: Disk space monitoring (statvfs/GetDiskFreeSpaceExW)
- **cpu_monitor.rs**: CPU tracking with auto-throttle at 0.95 threshold
- **resource_guard.rs**: Unified resource orchestration
- **degradation.rs**: Graceful degradation (Normal→Reduced→Minimal→Emergency)
- **accounts.rs**: GL account constants (AR_CONTROL="1100", AP_CONTROL="2000")
- **templates/**: YAML/JSON template loading with merge strategies

### Generator Modules (datasynth-generators/src/)

| Directory | Purpose |
|-----------|---------|
| (root) | je_generator, coa_generator, company_selector, user_generator, control_generator |
| master_data/ | vendor, customer, material, asset, employee generators |
| document_flow/ | p2p_generator, o2c_generator, three_way_match, document_chain_manager |
| intercompany/ | ic_generator, matching_engine, elimination_generator |
| balance/ | opening_balance, balance_tracker, trial_balance generators |
| subledger/ | ar, ap, fa, inventory generators + reconciliation |
| fx/ | fx_rate_service, currency_translator, cta_generator |
| period_close/ | close_engine, accruals, depreciation, year_end |
| anomaly/ | injector, types, strategies, patterns |
| data_quality/ | missing_values, format_variations, duplicates, typos, labels |
| audit/ | engagement, workpaper, evidence, risk, finding, judgment generators |

### Server (datasynth-server/src/)

- REST: `/api/config`, `/api/stream/{start|stop|pause|resume}`, `/api/stream/trigger/{pattern}`
- WebSocket: `/ws/events`
- Features: API key auth (`X-API-Key`), rate limiting, request timeout

### Desktop UI (datasynth-ui/)

Tauri + SvelteKit + TailwindCSS. Run: `cd crates/datasynth-ui && npm install && npm run tauri dev`

### Graph Module (datasynth-graph/src/)

Builders: transaction_graph, approval_graph, entity_graph
Exporters: pytorch_geometric (.pt), neo4j (CSV + Cypher), dgl

### Banking Module (datasynth-banking/src/)

KYC/AML generator with typologies: structuring, funnel, layering, mule, round_tripping, fraud, spoofing

### Process Mining (datasynth-ocpm/src/)

OCEL 2.0 event logs with P2P/O2C process generators

### Fingerprint Module (datasynth-fingerprint/src/)

Privacy-preserving extraction (differential privacy, k-anonymity) → .dsf files → synthesis

```bash
datasynth-data fingerprint extract --input ./data.csv --output ./fp.dsf --privacy-level standard
datasynth-data fingerprint validate ./fp.dsf
datasynth-data fingerprint evaluate --fingerprint ./fp.dsf --synthetic ./synthetic/
```

### Evaluation Module (datasynth-eval/src/)

- statistical/: Benford's Law, distributions, temporal patterns
- coherence/: Balance validation, IC matching, document chains
- quality/: Completeness, duplicates, format validation
- ml/: Feature distributions, label quality, splits
- enhancement/: AutoTuner generates config patches from evaluation gaps

### COSO Framework (datasynth-core/src/models/coso.rs)

COSO 2013 Internal Control-Integrated Framework:
- **CosoComponent**: ControlEnvironment, RiskAssessment, ControlActivities, InformationCommunication, MonitoringActivities
- **CosoPrinciple**: 17 principles (IntegrityAndEthics through DeficiencyEvaluation) with `component()` and `principle_number()` helpers
- **ControlScope**: EntityLevel, TransactionLevel, ItGeneralControl, ItApplicationControl
- **CosoMaturityLevel**: NonExistent, AdHoc, Repeatable, Defined, Managed, Optimized

Standard controls include 12 transaction-level (C001-C060) and 6 entity-level (C070-C081) controls with full COSO mappings.

### Distributions (datasynth-core/src/distributions/)

LineItemSampler, AmountSampler (log-normal + Benford), TemporalSampler (seasonality), BenfordSampler, FraudAmountGenerator, IndustrySeasonality, HolidayCalendar

## Key Design Decisions

1. **Deterministic RNG**: ChaCha8 with configurable seed
2. **Precise Decimals**: rust_decimal serialized as strings (no IEEE 754)
3. **Balanced Entries**: JournalEntry enforces debits = credits at construction
4. **Benford's Law**: Amount distribution follows first-digit law
5. **Document Chain Integrity**: Proper payment→invoice reference chains
6. **Balance Coherence**: Assets = Liabilities + Equity validation
7. **Collision-Free UUIDs**: Generator-type discriminators prevent ID collisions
8. **Graceful Degradation**: Progressive feature reduction under resource pressure
9. **Three-Way Match**: PO/GR/Invoice matching with configurable tolerances

## Configuration

YAML sections: `global`, `companies`, `chart_of_accounts`, `transactions`, `output`, `fraud`, `internal_controls`, `enterprise`, `master_data`, `document_flows`, `intercompany`, `balance`, `subledger`, `fx`, `period_close`, `graph_export`, `anomaly_injection`, `data_quality`, `business_processes`, `templates`, `approval`, `departments`

Presets: manufacturing, retail, financial_services, healthcare, technology
Complexity: small (~100 accounts), medium (~400), large (~2500)

### Internal Controls Config

```yaml
internal_controls:
  enabled: true
  coso_enabled: true                    # Enable COSO 2013 framework
  include_entity_level_controls: true   # Include C070-C081 entity-level controls
  target_maturity_level: "managed"      # ad_hoc|repeatable|defined|managed|optimized|mixed
  exception_rate: 0.02
  sod_violation_rate: 0.01
```

### Validation Rules

- period_months: 1-120
- compression level: 1-9
- rates/percentages: 0.0-1.0
- approval thresholds: ascending order
- distribution sums: 1.0 (±0.01)

## Anomaly Categories

- **Fraud**: FictitiousTransaction, RevenueManipulation, SplitTransaction, RoundTripping, GhostEmployee, DuplicatePayment
- **Error**: DuplicateEntry, ReversedAmount, WrongPeriod, WrongAccount, MissingReference
- **Process**: LatePosting, SkippedApproval, ThresholdManipulation
- **Statistical**: UnusualAmount, TrendBreak, BenfordViolation
- **Relational**: CircularTransaction, DormantAccountActivity

## Data Quality Variations

- **Missing**: MCAR, MAR, MNAR, Systematic
- **Formats**: Date (ISO/US/EU), Amount (comma/period), Identifier (case/padding)
- **Typos**: Keyboard-aware, transposition, OCR errors, homophones
- **Encoding**: Mojibake, BOM issues, HTML entities

## Export Files

| Category | Files |
|----------|-------|
| Transactions | journal_entries.csv/.json, acdoca.csv |
| Master Data | vendors, customers, materials, fixed_assets, employees, cost_centers |
| Document Flow | purchase_orders, goods_receipts, vendor_invoices, payments, sales_orders, deliveries, customer_invoices, customer_receipts, document_references |
| Subledgers | ar_*, ap_*, fa_*, inventory_* |
| Period Close | trial_balances/, accruals, depreciation, closing_entries |
| Consolidation | eliminations, currency_translation, consolidated_trial_balance |
| Labels | anomaly_labels, fraud_labels, quality_issues, quality_labels |
| Controls | internal_controls, control_*_mappings, sod_*, coso_control_mapping |
| Banking | banking_customers, bank_accounts, bank_transactions, kyc_profiles, aml_typology_labels |
| Process Mining | event_log.json (OCEL 2.0), objects.json, events.json, process_variants |
| Audit | audit_engagements, audit_workpapers, audit_evidence, audit_risks, audit_findings, audit_judgments |

## Performance

~100K+ entries/second single-threaded, scales with cores, memory-efficient streaming

## Python Wrapper

```bash
cd python && pip install -e ".[all]"
```

```python
from datasynth_py import DataSynth, Config, GlobalSettings, CompanyConfig, ChartOfAccountsSettings

config = Config(
    global_settings=GlobalSettings(industry="retail", start_date="2024-01-01", period_months=12),
    companies=[CompanyConfig(code="C001", name="Retail Corp", currency="USD", country="US")],
    chart_of_accounts=ChartOfAccountsSettings(complexity="small"),
)
result = DataSynth().generate(config=config, output={"format": "csv", "sink": "temp_dir"})
```

Blueprints: `blueprints.retail_small()`, `blueprints.banking_medium()`, `blueprints.manufacturing_large()`
