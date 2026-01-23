# Example: Cross-Border Data Sharing (GDPR Compliance)

This example demonstrates how to use fingerprinting to enable cross-border analytics while maintaining GDPR compliance.

---

## Scenario

**Acme Corp** is a multinational company with:
- EU subsidiary (Germany) with production ERP data
- US headquarters with a central analytics team
- Need to run fraud detection analytics on EU data
- Cannot transfer personal data outside EU due to GDPR

---

## Solution Architecture

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                              GERMANY (EU)                                   │
│                                                                             │
│  ┌─────────────────┐                                                        │
│  │ SAP ERP         │                                                        │
│  │ (Production)    │                                                        │
│  │                 │                                                        │
│  │ • Journal Entries: 5.2M rows                                            │
│  │ • Vendors: 12,000                                                        │
│  │ • Customers: 450,000                                                     │
│  │ • Employees: 2,500 (PII)                                                │
│  └────────┬────────┘                                                        │
│           │                                                                 │
│           ▼ Read-only access                                                │
│  ┌─────────────────┐     ┌─────────────────┐                               │
│  │ Data Governance │────▶│ Fingerprint     │                               │
│  │ Team            │     │ Extraction      │                               │
│  │                 │     │ (Weekly cron)   │                               │
│  └─────────────────┘     └────────┬────────┘                               │
│                                   │                                         │
│                                   ▼                                         │
│                          ┌─────────────────┐                               │
│                          │ acme_eu.dsf     │                               │
│                          │ (2.3 MB)        │                               │
│                          │                 │                               │
│                          │ Privacy: ε=0.5  │                               │
│                          │ k-anonymity: 10 │                               │
│                          └────────┬────────┘                               │
│                                   │                                         │
│                                   │ Signed with EU Data Team key            │
│                                   │                                         │
└───────────────────────────────────│─────────────────────────────────────────┘
                                    │
                          AWS S3 Transfer (Encrypted)
                                    │
┌───────────────────────────────────│─────────────────────────────────────────┐
│                              USA (HQ)                                       │
│                                   │                                         │
│                                   ▼                                         │
│                          ┌─────────────────┐                               │
│                          │ acme_eu.dsf     │                               │
│                          │ (Verified)      │                               │
│                          └────────┬────────┘                               │
│                                   │                                         │
│                                   ▼                                         │
│                          ┌─────────────────┐                               │
│                          │ DataSynth       │                               │
│                          │ Generator       │                               │
│                          └────────┬────────┘                               │
│                                   │                                         │
│                                   ▼                                         │
│                          ┌─────────────────┐                               │
│                          │ Synthetic Data  │                               │
│                          │ • 5.2M JEs      │                               │
│                          │ • Same patterns │                               │
│                          │ • Zero PII      │                               │
│                          └────────┬────────┘                               │
│                                   │                                         │
│           ┌───────────────────────┼───────────────────────┐                │
│           ▼                       ▼                       ▼                │
│  ┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐        │
│  │ Fraud Detection │    │ BI Dashboards   │    │ Ad-hoc Analysis │        │
│  │ ML Models       │    │                 │    │                 │        │
│  └─────────────────┘    └─────────────────┘    └─────────────────┘        │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## Step 1: EU - Extract Fingerprint

### Configuration File

