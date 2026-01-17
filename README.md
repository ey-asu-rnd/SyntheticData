# SyntheticData

A high-performance, configurable synthetic data generator for **complete enterprise simulation**. Produces realistic, interconnected General Ledger Journal Entries, Chart of Accounts, SAP HANA-compatible ACDOCA event logs, document flows, subledger records, and ML-ready graph exports at scale (10K to 100M+ transactions).

## Overview

SyntheticData generates coherent enterprise data that is indistinguishable from real corporate data for:
- Algorithm testing and validation
- Neural network training
- Analytics and BI system testing
- Audit procedure testing
- SOX compliance testing

## Features

### Core Data Generation
- **Realistic Statistical Distributions**: Line item counts, amounts, and patterns based on empirical research from real-world general ledger data
- **Benford's Law Compliance**: First-digit distribution following Benford's Law with configurable fraud patterns
- **Industry Presets**: Manufacturing, Retail, Financial Services, Healthcare, Technology, and more
- **Configurable Chart of Accounts**: Small (~100), Medium (~400), Large (~2500 accounts) with industry-specific account structures
- **Transaction Source Modeling**: Manual entries (20%), Automated/batch (70%), Recurring (7%), Adjustments (3%)
- **Temporal Patterns**: Month-end (2.5x), Quarter-end (4x), Year-end (6x) volume spikes with working hour patterns
- **Industry Seasonality**: Sector-specific patterns (Black Friday for retail, year-end for financial services, etc.)
- **Regional Holiday Calendars**: US, DE, GB, CN, JP, IN with lunar calendar support

### Enterprise Simulation

#### Master Data Management
- **Entity Registry**: Central registry with temporal validity tracking
- **Enhanced Vendors**: Payment terms, bank accounts, tax IDs, intercompany flags, vendor behavior patterns
- **Enhanced Customers**: Credit ratings, credit limits, payment behavior, intercompany relationships
- **Materials/Products**: Bill of materials, valuation methods, standard costs, account determination
- **Fixed Assets**: Depreciation schedules, useful life tracking, acquisition/disposal handling
- **Employees**: Manager hierarchy, approval limits, system roles, transaction authorizations

#### Document Flow Engine
- **Procure-to-Pay (P2P)**: Purchase Order → Goods Receipt → Vendor Invoice → Payment
- **Order-to-Cash (O2C)**: Sales Order → Delivery → Customer Invoice → Customer Receipt
- **Document References**: Complete chain linking with reference types (FollowOn, Payment, Reversal)
- **Three-Way Matching**: PO/GR/Invoice matching with configurable match rates

#### Intercompany Transactions
- **Ownership Structures**: Parent-subsidiary relationships with ownership percentages
- **IC Matching Engine**: Automatic generation of matched IC entry pairs
- **Transfer Pricing**: Cost-plus, resale minus, and comparable pricing methods
- **Consolidation Eliminations**: Automatic elimination entry generation

#### Balance Coherence
- **Opening Balances**: Coherent balance sheet generation (Assets = Liabilities + Equity)
- **Running Balance Tracker**: Real-time balance validation across all companies
- **Trial Balance Generation**: Period-end trial balances with validation
- **Financial Ratios**: DSO, DPO, gross margin targeting and validation

#### Subledger Simulation
- **Accounts Receivable**: Invoices, receipts, credit memos, aging buckets
- **Accounts Payable**: Invoices, payments, debit memos, payment schedules
- **Fixed Assets**: Asset register, depreciation schedules, disposals with gain/loss
- **Inventory**: Positions, movements, valuations (FIFO, LIFO, weighted average, standard cost)
- **GL Reconciliation**: Automatic subledger-to-GL control account reconciliation

#### Currency & FX
- **FX Rate Generation**: Realistic rates using Ornstein-Uhlenbeck (mean-reverting) process
- **Rate Types**: Spot, closing (period-end), and average (P&L) rates
- **Currency Translation**: Automatic translation of foreign subsidiary trial balances
- **CTA Generation**: Currency Translation Adjustment entries

