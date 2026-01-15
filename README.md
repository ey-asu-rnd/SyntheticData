# SyntheticData

A high-performance, configurable synthetic data generator for enterprise accounting data. Produces realistic General Ledger Journal Entries, Chart of Accounts, and SAP HANA-compatible ACDOCA event logs at scale (10K to 100M+ transactions).

## Features

- **Realistic Statistical Distributions**: Line item counts, amounts, and patterns based on empirical research from real-world general ledger data
- **Benford's Law Compliance**: First-digit distribution following Benford's Law with configurable fraud patterns
- **Industry Presets**: Manufacturing, Retail, Financial Services, Healthcare, Technology, and more
- **Configurable Chart of Accounts**: Small (~100), Medium (~400), Large (~2500 accounts) with industry-specific account structures
- **Transaction Source Modeling**: Manual entries (20%), Automated/batch (70%), Recurring (7%), Adjustments (3%)
- **Temporal Patterns**: Month-end (2.5x), Quarter-end (4x), Year-end (6x) volume spikes with working hour patterns
- **Industry Seasonality**: Sector-specific patterns (Black Friday for retail, year-end for financial services, etc.)
- **Regional Holiday Calendars**: US, DE, GB, CN, JP, IN with lunar calendar support
- **Internal Controls System (ICS)**: SOX 404 compliant controls with segregation of duties
- **Fraud Scenarios**: Configurable fraud injection including suspense account abuse, fictitious transactions, timing anomalies
- **Weighted Company Selection**: Transaction volume distribution based on company size
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

The generator supports industry-specific seasonal patterns:

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

Support for regional holidays with reduced activity:

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
    custom_holidays:
      - name: "Company Holiday"
        month: 7
        day: 4
        multiplier: 0.02
```

**Supported Holidays:**
- **US**: New Year, MLK Day, Presidents Day, Memorial Day, Independence Day, Labor Day, Thanksgiving, Christmas
- **DE**: New Year, Epiphany, Easter, Labor Day, Ascension, Whit Monday, German Unity Day, Christmas
- **GB**: New Year, Easter, May Day, Spring/Summer Bank Holidays, Christmas, Boxing Day
- **CN**: New Year, Chinese New Year (lunar), Qingming, Labor Day, Dragon Boat, Mid-Autumn, National Day
- **JP**: New Year, Coming of Age, National Foundation, Vernal Equinox, Showa Day, Constitution Day, Children's Day
- **IN**: Republic Day, Independence Day, Gandhi Jayanti, Diwali (lunar)

### Base Seasonality

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

**Fraud Amount Patterns:**
- **Normal**: Benford-compliant amounts
- **StatisticallyImprobable**: Anti-Benford distribution (excess 5s, 7s, 9s)
- **ObviousRoundNumbers**: $50,000.00, $99,999.99
- **ThresholdAdjacent**: Just below approval limits (e.g., $9,999 when limit is $10,000)

### Internal Controls System (ICS)

Configure SOX 404 compliant internal controls:

```yaml
internal_controls:
  enabled: true
  exception_rate: 0.02           # 2% control exceptions
  sod_violation_rate: 0.01       # 1% segregation of duties violations
  sox_materiality_threshold: 50000.0
  export_control_master_data: true
