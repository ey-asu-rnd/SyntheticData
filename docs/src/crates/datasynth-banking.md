# datasynth-banking

KYC/AML banking transaction generator for synthetic data.

## Overview

`datasynth-banking` provides comprehensive banking transaction simulation for:

- Compliance testing and model training
- AML/fraud detection system evaluation
- KYC process simulation
- Regulatory reporting testing

## Features

| Feature | Description |
|---------|-------------|
| Customer Generation | Retail, business, and trust customers with realistic KYC profiles |
| Account Generation | Multiple account types with proper feature sets |
| Transaction Engine | Persona-based transaction generation with causal drivers |
| AML Typologies | Structuring, funnel accounts, layering, mule networks, and more |
| Ground Truth Labels | Multi-level labels for ML training |
| Spoofing Mode | Adversarial transaction generation for robustness testing |

## Architecture

```
BankingOrchestrator (orchestration)
        |
Generators (customer, account, transaction, counterparty)
        |
Typologies (AML pattern injection)
        |
Labels (ground truth generation)
        |
Models (customer, account, transaction, KYC)
```

## Module Structure

### Models

| Model | Description |
|-------|-------------|
| `BankingCustomer` | Retail, Business, Trust customer types |
| `BankAccount` | Account with type, features, status |
| `BankTransaction` | Transaction with direction, channel, category |
| `KycProfile` | Expected activity envelope for compliance |
| `CounterpartyPool` | Transaction counterparty management |
| `CaseNarrative` | Investigation and compliance narratives |

### KYC Profile

```rust
pub struct KycProfile {
    pub declared_purpose: String,
    pub turnover_band: TurnoverBand,
    pub transaction_frequency: TransactionFrequency,
    pub expected_categories: Vec<TransactionCategory>,
    pub source_of_funds: SourceOfFunds,
    pub source_of_wealth: SourceOfWealth,
    pub geographic_exposure: Vec<String>,
    pub cash_intensity: CashIntensity,
    pub beneficial_owner_complexity: OwnerComplexity,
    // Ground truth fields
    pub is_deceiving: bool,
    pub actual_turnover_band: Option<TurnoverBand>,
}
```

### AML Typologies

| Typology | Description |
|----------|-------------|
| Structuring | Transactions below reporting thresholds ($10K) |
| Funnel Accounts | Multiple small deposits, few large withdrawals |
| Layering | Complex transaction chains to obscure origin |
| Mule Networks | Money mule payment chains |
| Round-Tripping | Circular transaction patterns |
| Credit Card Fraud | Fraudulent card transactions |
| Synthetic Identity | Fake identity transactions |
| Spoofing | Adversarial patterns for model testing |

### Labels

| Label Type | Description |
|------------|-------------|
| Entity Labels | Customer-level risk classifications |
| Relationship Labels | Relationship risk indicators |
| Transaction Labels | Transaction-level classifications |
| Narrative Labels | Investigation case narratives |

## Usage Examples

### Basic Generation

```rust
use synth_banking::{BankingOrchestrator, BankingConfig};

let config = BankingConfig::default();
let mut orchestrator = BankingOrchestrator::new(config, 12345);

// Generate customers and accounts
let customers = orchestrator.generate_customers();
let accounts = orchestrator.generate_accounts(&customers);

// Generate transaction stream
let transactions = orchestrator.generate_transactions(&accounts);
```

### With AML Typologies

```rust
use synth_banking::{BankingConfig, TypologyConfig};

let config = BankingConfig {
    customer_count: 1000,
    typologies: TypologyConfig {
        structuring_rate: 0.02,   // 2% structuring patterns
        funnel_rate: 0.01,        // 1% funnel accounts
        mule_rate: 0.005,         // 0.5% mule networks
        ..Default::default()
    },
    ..Default::default()
};
```

### Accessing Labels

```rust
let labels = orchestrator.generate_labels();

// Entity-level labels
for entity_label in &labels.entity_labels {
    println!("Customer {} risk: {:?}",
        entity_label.customer_id,
        entity_label.risk_tier
    );
}

// Transaction-level labels
for tx_label in &labels.transaction_labels {
    if tx_label.is_suspicious {
        println!("Suspicious: {} - {:?}",
            tx_label.transaction_id,
            tx_label.typology
        );
    }
}
```

## Key Types

### Customer Types

```rust
pub enum BankingCustomerType {
    Retail,     // Individual customers
    Business,   // Business accounts
    Trust,      // Trust/corporate entities
}
```

### Risk Tiers

```rust
pub enum RiskTier {
    Low,
    Medium,
    High,
    Prohibited,
}
```

### Transaction Categories

```rust
pub enum TransactionCategory {
    SalaryWages,
    BusinessPayment,
    Investment,
    RealEstate,
    Gambling,
    Cryptocurrency,
    CashDeposit,
    CashWithdrawal,
    WireTransfer,
    AtmWithdrawal,
    PosPayment,
    OnlinePayment,
    // ... more categories
}
```

### AML Typologies

```rust
pub enum AmlTypology {
    Structuring,
    Funnel,
    Layering,
    Mule,
    RoundTripping,
    CreditCardFraud,
    SyntheticIdentity,
    None,
}
```

## Export Files

| File | Description |
|------|-------------|
| `banking_customers.csv` | Customer profiles with KYC data |
| `bank_accounts.csv` | Account records with features |
| `bank_transactions.csv` | Transaction records |
| `kyc_profiles.csv` | Expected activity envelopes |
| `counterparties.csv` | Counterparty pool |
| `entity_risk_labels.csv` | Entity-level risk classifications |
| `transaction_risk_labels.csv` | Transaction-level labels |
| `aml_typology_labels.csv` | AML typology ground truth |

## See Also

- [datasynth-core](datasynth-core.md) - Core banking models
- [Fraud Detection Use Case](../use-cases/fraud-detection.md)
- [Anomaly Injection](../advanced/anomaly-injection.md)