```yaml
# /etc/datasynth/eu_extraction.yaml

input:
  type: "database"
  connection: "oracle://sap_readonly:${SAP_PASSWORD}@sap-prod.eu.acme.internal:1521/SAPDB"

  tables:
    - name: "BKPF"  # Accounting Document Header
      alias: "journal_headers"
      columns:
        include: ["BUKRS", "BELNR", "GJAHR", "BLDAT", "BUDAT", "USNAM", "WAERS"]
        exclude: ["XBLNR"]  # External reference (potentially sensitive)

    - name: "BSEG"  # Accounting Document Segment
      alias: "journal_lines"

    - name: "LFA1"  # Vendor Master
      alias: "vendors"
      columns:
        exclude: ["STRAS", "TELF1", "TELFX"]  # Address, phone, fax

    - name: "KNA1"  # Customer Master
      alias: "customers"
      columns:
        exclude: ["STRAS", "TELF1", "NAME1", "NAME2"]  # PII fields

  where:
    journal_headers: "GJAHR >= 2022"

output:
  path: "/secure/fingerprints/acme_eu_${DATE}.dsf"

privacy:
  level: "high"  # Sensitive financial data

  differential_privacy:
    enabled: true
    epsilon: 0.5  # Strong privacy

  k_anonymity:
    enabled: true
    k: 10  # Higher threshold for sensitive data

  suppression:
    always_suppress:
      - "*.STCD1"      # Tax ID
      - "*.STCD2"      # VAT number
      - "*.BANKS"      # Bank country
      - "*.BANKL"      # Bank key
      - "*.BANKN"      # Bank account
      - "vendors.TELF1"
      - "customers.NAME*"

    quasi_identifiers:
      - ["BUKRS", "KOSTL", "GJAHR", "MONAT"]  # Company, cost center, year, month

extraction:
  statistics:
    numeric:
      percentiles: [1, 5, 10, 25, 50, 75, 90, 95, 99]
      detect_distribution: true
      benford_analysis: true

    temporal:
      detect_seasonality: true
      granularity: "weekly"  # Not daily for privacy

  correlations:
    compute_pairwise: true
    min_samples: 100

  rules:
    infer_balance_equations: true
    infer_approval_thresholds: true
```

### Extraction Script

```bash
#!/bin/bash
# /opt/datasynth/scripts/weekly_extraction.sh

set -euo pipefail

DATE=$(date +%Y%m%d)
OUTPUT_DIR="/secure/fingerprints"
SIGNING_KEY="/etc/datasynth/keys/eu_data_team.pem"
S3_BUCKET="s3://acme-secure-transfer/fingerprints"

echo "[$(date)] Starting fingerprint extraction..."

# Extract fingerprint
datasynth-fingerprint extract \
    --config /etc/datasynth/eu_extraction.yaml \
    --output "${OUTPUT_DIR}/acme_eu_${DATE}.dsf" \
    --verbose

# Validate privacy compliance
echo "[$(date)] Validating privacy compliance..."
datasynth-fingerprint validate \
    "${OUTPUT_DIR}/acme_eu_${DATE}.dsf" \
    --strict

if [ $? -ne 0 ]; then
    echo "[$(date)] ERROR: Privacy validation failed!"
    exit 1
fi

# Sign the fingerprint
echo "[$(date)] Signing fingerprint..."
datasynth-fingerprint sign \
    "${OUTPUT_DIR}/acme_eu_${DATE}.dsf" \
    --key "${SIGNING_KEY}" \
    --signer "EU Data Governance Team"

# Upload to S3 (encrypted)
echo "[$(date)] Uploading to S3..."
aws s3 cp \
    "${OUTPUT_DIR}/acme_eu_${DATE}.dsf" \
    "${S3_BUCKET}/acme_eu_${DATE}.dsf" \
    --sse aws:kms \
    --sse-kms-key-id alias/acme-fingerprint-key

# Update latest symlink
aws s3 cp \
    "${S3_BUCKET}/acme_eu_${DATE}.dsf" \
    "${S3_BUCKET}/acme_eu_latest.dsf" \
    --sse aws:kms

echo "[$(date)] Fingerprint extraction complete!"

# Cleanup old fingerprints (keep 12 weeks)
find "${OUTPUT_DIR}" -name "acme_eu_*.dsf" -mtime +84 -delete
```

### Cron Schedule

```cron
# /etc/cron.d/datasynth-fingerprint
# Run every Sunday at 2:00 AM
0 2 * * 0 datasynth /opt/datasynth/scripts/weekly_extraction.sh >> /var/log/datasynth/extraction.log 2>&1
```

---

## Step 2: US - Generate Synthetic Data

### Configuration File

