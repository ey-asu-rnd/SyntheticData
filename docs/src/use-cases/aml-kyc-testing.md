# AML/KYC Testing

Generate realistic banking transaction data with KYC profiles and AML typologies for compliance testing and fraud detection model development.

## Overview

The `datasynth-banking` module generates synthetic banking data designed for:

- **AML System Testing**: Validate transaction monitoring rules against known patterns
- **KYC Process Testing**: Test customer onboarding and risk assessment workflows
- **ML Model Training**: Train supervised models with labeled fraud typologies
- **Scenario Analysis**: Test detection capabilities against specific attack patterns

## KYC Profile Generation

### Customer Types

| Type | Description | Typical Characteristics |
|------|-------------|-------------------------|
| **Retail** | Individual customers | Salary deposits, consumer spending |
| **Business** | Small to medium businesses | Payroll, supplier payments |
| **Trust** | Trust accounts, complex structures | Investment flows, distributions |

### KYC Profile Components

Each customer has a KYC profile defining expected behavior:

```yaml
kyc_profile:
  declared_turnover: 50000        # Expected monthly volume
  transaction_frequency: 25       # Expected transactions/month
  source_of_funds: "employment"   # Declared income source
  geographic_exposure: ["US", "EU"]
  cash_intensity: 0.05            # Expected cash ratio
  beneficial_owner_complexity: 1  # Ownership layers
```

### Risk Scoring

Customers are assigned risk scores based on:
- Geographic exposure (high-risk jurisdictions)
- Industry sector
- Transaction patterns vs. declared profile
- Beneficial ownership complexity

## AML Typology Generation

### Structuring

Breaking large transactions into smaller amounts to avoid reporting thresholds.

```
Detection Signatures:
- Multiple transactions just below $10,000 threshold
- Same-day deposits across multiple branches
- Round-number amounts (e.g., $9,900, $9,800)
```

**Configuration:**
```yaml
typologies:
  structuring:
    enabled: true
    rate: 0.001
    threshold: 10000
    margin: 500
```

### Funnel Accounts

Concentrating funds from multiple sources before moving to destination.

```
Pattern:
Source A ─┐
Source B ─┼─▶ Funnel Account ─▶ Destination
Source C ─┘

Detection Signatures:
- Many small inbound, few large outbound
- High throughput relative to account balance
- Short holding periods
```

### Layering

Complex chains of transactions to obscure fund origins.

```
Pattern:
Origin ─▶ Shell A ─▶ Shell B ─▶ Shell C ─▶ Destination
                          └─▶ Mixing ─┘

Detection Signatures:
- Rapid consecutive transfers
- Circular transaction patterns
- Cross-border routing through multiple jurisdictions
```

### Money Mule Networks

Using recruited individuals to move illicit funds.

```
Pattern:
Fraudster ─▶ Mule 1 ─▶ Cash Withdrawal
           ─▶ Mule 2 ─▶ Wire Transfer
           ─▶ Mule 3 ─▶ Crypto Exchange

Detection Signatures:
- New accounts with sudden high volume
- Immediate outbound after inbound
- Multiple accounts with similar patterns
```

### Round-Tripping

Moving funds in circular patterns to create apparent legitimacy.

```
Pattern:
Company A ─▶ Offshore ─▶ Company A (as "investment")

Detection Signatures:
- Funds return to origin within short period
- Offshore intermediaries
- Inflated invoicing
```

### Fraud Patterns

Credit card fraud and synthetic identity patterns.

```
Patterns:
- Card testing (small amounts across merchants)
- Account takeover (changed behavior profile)
- Synthetic identity (blended PII attributes)
```

## Generated Data

### Output Files

```
banking/
├── banking_customers.csv        # Customer profiles with KYC data
├── bank_accounts.csv            # Account records with features
├── bank_transactions.csv        # Transaction records
├── kyc_profiles.csv             # Expected activity envelopes
├── counterparties.csv           # Counterparty pool
├── aml_typology_labels.csv      # Ground truth typology labels
├── entity_risk_labels.csv       # Entity-level risk classifications
└── transaction_risk_labels.csv  # Transaction-level classifications
```

### Customer Record

```csv
customer_id,customer_type,name,created_at,risk_score,kyc_status,pep_flag,sanctions_flag
CUST001,retail,John Smith,2024-01-15,25,verified,false,false
CUST002,business,Acme Corp,2024-02-01,65,enhanced_due_diligence,false,false
```

### Transaction Record