#### Period Close Engine
- **Monthly Close Tasks**: Depreciation runs, accruals, allocations
- **Intercompany Settlement**: Automatic IC netting and settlement entries
- **Year-End Close**: Income statement closing, retained earnings rollforward
- **Tax Provision**: Configurable tax provision calculations

### ML & Analytics Features

#### Graph/Network Export
- **Transaction Network**: Accounts/entities as nodes, transactions as edges
- **Approval Network**: Users as nodes, approvals as edges (for SoD detection)
- **Entity Relationship Graph**: Legal entities with ownership edges

**Export Formats:**
- PyTorch Geometric (.pt files with node_features, edge_index, edge_attr, train/val/test masks)
- Neo4j (CSV files with Cypher import scripts)
- DGL (Deep Graph Library format)

**ML Features:**
- Temporal features (weekday, period, month-end, year-end flags)
- Amount features (log-transformed, Benford probability)
- Structural features (line count, unique accounts)
- Categorical features (business process, source type - one-hot encoded)

#### Anomaly Injection Framework
- **Fraud Types**: 20+ fraud scenarios including fictitious transactions, revenue manipulation, kickbacks
- **Error Types**: Duplicates, reversed amounts, wrong periods, misclassifications
- **Process Issues**: Late postings, skipped approvals, threshold manipulation
- **Statistical Anomalies**: Unusual amounts, trend breaks, Benford violations
- **Relational Anomalies**: Circular transactions, dormant account activity

**Features:**
- Configurable rates per anomaly category
- Temporal patterns (year-end spikes)
- Anomaly clustering (realistic batch patterns)
- Full labeling for supervised learning

#### Data Quality Variations
- **Missing Values**: MCAR, MAR, MNAR, and Systematic missing patterns
- **Format Variations**: Date formats (ISO, US, EU), amount formats (comma/dot separators)
- **Duplicates**: Exact, near-duplicate, and fuzzy duplicate generation
- **Typos**: Keyboard-aware substitution, transposition, OCR errors, homophones
- **Encoding Issues**: Mojibake, missing characters, BOM issues, HTML entities

### Compliance & Controls
- **Internal Controls System (ICS)**: SOX 404 compliant controls with segregation of duties
- **Fraud Scenarios**: Configurable fraud injection including suspense account abuse, fictitious transactions, timing anomalies
- **Weighted Company Selection**: Transaction volume distribution based on company size
- **Deterministic Generation**: Seeded RNG ensures reproducible output for testing
- **Multiple Output Formats**: CSV and JSON (Parquet planned for future release)

### Server & API

#### REST API
- **Configuration Management**: GET/POST `/api/config` for runtime configuration
- **Generation Control**: Start, pause, resume, stop generation streams
- **Real-time Streaming**: WebSocket endpoint for live event streaming
- **Authentication**: API key-based authentication middleware
- **Rate Limiting**: Configurable request rate limiting

#### gRPC API
- **Streaming Generation**: Server-side streaming for high-performance data delivery
- **Pattern Triggers**: Trigger specific generation patterns programmatically
- **Control Commands**: Pause, resume, and stop generation streams

#### Security & Production Features
- **Authentication**: API key validation with configurable exempt paths
- **Rate Limiting**: Sliding window rate limiter with per-client tracking
- **Timeout Handling**: Configurable request timeouts
- **Memory Limits**: Enforced memory limits to prevent OOM conditions
- **Comprehensive Logging**: Detailed logging throughout the generation pipeline

### CLI Features
- **Pause/Resume**: Send `SIGUSR1` signal to toggle pause during generation (Unix)
- **Worker Threads**: Configure number of worker threads for parallel generation
- **Verbose Mode**: Detailed logging with `-v` flag

### Desktop UI (Tauri + SvelteKit)
- **Visual Configuration**: Comprehensive UI for all 15+ configuration sections
- **Real-time Streaming**: Live generation viewer with WebSocket streaming
- **Preset Management**: Industry presets with one-click application
- **Validation Feedback**: Real-time configuration validation with error display
- **Cross-Platform**: Native desktop app for Windows, macOS, and Linux