```yaml
# ~/analytics/generation_config.yaml

fingerprint:
  path: "./acme_eu_latest.dsf"
  verify_signature: true
  public_key: "./keys/eu_public_key.pem"

output:
  path: "./synthetic_eu_data/"
  format: "parquet"

  parquet:
    compression: "snappy"

  splitting:
    enabled: true
    max_rows_per_file: 1000000

generation:
  scale: 1.0  # Same volume as original
  seed: 20241215

  date_range:
    # Generate for current year (shifted from original)
    start: "2024-01-01"
    end: "2024-12-31"

  threads: 16

  correlations:
    enabled: true
    method: "copula"

  rules:
    balance_equations: true
    referential_integrity: true

  anomalies:
    source: "fingerprint"
    # Slightly higher rate for model training
    rate_multiplier: 1.5

overrides:
  companies:
    - code: "ACME_EU"
      name: "Acme Europe GmbH"
      currency: "EUR"
      country: "DE"
      volume_weight: 1.0
```

### Generation Script

```bash
#!/bin/bash
# ~/analytics/scripts/generate_synthetic.sh

set -euo pipefail

S3_BUCKET="s3://acme-secure-transfer/fingerprints"
LOCAL_DIR="./fingerprints"
OUTPUT_DIR="./synthetic_eu_data"

# Download latest fingerprint
echo "Downloading latest fingerprint..."
mkdir -p "${LOCAL_DIR}"
aws s3 cp \
    "${S3_BUCKET}/acme_eu_latest.dsf" \
    "${LOCAL_DIR}/acme_eu_latest.dsf"

# Verify signature
echo "Verifying fingerprint signature..."
datasynth-fingerprint verify \
    "${LOCAL_DIR}/acme_eu_latest.dsf" \
    --key ./keys/eu_public_key.pem

if [ $? -ne 0 ]; then
    echo "ERROR: Signature verification failed!"
    exit 1
fi

# Generate synthetic data
echo "Generating synthetic data..."
datasynth-data generate \
    --fingerprint "${LOCAL_DIR}/acme_eu_latest.dsf" \
    --output "${OUTPUT_DIR}/" \
    --config ./generation_config.yaml \
    --progress \
    --validate

# Evaluate fidelity
echo "Evaluating fidelity..."
datasynth-data evaluate \
    --fingerprint "${LOCAL_DIR}/acme_eu_latest.dsf" \
    --synthetic "${OUTPUT_DIR}/" \
    --output "./reports/fidelity_report.html" \
    --format html \
    --detailed

echo "Synthetic data generation complete!"
echo "Output: ${OUTPUT_DIR}/"
echo "Report: ./reports/fidelity_report.html"
```

---

## Step 3: Use Synthetic Data for Analytics

### Load into Analytics Environment

```python
# analytics/load_synthetic_data.py
import pandas as pd
from pathlib import Path

def load_synthetic_data(data_dir: str) -> dict[str, pd.DataFrame]:
    """Load synthetic data into pandas DataFrames."""
    data_dir = Path(data_dir)
    tables = {}

    for parquet_file in data_dir.glob("*.parquet"):
        table_name = parquet_file.stem
        tables[table_name] = pd.read_parquet(parquet_file)
        print(f"Loaded {table_name}: {len(tables[table_name]):,} rows")

    return tables

# Load data
data = load_synthetic_data("./synthetic_eu_data/")

# Use for analytics
journal_entries = data["journal_headers"]
print(f"Journal entries: {len(journal_entries):,}")
print(f"Date range: {journal_entries['posting_date'].min()} to {journal_entries['posting_date'].max()}")
print(f"Total amount: €{journal_entries['amount'].sum():,.2f}")
```

### Train Fraud Detection Model