```

**Control Types:**
- **Preventive**: Controls that prevent errors/fraud before they occur
- **Detective**: Controls that detect errors/fraud after occurrence
- **Monitoring**: Ongoing monitoring controls

**SOX Assertions:**
- Existence
- Completeness
- Valuation
- Rights and Obligations
- Presentation and Disclosure

**Segregation of Duties (SoD) Conflicts:**
- Preparer-Approver
- Requester-Approver
- Reconciler-Poster
- Payment-Releaser

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
- `is_fraud`: Boolean fraud indicator
- `fraud_type`: Type of fraud if applicable
- `control_ids`: Applied internal control IDs
- `sox_relevant`: Whether transaction is SOX material
- `control_status`: `Effective`, `Exception`, `NotTested`
- `sod_violation`: Boolean SoD violation indicator
- `sod_conflict_type`: Type of SoD conflict if applicable

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
- `ZSIM_CONTROL_IDS`: Applied control IDs
- `ZSIM_SOX_RELEVANT`: SOX relevance flag
- `ZSIM_CONTROL_STATUS`: Control effectiveness
- `ZSIM_SOD_VIOLATION`: SoD violation flag
- `ZSIM_SOD_CONFLICT`: SoD conflict type

### Internal Controls Master Data

When `export_control_master_data: true`, additional files are generated:

- `internal_controls.csv`: Control definitions with IDs, types, objectives, frequencies
- `control_account_mappings.csv`: Control-to-GL account mappings
- `control_process_mappings.csv`: Control-to-business process mappings
- `sod_conflict_pairs.csv`: SoD conflict definitions
- `sod_rules.csv`: SoD rule configurations

## Architecture

```
SyntheticData/
├── Cargo.toml                    # Workspace root
└── crates/
    ├── synth-core/              # Domain models, traits, distributions
    │   ├── models/              # JournalEntry, GLAccount, ACDOCA, Company, User
    │   │   ├── internal_control.rs   # SOX control definitions
    │   │   ├── control_mapping.rs    # Control-entity mappings
    │   │   └── sod.rs                # Segregation of duties
    │   ├── distributions/       # Statistical samplers
    │   │   ├── amount.rs        # Amount distribution with Benford
    │   │   ├── benford.rs       # Benford's Law sampler
    │   │   ├── seasonality.rs   # Industry seasonal patterns
    │   │   ├── holidays.rs      # Regional holiday calendars
    │   │   ├── temporal.rs      # Date/time patterns
    │   │   └── line_item.rs     # Line item count distribution
    │   └── traits/              # Generator, Sink interfaces
    ├── synth-config/            # Configuration schema and validation
    │   ├── schema.rs            # GeneratorConfig, TransactionConfig
    │   └── presets.rs           # Industry preset generation
    ├── synth-generators/        # Data generators
    │   ├── coa_generator.rs     # Chart of Accounts generator
    │   ├── je_generator.rs      # Journal Entry generator
    │   ├── control_generator.rs # Internal controls generator
    │   ├── company_selector.rs  # Weighted company selection
    │   └── user_generator.rs    # User/persona generator
    ├── synth-output/            # Output sinks
    │   ├── csv_sink.rs          # CSV output
    │   ├── parquet_sink.rs      # Parquet output
    │   ├── json_sink.rs         # JSON output
    │   └── control_export.rs    # Control master data export
    ├── synth-runtime/           # Orchestration
    │   └── orchestrator.rs      # Generation coordination
    └── synth-cli/               # Command-line interface
        └── main.rs              # CLI entry point
```

## Use Cases

- **Process Mining**: Generate realistic event logs for process discovery and conformance checking
- **Analytics Testing**: Create large datasets for testing BI and analytics tools
- **Machine Learning**: Training data for fraud detection and anomaly detection models
- **Audit Testing**: Realistic data with known fraud patterns for audit procedure testing
- **SOX Compliance Testing**: Test internal controls and segregation of duties monitoring
- **System Testing**: Load testing for ERP and accounting systems
- **Education**: Realistic accounting data for training and demonstrations

## Performance

- Single-threaded: ~100K+ entries/second
- Parallel: Scales with available cores
- Memory-efficient streaming for large volumes

## Example Configuration

```yaml
global:
  seed: 12345
  industry: retail
  start_date: 2024-01-01
  period_months: 12
  group_currency: USD

companies:
  - code: "1000"
    name: "US Headquarters"
    currency: USD
    country: US
    volume_weight: 1.0
  - code: "2000"
    name: "EU Operations"
    currency: EUR
    country: DE
    volume_weight: 0.5
  - code: "3000"
    name: "APAC Region"
    currency: JPY
    country: JP
    volume_weight: 0.3

transactions:
  benford:
    enabled: true
    exempt_sources: [recurring, payroll]
  enhanced_seasonality:
    use_industry_seasonality: true
    holiday_regions: [US, DE, JP]

fraud:
  enabled: true
  fraud_rate: 0.005
  approval_thresholds: [1000, 5000, 10000, 50000, 100000]

internal_controls:
  enabled: true
  exception_rate: 0.02
  sod_violation_rate: 0.01
  export_control_master_data: true
```

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