## Installation

### From Source

```bash
git clone https://github.com/your-repo/SyntheticData.git
cd SyntheticData
cargo build --release
```

The binary will be available at `target/release/synth-data`.

## Quick Start

```bash
# Generate a sample configuration file
synth-data init --industry manufacturing --complexity medium -o config.yaml

# Validate the configuration
synth-data validate --config config.yaml

# Generate synthetic data
synth-data generate --config config.yaml --output ./output

# View available presets and options
synth-data info
```

## CLI Commands

### `init` - Create Configuration

```bash
synth-data init --industry <INDUSTRY> --complexity <COMPLEXITY> -o <OUTPUT_FILE>
```

**Industries**: `manufacturing`, `retail`, `financial_services`, `healthcare`, `technology`, `energy`, `telecom`, `transportation`, `hospitality`, `professional_services`

**Complexity**: `small`, `medium`, `large`

### `validate` - Validate Configuration

```bash
synth-data validate --config <CONFIG_FILE>
```

### `generate` - Generate Data

```bash
synth-data generate --config <CONFIG_FILE> --output <OUTPUT_DIR>
```

### `info` - Show Available Options

```bash
synth-data info
```

## Server Usage

### Starting the Server

```bash
# Start with default settings
cargo run -p synth-server -- --port 3000

# With worker threads and authentication
cargo run -p synth-server -- --port 3000 --worker-threads 4
```

### REST API Endpoints

```bash
# Get current configuration
curl http://localhost:3000/api/config

# Update configuration
curl -X POST http://localhost:3000/api/config \
  -H "Content-Type: application/json" \
  -d '{"industry": "manufacturing", "period_months": 12}'

# Start generation stream
curl -X POST http://localhost:3000/api/stream/start

# Pause/Resume generation
curl -X POST http://localhost:3000/api/stream/pause
curl -X POST http://localhost:3000/api/stream/resume

# Stop generation
curl -X POST http://localhost:3000/api/stream/stop

# Trigger specific pattern
curl -X POST http://localhost:3000/api/stream/trigger/month_end
```

### WebSocket Streaming

Connect to `ws://localhost:3000/ws/events` for real-time event streaming.

### CLI Pause/Resume (Unix)

During generation, send SIGUSR1 to toggle pause:

```bash
# Start generation in background
synth-data generate --demo &

# Get PID and toggle pause
kill -USR1 $(pgrep synth-data)
```

## Configuration

