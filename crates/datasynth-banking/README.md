# datasynth-banking

KYC/AML banking transaction generator for compliance testing and fraud detection ML.

## Overview

`datasynth-banking` provides realistic banking data generation for:

- **KYC/AML Testing**: Generate transaction data for compliance system validation
- **Fraud Detection ML**: Labeled data for supervised learning models
- **Stress Testing**: High-volume transaction generation for system testing
- **Typology Simulation**: Realistic AML typologies (structuring, layering, mule networks)

## Key Components

### Models (`models/`)

| Model | Description |
|-------|-------------|
| `BankingCustomer` | Retail, Business, Trust customer personas |
| `BankAccount` | Account types with feature sets |
| `BankTransaction` | Transaction records with direction/channel |
| `KycProfile` | Expected activity envelope (turnover, frequency, sources) |
| `CounterpartyPool` | Transaction counterparty management |
| `CaseNarrative` | Investigation and compliance narratives |

### Generators (`generators/`)

| Generator | Description |
|-----------|-------------|
| `customer_generator` | Customer with KYC profile generation |
| `account_generator` | Account creation with proper features |
| `transaction_generator` | Persona-based transaction generation |
| `counterparty_generator` | Counterparty pool management |

### AML Typologies (`typologies/`)

| Typology | Description |
|----------|-------------|
| `structuring` | Structuring below reporting thresholds |
| `funnel` | Funnel account patterns for layering |
| `layering` | Complex transaction layering schemes |
| `mule` | Money mule network patterns |
| `round_tripping` | Round-tripping schemes |
| `fraud` | Credit card fraud, synthetic identity fraud |
| `spoofing` | Adversarial transaction generation |

### Customer Personas (`personas/`)

| Persona | Description |
|---------|-------------|
| `retail` | Individual customer behavioral patterns |
| `business` | Business account patterns |
| `trust` | Trust/corporate patterns |

### Labels (`labels/`)

| Label Type | Description |
|------------|-------------|
| `entity_labels` | Entity-level ML labels |
| `relationship_labels` | Relationship risk labels |
| `transaction_labels` | Transaction classification labels |
| `narrative_generator` | Investigation narrative generation |

## Usage

```rust
use datasynth_banking::{BankingOrchestrator, BankingConfig};

let config = BankingConfig::default();
let mut orchestrator = BankingOrchestrator::new(config, seed);

// Generate banking data
let result = orchestrator.generate()?;

// Access generated data
println!("Customers: {}", result.customers.len());
println!("Transactions: {}", result.transactions.len());
println!("Suspicious labels: {}", result.labels.suspicious_count());
```

## Output Files

| File | Description |
|------|-------------|
| `banking_customers.csv` | Customer profiles with KYC data |
| `bank_accounts.csv` | Account records with features |
| `bank_transactions.csv` | Transaction records |
| `kyc_profiles.csv` | Expected activity envelopes |
| `counterparties.csv` | Counterparty pool |
| `aml_typology_labels.csv` | AML typology labels |
| `entity_risk_labels.csv` | Entity-level risk classifications |
| `transaction_risk_labels.csv` | Transaction-level classifications |

## License

Apache-2.0 - See [LICENSE](../../LICENSE) for details.