```python
# analytics/train_fraud_model.py
import pandas as pd
from sklearn.ensemble import IsolationForest
from sklearn.model_selection import train_test_split
import mlflow

def train_fraud_model():
    # Load synthetic data
    df = pd.read_parquet("./synthetic_eu_data/journal_entries.parquet")

    # Feature engineering
    features = pd.DataFrame({
        'amount': df['amount'],
        'amount_log': np.log1p(df['amount']),
        'is_month_end': df['posting_date'].dt.day >= 28,
        'is_round_number': df['amount'] % 1000 == 0,
        'line_count': df['line_count'],
        'hour_posted': df['created_at'].dt.hour,
    })

    # Train model
    with mlflow.start_run():
        mlflow.log_param("data_source", "synthetic_fingerprint")
        mlflow.log_param("fingerprint", "acme_eu_latest.dsf")

        model = IsolationForest(
            contamination=0.02,  # Expected fraud rate
            random_state=42
        )
        model.fit(features)

        # Evaluate on synthetic test set
        predictions = model.predict(features)
        anomaly_rate = (predictions == -1).mean()

        mlflow.log_metric("synthetic_anomaly_rate", anomaly_rate)
        mlflow.sklearn.log_model(model, "fraud_model")

        print(f"Model trained. Anomaly rate: {anomaly_rate:.2%}")

if __name__ == "__main__":
    train_fraud_model()
```

---

## GDPR Compliance Documentation

### Data Protection Impact Assessment (DPIA) Excerpt

```markdown
## Data Flow: EU ERP to US Analytics

### Processing Activity
- **Purpose**: Fraud detection analytics and model training
- **Legal Basis**: Legitimate interest (fraud prevention)
- **Data Subjects**: Vendors, customers (business entities)

### Data Transfer Mechanism
- **Method**: Statistical fingerprint extraction
- **Data Transferred**: Aggregate statistics only
- **Personal Data**: None

### Privacy Safeguards
| Safeguard | Implementation |
|-----------|----------------|
| Data Minimization | Only aggregate statistics extracted |
| Differential Privacy | ε=0.5 (strong privacy guarantee) |
| k-Anonymity | k=10 (all groups have ≥10 records) |
| Field Suppression | Names, addresses, tax IDs excluded |
| Encryption | AES-256 in transit and at rest |
| Access Control | Signed fingerprints, key management |

### Risk Assessment
| Risk | Likelihood | Impact | Mitigation |
|------|------------|--------|------------|
| Re-identification | Very Low | High | DP + k-anonymity |
| Data breach | Low | Medium | Encryption, signing |
| Inference attack | Very Low | Medium | Aggregate stats only |

### Conclusion
The fingerprint-based approach does not constitute a transfer of personal
data under GDPR Article 4(1) as no information relating to identified or
identifiable natural persons is transferred.
```

---

## Monitoring and Alerts

### Extraction Monitoring

```yaml
# /etc/datasynth/monitoring.yaml
alerts:
  - name: "extraction_failure"
    condition: "exit_code != 0"
    channels: ["email:data-team@acme.com", "slack:#data-alerts"]

  - name: "privacy_warning"
    condition: "warnings > 0"
    channels: ["email:privacy@acme.com"]

  - name: "epsilon_budget_high"
    condition: "epsilon_spent > 0.9 * epsilon_budget"
    channels: ["email:data-team@acme.com"]

metrics:
  - name: "extraction_duration"
    type: "gauge"
  - name: "fingerprint_size_bytes"
    type: "gauge"
  - name: "epsilon_spent"
    type: "gauge"
  - name: "categories_suppressed"
    type: "counter"
```

### Dashboard

```sql
-- Fingerprint extraction metrics (for Grafana)
SELECT
    date_trunc('week', extraction_time) as week,
    avg(duration_seconds) as avg_duration,
    avg(fingerprint_size_mb) as avg_size,
    avg(epsilon_spent) as avg_epsilon,
    sum(categories_suppressed) as total_suppressed
FROM fingerprint_extractions
WHERE extraction_time > now() - interval '3 months'
GROUP BY 1
ORDER BY 1;
```

---

## Summary

This example demonstrates:

1. **Privacy-preserving extraction** with high privacy settings (ε=0.5, k=10)
2. **Secure transfer** with encryption and signing
3. **Signature verification** before use
4. **Synthetic data generation** matching original characteristics
5. **GDPR compliance** through aggregate statistics only
6. **Practical analytics** using synthetic data for ML training

The key insight is that statistical fingerprints are not personal data under GDPR, enabling cross-border analytics without data transfer restrictions.