The generator uses YAML configuration files. See the [Example Enterprise Configuration](#example-enterprise-configuration) section for a complete example.

### Global Settings

```yaml
global:
  seed: 42                    # Optional: for reproducible generation
  industry: manufacturing
  start_date: 2024-01-01
  period_months: 12
  group_currency: USD
  parallel: true
```

### Companies with Weighted Selection

Companies are selected for transactions based on their `volume_weight`. Higher weights mean more transactions.

```yaml
companies:
  - code: "1000"
    name: "US Headquarters"
    currency: USD
    country: US
    volume_weight: 1.0        # 50% of transactions (1.0 / 2.0 total)
  - code: "2000"
    name: "EU Subsidiary"
    currency: EUR
    country: DE
    volume_weight: 0.5        # 25% of transactions
  - code: "3000"
    name: "APAC Operations"
    currency: JPY
    country: JP
    volume_weight: 0.5        # 25% of transactions
```

### Statistical Distributions

Based on empirical research, the default distributions are:

**Line Item Distribution:**
| Line Items | Percentage |
|------------|------------|
| 2 items    | 60.68%     |
| 3 items    | 5.77%      |
| 4 items    | 16.63%     |
| 5 items    | 3.06%      |
| 6 items    | 3.32%      |
| 7 items    | 1.13%      |
| 8 items    | 1.88%      |
| 9 items    | 0.42%      |
| 10-99      | 6.33%      |
| 100-999    | 0.76%      |
| 1000+      | 0.02%      |

**Debit/Credit Balance:**
- 82% equal debit and credit line counts
- 11% more credit lines
- 7% more debit lines

**Even/Odd Line Count:**
- 88% even line count
- 12% odd line count

### Benford's Law Compliance

Transaction amounts follow Benford's Law for first-digit distribution:

```yaml
transactions:
  benford:
    enabled: true                    # Enable Benford-compliant amounts
    exempt_sources:                  # Sources exempt from Benford's Law
      - recurring                    # Payroll, rent - naturally round numbers
      - payroll
```

**First-Digit Probabilities (Benford's Law):**
| Digit | Probability |
|-------|-------------|
| 1     | 30.1%       |
| 2     | 17.6%       |
| 3     | 12.5%       |
| 4     | 9.7%        |
| 5     | 7.9%        |
| 6     | 6.7%        |
| 7     | 5.8%        |
| 8     | 5.1%        |
| 9     | 4.6%        |

### Industry Seasonality

```yaml
transactions:
  enhanced_seasonality:
    use_industry_seasonality: true
    custom_events:
      - name: "Company Anniversary Sale"
        start_month: 6
        start_day: 15
        end_month: 6
        end_day: 20
        multiplier: 2.0
```

**Industry Patterns:**

| Industry | Peak Events | Multiplier |
|----------|-------------|------------|
| Retail | Black Friday (Nov 20-30) | 8x |
| Retail | Christmas Rush (Dec 15-24) | 6x |
| Retail | Post-Holiday Returns (Jan 1-15) | 3x |
| Manufacturing | Year-End Close (Dec 20-31) | 4x |
| Manufacturing | Q4 Inventory Buildup (Oct-Nov) | 2x |
| Financial Services | Year-End (Dec 15-31) | 8x |
| Financial Services | Quarter-Ends (last 5 days) | 5x |
| Healthcare | Year-End (Dec 15-31) | 3x |
| Healthcare | Insurance Renewal (Jan) | 2x |
| Technology | Q4 Enterprise Deals (Dec) | 4x |

### Regional Holiday Calendars

```yaml
transactions:
  enhanced_seasonality:
    holiday_regions:
      - US    # United States
      - DE    # Germany
      - GB    # United Kingdom
      - CN    # China (includes lunar calendar)
      - JP    # Japan
      - IN    # India (includes Diwali)
```

### Fraud Scenarios

```yaml
fraud:
  enabled: true
  fraud_rate: 0.005                # 0.5% of transactions
  approval_thresholds:             # For threshold-adjacent fraud
    - 1000
    - 5000
    - 10000
    - 50000
    - 100000
  fraud_type_distribution:
    suspense_account_abuse: 0.25
    fictitious_transaction: 0.15
    revenue_manipulation: 0.10
    timing_anomaly: 0.10
    split_transaction: 0.15        # Just below approval thresholds
    round_number: 0.10             # Suspicious round amounts
    duplicate_payment: 0.15
```

### Internal Controls System (ICS)

```yaml
internal_controls:
  enabled: true
  exception_rate: 0.02           # 2% control exceptions
  sod_violation_rate: 0.01       # 1% segregation of duties violations
  sox_materiality_threshold: 50000.0
  export_control_master_data: true
```

### Anomaly Injection

```yaml
anomaly_injection:
  enabled: true
  total_rate: 0.02               # 2% total anomaly rate
  fraud_rate: 0.005              # 0.5% fraud rate
  error_rate: 0.01               # 1% error rate
  generate_labels: true          # Generate labels for ML
  clustering:
    enabled: true                # Anomalies come in batches
    cluster_probability: 0.3
  temporal_patterns:
    year_end_spike: 2.0          # 2x anomalies at year-end
```

### Data Quality Variations

```yaml
data_quality:
  enabled: true
  missing_rate: 0.01             # 1% missing values
  typo_rate: 0.005               # 0.5% typos
  duplicate_rate: 0.002          # 0.2% duplicates
  format_variation_rate: 0.05    # 5% format variations
```

### Graph Export

```yaml
graph_export:
  enabled: true
  formats:
    - pytorch_geometric
    - neo4j
    - dgl
  include_features: true
  train_val_test_split:
    train: 0.7
    validation: 0.15
    test: 0.15
```

## Output File Structure

```
output/
├── master_data/
│   ├── vendors.csv
│   ├── customers.csv
│   ├── materials.csv
│   ├── fixed_assets.csv
│   ├── employees.csv
│   └── cost_centers.csv
├── transactions/
│   ├── journal_entries.csv
│   ├── purchase_orders.csv
│   ├── goods_receipts.csv
│   ├── vendor_invoices.csv
│   ├── payments.csv
│   ├── sales_orders.csv
│   ├── deliveries.csv
│   ├── customer_invoices.csv
│   ├── customer_receipts.csv
│   └── document_references.csv
├── subledgers/
│   ├── ar_open_items.csv
│   ├── ar_aging.csv
│   ├── ap_open_items.csv
│   ├── ap_aging.csv
│   ├── fa_register.csv
│   ├── fa_depreciation.csv
│   ├── inventory_positions.csv
│   └── inventory_movements.csv
├── period_close/
│   ├── trial_balances/
│   │   └── *.csv
│   ├── accruals.csv
│   ├── depreciation.csv
│   └── closing_entries.csv
├── consolidation/
│   ├── eliminations.csv
│   ├── currency_translation.csv
│   └── consolidated_trial_balance.csv
├── fx/
│   ├── daily_rates.csv
│   ├── period_rates.csv
│   └── cta_adjustments.csv
├── graphs/
│   ├── transaction_network/
│   │   └── pytorch_geometric/
│   │       ├── node_features.pt
│   │       ├── edge_index.pt
│   │       ├── edge_attr.pt
│   │       ├── labels.pt
│   │       ├── train_mask.pt
│   │       ├── val_mask.pt
│   │       └── test_mask.pt
│   ├── approval_network/
│   │   └── pytorch_geometric/*.pt
│   └── entity_relationship/
│       └── neo4j/
│           ├── nodes_*.csv
│           ├── edges_*.csv
│           └── import.cypher
├── labels/
│   ├── anomaly_labels.csv
│   ├── fraud_labels.csv
│   └── quality_issues.csv
└── controls/
    ├── internal_controls.csv
    ├── control_account_mappings.csv
    ├── control_process_mappings.csv
    ├── sod_conflict_pairs.csv
    └── sod_rules.csv
```

## Architecture

```
SyntheticData/
├── Cargo.toml                    # Workspace root
└── crates/
    ├── synth-core/              # Domain models, traits, distributions
    │   ├── models/
    │   │   ├── journal_entry.rs
    │   │   ├── master_data.rs      # Enhanced Vendor/Customer
    │   │   ├── material.rs         # Material/Product Master
    │   │   ├── fixed_asset.rs      # Fixed Asset Master
    │   │   ├── entity_registry.rs  # Central entity registry
    │   │   ├── internal_control.rs # SOX control definitions
    │   │   ├── anomaly.rs          # Anomaly types and labels
    │   │   └── ...
    │   ├── distributions/       # Statistical samplers
    │   └── traits/              # Generator, Sink interfaces
    ├── synth-config/            # Configuration schema and validation
    ├── synth-generators/        # Data generators
    │   ├── master_data/         # Entity generators
    │   ├── document_flow/       # P2P, O2C flow generators
    │   ├── intercompany/        # IC transaction generators
    │   ├── balance/             # Balance coherence
    │   ├── subledger/           # AR, AP, FA, Inventory
    │   ├── fx/                  # FX rate generation
    │   ├── period_close/        # Period close engine
    │   ├── anomaly/             # Anomaly injection
    │   ├── data_quality/        # Data quality variations
    │   └── ...
    ├── synth-graph/             # Graph/network export
    │   ├── models/              # Node, edge, graph structures
    │   ├── builders/            # Graph builders
    │   ├── exporters/           # PyTorch Geometric, Neo4j, DGL
    │   └── ml/                  # Feature computation, splits
    ├── synth-output/            # Output sinks
    ├── synth-runtime/           # Orchestration
    ├── synth-cli/               # Command-line interface
    ├── synth-server/            # REST/gRPC/WebSocket server
    │   ├── rest/                # Axum REST API with auth & rate limiting
    │   ├── grpc/                # Tonic gRPC service
    │   └── websocket/           # Real-time event streaming
    └── synth-ui/                # Tauri/SvelteKit desktop UI
```

## Example Enterprise Configuration

```yaml
global:
  seed: 12345
  start_date: 2022-01-01
  period_months: 36
  group_currency: USD
  industry: manufacturing

enterprise:
  name: "Global Manufacturing Corp"
  legal_entities:
    - code: "1000"
      name: "GMC Holdings"
      country: US
      currency: USD
      is_parent: true
    - code: "1100"
      name: "GMC Americas"
      parent: "1000"
      ownership_percent: 100
    - code: "1200"
      name: "GMC Europe"
      country: DE
      currency: EUR
      parent: "1000"
      ownership_percent: 100

master_data:
  vendors: { count: 500, intercompany_percent: 0.05 }
  customers: { count: 2000, intercompany_percent: 0.05 }
  materials: { count: 5000 }
  fixed_assets: { count: 800 }
  employees: { count: 1500 }

document_flows:
  p2p: { enabled: true, three_way_match_rate: 0.95 }
  o2c: { enabled: true, credit_check_failure_rate: 0.02 }

intercompany:
  enabled: true
  ic_transaction_rate: 0.15
  transfer_pricing_method: cost_plus
  markup_percent: 0.05

balance:
  generate_opening_balances: true
  generate_trial_balances: true
  target_gross_margin: 0.35
  target_dso_days: 45

subledger:
  ar: { enabled: true, average_payment_days: 45 }
  ap: { enabled: true, average_payment_days: 30 }
  fa: { enabled: true, generate_depreciation: true }
  inventory: { enabled: true, valuation_method: standard_cost }

fx:
  enabled: true
  generate_cta: true

period_close:
  enabled: true
  run_depreciation: true
  generate_accruals: true
  intercompany_settlement: true

graph_export:
  enabled: true
  formats: [pytorch_geometric, neo4j]

anomaly_injection:
  enabled: true
  total_rate: 0.02
  fraud_rate: 0.005
  generate_labels: true

data_quality:
  enabled: true
  missing_rate: 0.01
  typo_rate: 0.005
```

## Use Cases

- **Process Mining**: Generate realistic event logs for process discovery and conformance checking
- **Analytics Testing**: Create large datasets for testing BI and analytics tools
- **Machine Learning**: Training data for fraud detection, anomaly detection, and GNN models
- **Audit Testing**: Realistic data with known fraud patterns for audit procedure testing
- **SOX Compliance Testing**: Test internal controls and segregation of duties monitoring
- **System Testing**: Load testing for ERP and accounting systems
- **Education**: Realistic accounting data for training and demonstrations
- **Data Engineering**: Test ETL pipelines with realistic data quality issues

## Performance

- Single-threaded: ~100K+ entries/second
- Parallel: Scales with available cores
- Memory-efficient streaming for large volumes

## Dependencies

- `serde` / `serde_yaml` / `serde_json`: Serialization
- `rust_decimal`: Precise financial calculations
- `chrono`: Date/time handling
- `uuid`: Document identifiers
- `rand` / `rand_chacha` / `rand_distr`: Statistical sampling
- `clap`: CLI parsing
- `indicatif`: Progress reporting
- `arrow` / `parquet`: Columnar output formats

## License

MIT

## Contributing

Contributions welcome! Please open an issue or pull request.
