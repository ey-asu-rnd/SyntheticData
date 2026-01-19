# synth-core

Core domain models, traits, and distributions for synthetic accounting data generation.

## Overview

`synth-core` provides the foundational building blocks for the SyntheticData workspace:

- **Domain Models**: Journal entries, chart of accounts, master data, documents, anomalies
- **Statistical Distributions**: Line item sampling, amount generation, temporal patterns
- **Core Traits**: Generator and Sink interfaces for extensibility
- **Template System**: File-based templates for regional/sector customization
- **Infrastructure**: UUID factory, memory guard, GL account constants

## Key Components

### Domain Models (`models/`)

| Module | Description |
|--------|-------------|
| `journal_entry.rs` | Journal entry header and balanced line items |
| `chart_of_accounts.rs` | Hierarchical GL accounts with account types |
| `master_data.rs` | Enhanced vendors, customers with payment behavior |
| `documents.rs` | Purchase orders, invoices, goods receipts, payments |
| `temporal.rs` | Bi-temporal data model for audit trails |
| `anomaly.rs` | Anomaly types and labels for ML training |
| `internal_control.rs` | SOX 404 control definitions |

### Statistical Distributions (`distributions/`)

| Distribution | Description |
|--------------|-------------|
| `LineItemSampler` | Empirical distribution (60.68% two-line, 88% even counts) |
| `AmountSampler` | Log-normal with round-number bias, Benford compliance |
| `TemporalSampler` | Seasonality patterns with industry integration |
| `BenfordSampler` | First-digit distribution following P(d) = log10(1 + 1/d) |

### Infrastructure

| Component | Description |
|-----------|-------------|
| `uuid_factory.rs` | Deterministic FNV-1a hash-based UUID generation |
| `memory_guard.rs` | Cross-platform memory tracking with soft/hard limits |
| `accounts.rs` | Centralized GL control account numbers |
| `templates/` | YAML/JSON template loading and merging |

## Usage

```rust
use synth_core::models::{JournalEntry, JournalEntryLine};
use synth_core::distributions::AmountSampler;

// Create a balanced journal entry
let mut entry = JournalEntry::new(header);
entry.add_line(JournalEntryLine::debit("1100", amount, "AR Invoice"));
entry.add_line(JournalEntryLine::credit("4000", amount, "Revenue"));

// Sample realistic amounts
let sampler = AmountSampler::new(seed);
let amount = sampler.sample_benford_compliant(1000.0, 100000.0);
```

## License

Apache-2.0 - See [LICENSE](../../LICENSE) for details.
