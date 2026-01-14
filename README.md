# SyntheticData

A high-performance, configurable synthetic data generator for enterprise accounting data. Produces realistic General Ledger Journal Entries, Chart of Accounts, and SAP HANA-compatible ACDOCA event logs at scale (10K to 100M+ transactions).

## Features

- **Realistic Statistical Distributions**: Line item counts, amounts, and patterns based on empirical research from real-world general ledger data
- **Industry Presets**: Manufacturing, Retail, Financial Services, Healthcare, Technology
- **Configurable Chart of Accounts**: Small (~100), Medium (~400), Large (~2500 accounts) with industry-specific account structures
- **Transaction Source Modeling**: Manual entries (20%), Automated/batch (70%), Recurring (7%), Adjustments (3%)
- **Temporal Patterns**: Month-end (2.5x), Quarter-end (4x), Year-end (6x) volume spikes with working hour patterns
- **Fraud Scenarios**: Configurable fraud injection including suspense account abuse, fictitious transactions, timing anomalies
- **Deterministic Generation**: Seeded RNG ensures reproducible output for testing
- **Multiple Output Formats**: CSV, Parquet, JSON with optional compression

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

**Industries**: `manufacturing`, `retail`, `financial_services`, `healthcare`, `technology`

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

## Configuration

The generator uses YAML configuration files with the following sections:

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

### Companies

```yaml
companies:
  - code: "1000"
    name: "US Manufacturing"
    currency: USD
    country: US
    annual_transaction_volume: hundred_k  # ten_k, hundred_k, one_m, ten_m, hundred_m
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

### Seasonality

```yaml
transactions:
  seasonality:
    month_end_spike: true
    month_end_multiplier: 2.5      # 2.5x normal volume
    quarter_end_multiplier: 4.0    # 4x normal volume
    year_end_multiplier: 6.0       # 6x normal volume
    weekend_activity: 0.1          # 10% of normal
    holiday_activity: 0.05         # 5% of normal
```

### Fraud Scenarios

```yaml
fraud:
  enabled: true
  fraud_rate: 0.005              # 0.5% of transactions
  fraud_type_distribution:
    suspense_account_abuse: 0.25
    fictitious_transaction: 0.15
    revenue_manipulation: 0.10
    timing_anomaly: 0.10
```

## Output Data Models

### Journal Entry

Each journal entry contains:

**Header:**
- `document_id`: UUID v4 (deterministic based on seed)
- `company_code`: Company identifier
- `fiscal_year`, `fiscal_period`: Accounting period
- `posting_date`, `document_date`: Transaction dates
- `created_at`: Timestamp with working hour patterns
- `source`: `manual`, `automated`, `recurring`, `adjustment`
- `user_persona`: `junior_accountant`, `senior_accountant`, `controller`, `automated_system`
- `business_process`: `O2C`, `P2P`, `R2R`, `H2R`, `A2R`

**Line Items:**
- `gl_account`: Account number from Chart of Accounts
- `debit_amount`, `credit_amount`: Transaction amounts (always balanced)
- `cost_center`, `profit_center`, `segment`: Optional dimensions

### Chart of Accounts

Hierarchical account structure with:
- Account types: Asset, Liability, Equity, Revenue, Expense
- Industry-specific sub-types and weights
- Configurable hierarchy depth (2-5 levels)

### ACDOCA (SAP HANA Universal Journal)

SAP-compatible event log format with fields:
- `RLDNR`: Ledger
- `RBUKRS`: Company code
- `GJAHR`, `MONAT`: Fiscal year/period
- `BELNR`: Document number
- `RACCT`: Account
- `HSL`, `MSL`: Amounts in local/group currency

## Architecture

```
SyntheticData/
├── Cargo.toml                    # Workspace root
└── crates/
    ├── synth-core/              # Domain models, traits, distributions
    │   ├── models/              # JournalEntry, GLAccount, ACDOCA, Company, User
    │   ├── distributions/       # LineItem, Amount, Temporal samplers
    │   └── traits/              # Generator, Sink interfaces
    ├── synth-config/            # Configuration schema and validation
    │   ├── schema.rs            # GeneratorConfig, TransactionConfig
    │   └── presets.rs           # Industry preset generation
    ├── synth-generators/        # Data generators
    │   ├── coa_generator.rs     # Chart of Accounts generator
    │   └── je_generator.rs      # Journal Entry generator
    ├── synth-output/            # Output sinks
    │   ├── csv_sink.rs          # CSV output
    │   ├── parquet_sink.rs      # Parquet output (placeholder)
    │   └── json_sink.rs         # JSON output
    ├── synth-runtime/           # Orchestration
    │   └── orchestrator.rs      # Generation coordination
    └── synth-cli/               # Command-line interface
        └── main.rs              # CLI entry point
```

## Use Cases

- **Process Mining**: Generate realistic event logs for process discovery and conformance checking
- **Analytics Testing**: Create large datasets for testing BI and analytics tools
- **Machine Learning**: Training data for fraud detection and anomaly detection models
- **System Testing**: Load testing for ERP and accounting systems
- **Education**: Realistic accounting data for training and demonstrations

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
