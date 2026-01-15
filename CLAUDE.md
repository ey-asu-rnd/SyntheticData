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

This is a Rust workspace with 6 crates following a layered architecture:

```
synth-cli          → Binary entry point (commands: generate, validate, init, info)
    ↓
synth-runtime      → Orchestration layer (GenerationOrchestrator coordinates workflow)
    ↓
synth-generators   → Data generators (CoA, JournalEntry, User, Control, CompanySelector)
    ↓
synth-config       → Configuration schema, validation, industry presets
    ↓
synth-core         → Domain models, traits, statistical distributions, templates
    ↓
synth-output       → Output sinks (CSV, JSON, Parquet, ControlExport)
```

### Key Domain Models (synth-core/src/models/)

- **JournalEntry**: Header + balanced line items (debits must equal credits)
- **ChartOfAccounts**: Hierarchical GL accounts with AccountType/AccountSubType
- **ACDOCA**: SAP HANA Universal Journal format with ZSIM_ simulation fields
- **User**: Persona-based users (junior_accountant, senior_accountant, controller, automated_system)
- **ApprovalWorkflow**: Threshold-based approval chains
- **MasterData**: Vendor/Customer pools for realistic references
- **InternalControl**: SOX 404 control definitions with control types and assertions
- **ControlMapping**: Control-to-entity mappings (accounts, processes, thresholds, document types)
- **SoD**: Segregation of Duties conflict types and violation records

### Statistical Distributions (synth-core/src/distributions/)

- **LineItemSampler**: Empirical distribution (60.68% two-line entries, 88% even line counts)
- **AmountSampler**: Log-normal with round-number bias (25% chance of .00 endings), Benford's Law compliance
- **TemporalSampler**: Seasonality patterns with industry and holiday integration
- **BenfordSampler**: First-digit distribution following Benford's Law P(d) = log10(1 + 1/d)
- **FraudAmountGenerator**: Suspicious amount patterns (threshold-adjacent, round numbers, anti-Benford)
- **IndustrySeasonality**: Industry-specific volume patterns for 10 sectors
- **HolidayCalendar**: Regional holidays for US, DE, GB, CN, JP, IN

### Templates (synth-core/src/templates/)

- **MultiCultureNameGenerator**: 7 cultures, 50+ names each for realistic user pools
- **DescriptionGenerator**: Business process-specific header/line text
- **ReferenceGenerator**: Invoice, PO, check number formats

### Core Traits (synth-core/src/traits/)

- **Generator**: `generate_batch()` and `generate_stream()` for data generation
- **Sink**: Output destination interface (CSV, JSON, Parquet implementations)

## Key Design Decisions

1. **Deterministic RNG**: Uses ChaCha8 with configurable seed for reproducible output
2. **Precise Decimals**: `rust_decimal` for financial calculations (no floating point)
3. **Balanced Entries**: JournalEntry enforces debits = credits at construction time
4. **Empirical Distributions**: Based on academic research on real GL data patterns
5. **Benford's Law**: Amount distribution follows first-digit law with fraud pattern exceptions
6. **Weighted Company Selection**: Companies selected based on volume_weight for realistic distribution

## Configuration Schema

Config files use YAML with sections: `global`, `companies`, `chart_of_accounts`, `transactions`, `output`, `fraud`, `internal_controls`, `business_processes`, `templates`, `approval`, `departments`.

Industry presets: manufacturing, retail, financial_services, healthcare, technology
Complexity levels: small (~100 accounts), medium (~400), large (~2500)

## New Features (Audit/Fraud/Compliance)

### Benford's Law Compliance

```yaml
transactions:
  benford:
    enabled: true
    tolerance: 0.1
    exempt_sources: [recurring, payroll]
```

- Amounts follow Benford's Law first-digit distribution
- Smart exemptions for payroll, recurring transactions
- Fraud patterns intentionally deviate for detection testing

### Fraud Amount Patterns

- **Normal**: Standard Benford-compliant amounts
- **StatisticallyImprobable**: Anti-Benford distribution (excess 5s, 7s, 9s)
- **ObviousRoundNumbers**: $50,000.00, $99,999.99
- **ThresholdAdjacent**: Just below approval limits ($9,999, $49,999)

### Internal Controls System (ICS)

```yaml
internal_controls:
  enabled: true
  exception_rate: 0.02      # 2% control exceptions
  sod_violation_rate: 0.01  # 1% SoD violations
  export_control_master_data: true
  sox_materiality_threshold: 10000
```