```csv
transaction_id,account_id,timestamp,amount,currency,direction,channel,category,counterparty_id
TXN001,ACC001,2024-03-15T10:30:00Z,9800.00,USD,credit,branch,cash_deposit,
TXN002,ACC001,2024-03-15T11:45:00Z,9750.00,USD,credit,branch,cash_deposit,
```

### Typology Label

```csv
transaction_id,typology,confidence,pattern_id,related_transactions
TXN001,structuring,0.95,STRUCT_001,"TXN001,TXN002,TXN003"
TXN002,structuring,0.95,STRUCT_001,"TXN001,TXN002,TXN003"
```

## Configuration

### Basic Banking Setup

```yaml
banking:
  enabled: true
  customers:
    retail: 5000
    business: 500
    trust: 50

  transactions:
    target_count: 500000
    date_range:
      start: 2024-01-01
      end: 2024-12-31

  typologies:
    structuring:
      enabled: true
      rate: 0.002
    funnel:
      enabled: true
      rate: 0.001
    layering:
      enabled: true
      rate: 0.0005
    mule:
      enabled: true
      rate: 0.001
    fraud:
      enabled: true
      rate: 0.005

  labels:
    generate: true
    include_confidence: true
    include_related: true
```

### Adversarial Testing

Generate transactions designed to evade detection:

```yaml
banking:
  typologies:
    spoofing:
      enabled: true
      strategies:
        - threshold_aware        # Varies amounts around thresholds
        - temporal_distribution  # Spreads over time windows
        - channel_mixing         # Uses multiple channels
```

## Use Cases

### Transaction Monitoring Rule Testing

```bash
# Generate data with known structuring patterns
datasynth-data generate --config banking_structuring.yaml --output ./test_data

# Expected results:
# - 0.2% of transactions should trigger structuring alerts
# - Labels in aml_typology_labels.csv for validation
```

### ML Model Training

```python
import pandas as pd
from sklearn.model_selection import train_test_split

# Load transactions and labels
transactions = pd.read_csv("banking/bank_transactions.csv")
labels = pd.read_csv("banking/aml_typology_labels.csv")

# Merge and prepare features
data = transactions.merge(labels, on="transaction_id", how="left")
data["is_suspicious"] = data["typology"].notna()

# Split for training
X_train, X_test, y_train, y_test = train_test_split(
    data[features],
    data["is_suspicious"],
    test_size=0.2,
    stratify=data["is_suspicious"]
)
```

### Network Analysis

The banking data supports graph-based analysis:

```python
import networkx as nx

# Build transaction network
G = nx.DiGraph()
for _, txn in transactions.iterrows():
    if txn["counterparty_id"]:
        G.add_edge(txn["account_id"], txn["counterparty_id"],
                   weight=txn["amount"])

# Detect funnel accounts (high in-degree, low out-degree)
in_degree = dict(G.in_degree())
out_degree = dict(G.out_degree())
funnels = [n for n in G.nodes()
           if in_degree.get(n, 0) > 10 and out_degree.get(n, 0) < 3]
```

### KYC Deviation Analysis

```python
# Compare actual behavior to KYC profile
customers = pd.read_csv("banking/banking_customers.csv")
kyc = pd.read_csv("banking/kyc_profiles.csv")
transactions = pd.read_csv("banking/bank_transactions.csv")

# Calculate actual monthly volumes
actual = transactions.groupby(["customer_id", "month"])["amount"].sum()

# Compare to declared turnover
merged = actual.merge(kyc, on="customer_id")
merged["deviation"] = (merged["actual"] - merged["declared_turnover"]) / merged["declared_turnover"]

# Flag significant deviations
alerts = merged[merged["deviation"].abs() > 0.5]
```

## Best Practices

### Realistic Testing

1. **Match production volumes**: Configure similar customer counts and transaction rates
2. **Use realistic ratios**: Keep typology rates at realistic levels (0.1-1%)
3. **Include noise**: Add legitimate edge cases that shouldn't trigger alerts

### Label Quality

1. **Verify ground truth**: Labels reflect injected patterns, not detected ones
2. **Include confidence**: Use confidence scores for uncertain classifications
3. **Track related transactions**: Pattern IDs link related suspicious activity

### Model Validation

1. **Test detection rates**: Measure recall against known patterns
2. **Check false positives**: Ensure legitimate transactions aren't flagged
3. **Validate across typologies**: Test each pattern type separately

## See Also

- [datasynth-banking Crate](../crates/datasynth-banking.md)
- [Fraud Detection ML](fraud-detection.md)
- [Graph Export](../advanced/graph-export.md)
- [Anomaly Injection](../advanced/anomaly-injection.md)