**Control Types**: Preventive, Detective, Monitoring
**SOX Assertions**: Existence, Completeness, Valuation, RightsAndObligations, PresentationAndDisclosure
**Control Status**: Effective, Exception, NotTested

Standard controls (C001-C060):
- C001: Cash reconciliation
- C002: Large transaction approval
- C010/C011: P2P controls
- C020/C021: O2C controls
- C030/C031/C032: GL controls
- C040: Payroll controls
- C050: Fixed asset controls
- C060: Intercompany controls

### Segregation of Duties (SoD)

**Conflict Types**:
- PreparerApprover: Same person prepared and approved
- RequesterApprover: Self-approved requests
- ReconcilerPoster: Reconciled and posted adjustments
- MasterDataMaintainer: Maintains vendor data and processes payments
- PaymentReleaser: Created and released payment
- JournalEntryPoster: Posted to sensitive accounts without review
- SystemAccessConflict: Multiple conflicting access roles

### Industry Seasonality

10 industries with specific seasonal patterns:

**Retail**: Black Friday 8x, Christmas 6x, Summer 0.7x
**Manufacturing**: Year-end 4x, Q4 buildup 2x, Summer shutdown 0.6x
**Financial Services**: Year-end 8x, Quarter-ends 5x, Tax deadline 3x
**Healthcare**: Year-end 3x, Open enrollment 2x, Summer 0.8x
**Technology**: Q4 enterprise deals 4x, Holiday sales 2x, Summer 0.7x

### Regional Holiday Calendars

Supported regions: US, DE (Germany), GB (UK), CN (China), JP (Japan), IN (India)

- Bank holidays with activity multipliers
- Lunar calendar holidays (Chinese New Year, Diwali)
- Proper calculation of floating holidays (Easter, Thanksgiving)

### Weighted Company Selection

```yaml
companies:
  - code: "1000"
    name: "US HQ"
    volume_weight: 1.0    # 50% of transactions
  - code: "2000"
    name: "EU Sub"
    volume_weight: 0.5    # 25% of transactions
  - code: "3000"
    name: "APAC"
    volume_weight: 0.5    # 25% of transactions
```

## Fraud Scenarios

10 configurable fraud types: SuspenseAccountAbuse, FictitiousTransaction, RevenueManipulation, ExpenseCapitalization, SplitTransaction, TimingAnomaly, UnauthorizedAccess, DuplicatePayment, GhostEmployee, KickbackScheme

Each fraud type maps to an amount pattern:
- SplitTransaction → ThresholdAdjacent (just below limits)
- FictitiousTransaction → ObviousRoundNumbers
- RevenueManipulation → StatisticallyImprobable (anti-Benford)

## Export Files

### Transaction Data
- `journal_entries.parquet` / `.csv` / `.json`
- `acdoca.parquet` - SAP HANA Universal Journal format

### Control Master Data (when `export_control_master_data: true`)
- `internal_controls.csv` - Control definitions
- `control_account_mappings.csv` - Control ↔ Account
- `control_process_mappings.csv` - Control ↔ Business Process
- `control_threshold_mappings.csv` - Control ↔ Amount thresholds
- `control_doctype_mappings.csv` - Control ↔ Document types
- `sod_conflict_pairs.csv` - SoD conflict definitions
- `sod_rules.csv` - SoD rule definitions

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

    // Fraud markers
    pub is_fraud: bool,
    pub fraud_type: Option<FraudType>,

    // Control markers (NEW)
    pub control_ids: Vec<String>,
    pub sox_relevant: bool,
    pub control_status: ControlStatus,
    pub sod_violation: bool,
    pub sod_conflict_type: Option<SodConflictType>,
}
```

## ACDOCA ZSIM_ Fields

Simulation metadata fields in ACDOCA output:
- `sim_batch_id`: Generation batch identifier
- `sim_is_fraud`: Fraud flag
- `sim_fraud_type`: Type of fraud if applicable
- `sim_business_process`: Business process category
- `sim_user_persona`: User persona who created entry
- `sim_je_uuid`: Original journal entry UUID
- `sim_control_ids`: Comma-separated control IDs
- `sim_sox_relevant`: SOX 404 relevance flag
- `sim_control_status`: Control effectiveness status
- `sim_sod_violation`: SoD violation flag
- `sim_sod_conflict`: SoD conflict type if violated
