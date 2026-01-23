# Integration Patterns

This guide covers common integration patterns for using fingerprinting in real-world scenarios.

---

## Pattern 1: Cross-Border Data Sharing

### Scenario

A multinational company needs to share ERP data with a US-based analytics team, but the data originates from EU subsidiaries subject to GDPR.

### Architecture

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                         EU DATA CENTER (GDPR Region)                        │
│                                                                             │
│  ┌──────────────────┐                                                       │
│  │ Production ERP   │                                                       │
│  │ (SAP/Oracle)     │                                                       │
│  └────────┬─────────┘                                                       │
│           │                                                                 │
│           ▼                                                                 │
│  ┌──────────────────┐     ┌──────────────────┐                             │
│  │ Read-Only        │────▶│ Fingerprint      │                             │
│  │ Replica          │     │ Extractor        │                             │
│  └──────────────────┘     └────────┬─────────┘                             │
│                                    │                                        │
│                                    ▼                                        │
│                           ┌──────────────────┐                             │
│                           │ .dsf Fingerprint │                             │
│                           │ (No PII)         │                             │
│                           └────────┬─────────┘                             │
│                                    │                                        │
└────────────────────────────────────┼────────────────────────────────────────┘
                                     │
                          SECURE TRANSFER
                          (Encrypted, Signed)
                                     │
┌────────────────────────────────────┼────────────────────────────────────────┐
│                         US DATA CENTER                                      │
│                                    │                                        │
│                                    ▼                                        │
│                           ┌──────────────────┐                             │
│                           │ .dsf Fingerprint │                             │
│                           │ (Received)       │                             │
│                           └────────┬─────────┘                             │
│                                    │                                        │
│                                    ▼                                        │
│                           ┌──────────────────┐                             │
│                           │ DataSynth        │                             │
│                           │ Generator        │                             │
│                           └────────┬─────────┘                             │
│                                    │                                        │
│                                    ▼                                        │
│                           ┌──────────────────┐                             │
│                           │ Synthetic Data   │                             │
│                           │ (Analytics-Ready)│                             │
│                           └────────┬─────────┘                             │
│                                    │                                        │
│           ┌────────────────────────┼────────────────────────┐              │
│           ▼                        ▼                        ▼              │
│  ┌──────────────────┐    ┌──────────────────┐    ┌──────────────────┐     │
│  │ ML Training      │    │ BI Dashboards    │    │ Ad-Hoc Analytics │     │
│  └──────────────────┘    └──────────────────┘    └──────────────────┘     │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

### Implementation

**EU Side (Data Owner):**

```bash
#!/bin/bash
# extract_fingerprint.sh - Run weekly via cron

# Connect to read-only replica
datasynth-fingerprint extract \
    --connection "postgresql://readonly:${DB_PASS}@replica.eu.internal/erp" \
    --tables "gl_journal_entries,ap_vendors,ar_customers,gl_accounts" \
    --where "fiscal_year >= 2023" \
    --output "/secure/fingerprints/erp_$(date +%Y%m%d).dsf" \
    --privacy-level high \
    --config /etc/datasynth/eu_extraction.yaml

# Sign the fingerprint
datasynth-fingerprint sign \
    --input "/secure/fingerprints/erp_$(date +%Y%m%d).dsf" \
    --key /etc/datasynth/signing_key.pem

# Upload to secure transfer
aws s3 cp \
    "/secure/fingerprints/erp_$(date +%Y%m%d).dsf" \
    "s3://secure-transfer-bucket/fingerprints/" \
    --sse aws:kms
```

**US Side (Data Consumer):**

```bash
#!/bin/bash
# generate_synthetic.sh - Run after fingerprint received

# Download latest fingerprint
LATEST=$(aws s3 ls s3://secure-transfer-bucket/fingerprints/ | sort | tail -1 | awk '{print $4}')
aws s3 cp "s3://secure-transfer-bucket/fingerprints/${LATEST}" /data/fingerprints/

# Verify signature
datasynth-fingerprint verify \
    --input "/data/fingerprints/${LATEST}" \
    --key /etc/datasynth/eu_public_key.pem

# Generate synthetic data
datasynth-data generate \
    --fingerprint "/data/fingerprints/${LATEST}" \
    --output /data/synthetic/erp/ \
    --format parquet \
    --validate
```

### GDPR Compliance Notes

- Fingerprints contain no personal data (Article 4 definition)
- Aggregate statistics are not personal data (Recital 26)
- Document the legal basis and data flow in your DPIA

---

## Pattern 2: Vendor Software Testing

### Scenario

A software vendor needs realistic test data to develop and test a new ERP module, but the client cannot share production data.

### Architecture

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                              CLIENT SITE                                    │
│                                                                             │
│  ┌──────────────────┐     ┌──────────────────┐                             │
│  │ Production ERP   │────▶│ Fingerprint Tool │                             │
│  │                  │     │ (One-time run)   │                             │
│  └──────────────────┘     └────────┬─────────┘                             │
│                                    │                                        │
│                                    ▼                                        │
│                           ┌──────────────────┐                             │
│                           │ .dsf Fingerprint │─────────────────────┐       │
│                           └──────────────────┘                     │       │
│                                                                    │       │
└────────────────────────────────────────────────────────────────────│───────┘
                                                                     │
                                                        Secure Email/Portal
                                                                     │
┌────────────────────────────────────────────────────────────────────│───────┐
│                           VENDOR SITE                              │       │
│                                                                    ▼       │
│                           ┌──────────────────┐                             │
│                           │ .dsf Fingerprint │                             │
│                           └────────┬─────────┘                             │
│                                    │                                        │
│           ┌────────────────────────┴────────────────────────┐              │
│           ▼                                                 ▼              │
│  ┌──────────────────┐                              ┌──────────────────┐    │
│  │ DEV Environment  │                              │ QA Environment   │    │
│  │                  │                              │                  │    │
│  │ datasynth        │                              │ datasynth        │    │
│  │ --scale 0.1      │                              │ --scale 1.0      │    │
│  │                  │                              │                  │    │
│  │ 100K records     │                              │ 1M records       │    │
│  │ Quick iteration  │                              │ Full-scale test  │    │
│  └──────────────────┘                              └──────────────────┘    │
│           │                                                 │              │
│           │            ┌──────────────────┐                │              │
│           └───────────▶│ Staging/UAT      │◀───────────────┘              │
│                        │                  │                                │
│                        │ Final testing    │                                │
│                        │ with client      │                                │
│                        └──────────────────┘                                │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

### Implementation

**Client Provides:**

```yaml
# client_extraction.yaml
input:
  connection: "oracle://erp_readonly:pass@erp-prod/ERPDB"
  tables:
    - "GL.JOURNAL_ENTRIES"
    - "AP.VENDORS"
    - "AR.CUSTOMERS"
    - "GL.CHART_OF_ACCOUNTS"

output:
  path: "./vendor_fingerprint.dsf"

privacy:
  level: "high"

  suppression:
    always_suppress:
      - "*.EMAIL"
      - "*.PHONE"
      - "*.SSN"
      - "*.TAX_ID"
      - "VENDORS.BANK_ACCOUNT"
```

**Vendor Development Environment:**

```yaml
# dev_generation.yaml
fingerprint:
  path: "./client_fingerprint.dsf"

output:
  path: "./dev_data/"
  format: "csv"

generation:
  scale: 0.1  # 10% for fast iteration
  seed: 12345  # Reproducible for debugging
```

**Vendor QA Environment:**

```yaml
# qa_generation.yaml
fingerprint:
  path: "./client_fingerprint.dsf"

output:
  path: "./qa_data/"
  format: "parquet"

generation:
  scale: 1.0  # Full scale
  threads: 8

  # Add extra edge cases for testing
  anomalies:
    rate_multiplier: 2.0  # More edge cases
```

---

## Pattern 3: ML Model Development Pipeline

### Scenario

A data science team needs to train fraud detection models but cannot access production data due to security policies.

### Architecture

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                         ML PIPELINE                                         │
│                                                                             │
│  ┌──────────────────────────────────────────────────────────────────────┐  │
│  │                    DATA PREPARATION                                   │  │
│  │                                                                       │  │
│  │  ┌─────────────┐     ┌─────────────┐     ┌─────────────┐            │  │
│  │  │ Production  │────▶│ Fingerprint │────▶│ .dsf File   │            │  │
│  │  │ Data        │     │ Extraction  │     │             │            │  │
│  │  └─────────────┘     └─────────────┘     └──────┬──────┘            │  │
│  │                                                  │                   │  │
│  └──────────────────────────────────────────────────│───────────────────┘  │
│                                                     │                       │
│  ┌──────────────────────────────────────────────────│───────────────────┐  │
│  │                    DATA GENERATION               │                   │  │
│  │                                                  ▼                   │  │
│  │  ┌─────────────────────────────────────────────────────────────┐    │  │
│  │  │                   DataSynth Generator                        │    │  │
│  │  └─────────────────────────────────────────────────────────────┘    │  │
│  │           │                    │                    │               │  │
│  │           ▼                    ▼                    ▼               │  │
│  │  ┌─────────────┐     ┌─────────────┐     ┌─────────────┐          │  │
│  │  │ Training    │     │ Validation  │     │ Test        │          │  │
│  │  │ Data (70%)  │     │ Data (15%)  │     │ Data (15%)  │          │  │
│  │  │             │     │             │     │             │          │  │
│  │  │ High fraud  │     │ Normal      │     │ Holdout     │          │  │
│  │  │ rate (5%)   │     │ fraud rate  │     │ evaluation  │          │  │
│  │  └─────────────┘     └─────────────┘     └─────────────┘          │  │
│  │                                                                    │  │
│  └────────────────────────────────────────────────────────────────────┘  │
│                                                                          │
│  ┌────────────────────────────────────────────────────────────────────┐  │
│  │                    MODEL TRAINING                                   │  │
│  │                                                                     │  │
│  │  ┌─────────────┐     ┌─────────────┐     ┌─────────────┐          │  │
│  │  │ Feature     │────▶│ Model       │────▶│ Hyperparameter│        │  │
│  │  │ Engineering │     │ Training    │     │ Tuning       │          │  │
│  │  └─────────────┘     └─────────────┘     └─────────────┘          │  │
│  │                                                                     │  │
│  └────────────────────────────────────────────────────────────────────┘  │
│                                                                          │
│  ┌────────────────────────────────────────────────────────────────────┐  │
│  │                    MODEL VALIDATION                                 │  │
│  │                                                                     │  │
│  │  ┌─────────────┐     ┌─────────────┐     ┌─────────────┐          │  │
│  │  │ Test on     │────▶│ Deploy to   │────▶│ Monitor on  │          │  │
│  │  │ Synthetic   │     │ Production  │     │ Real Data   │          │  │
│  │  └─────────────┘     └─────────────┘     └─────────────┘          │  │
│  │                                                                     │  │
│  └────────────────────────────────────────────────────────────────────┘  │
│                                                                          │
└──────────────────────────────────────────────────────────────────────────┘
```

### Implementation

**Data Generation Script:**

```python
# generate_ml_data.py
from datasynth_py import DataSynth, FingerprintConfig

def generate_ml_datasets(fingerprint_path: str, output_dir: str):
    synth = DataSynth()

    # Training data: Higher fraud rate for model learning
    synth.generate(
        fingerprint=fingerprint_path,
        output={
            "path": f"{output_dir}/train/",
            "format": "parquet"
        },
        overrides={
            "scale": 0.7,
            "seed": 42,
            "anomaly_injection": {
                "fraud_rate": 0.05,  # 5% fraud for training
                "rate_multiplier": 5.0
            }
        }
    )

    # Validation data: Normal fraud rate
    synth.generate(
        fingerprint=fingerprint_path,
        output={
            "path": f"{output_dir}/validation/",
            "format": "parquet"
        },
        overrides={
            "scale": 0.15,
            "seed": 43,
            "anomaly_injection": {
                "source": "fingerprint"  # Use real-world rate
            }
        }
    )

    # Test data: Normal fraud rate, different seed
    synth.generate(
        fingerprint=fingerprint_path,
        output={
            "path": f"{output_dir}/test/",
            "format": "parquet"
        },
        overrides={
            "scale": 0.15,
            "seed": 44,
            "anomaly_injection": {
                "source": "fingerprint"
            }
        }
    )

if __name__ == "__main__":
    generate_ml_datasets(
        fingerprint_path="./erp_fingerprint.dsf",
        output_dir="./ml_data/"
    )
```

**Model Training Pipeline (MLflow example):**

```python
# train_fraud_model.py
import mlflow
import pandas as pd
from sklearn.ensemble import IsolationForest

def train_fraud_model():
    mlflow.set_experiment("fraud_detection_synthetic")

    with mlflow.start_run():
        # Load synthetic training data
        train_df = pd.read_parquet("./ml_data/train/")

        # Log data generation parameters
        mlflow.log_param("data_source", "synthetic_fingerprint")
        mlflow.log_param("fingerprint_version", "erp_fingerprint_v2.dsf")
        mlflow.log_param("fraud_rate", 0.05)

        # Feature engineering
        features = engineer_features(train_df)

        # Train model
        model = IsolationForest(
            contamination=0.05,
            random_state=42
        )
        model.fit(features)

        # Validate on synthetic validation set
        val_df = pd.read_parquet("./ml_data/validation/")
        val_features = engineer_features(val_df)
        val_predictions = model.predict(val_features)

        # Calculate metrics
        precision, recall, f1 = calculate_metrics(
            val_df["is_fraud"],
            val_predictions
        )

        mlflow.log_metric("val_precision", precision)
        mlflow.log_metric("val_recall", recall)
        mlflow.log_metric("val_f1", f1)

        # Save model
        mlflow.sklearn.log_model(model, "fraud_model")
```

---

## Pattern 4: Audit Procedure Development

### Scenario

An audit firm needs to develop and test analytical audit procedures using realistic data, but cannot use actual client data outside of engagements.

### Architecture

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                       AUDIT FIRM WORKFLOW                                   │
│                                                                             │
│  ┌──────────────────────────────────────────────────────────────────────┐  │
│  │                 CLIENT ENGAGEMENT (Secure)                            │  │
│  │                                                                       │  │
│  │  ┌─────────────┐                      ┌─────────────┐                │  │
│  │  │ Client ERP  │─────────────────────▶│ Fingerprint │                │  │
│  │  │ Data        │  On-site extraction  │ .dsf        │                │  │
│  │  └─────────────┘                      └──────┬──────┘                │  │
│  │                                              │                        │  │
│  └──────────────────────────────────────────────│────────────────────────┘  │
│                                                 │                            │
│                                    Secure transfer to firm                   │
│                                                 │                            │
│  ┌──────────────────────────────────────────────│────────────────────────┐  │
│  │                AUDIT FIRM (Development)      │                        │  │
│  │                                              ▼                        │  │
│  │  ┌────────────────────────────────────────────────────────────────┐  │  │
│  │  │                    FINGERPRINT LIBRARY                          │  │  │
│  │  │                                                                 │  │  │
│  │  │  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌──────────┐       │  │  │
│  │  │  │ Retail   │  │ Manufact.│  │ Financial│  │ Healthcare│       │  │  │
│  │  │  │ Industry │  │ Industry │  │ Services │  │ Industry  │       │  │  │
│  │  │  └──────────┘  └──────────┘  └──────────┘  └──────────┘       │  │  │
│  │  │                                                                 │  │  │
│  │  └────────────────────────────────────────────────────────────────┘  │  │
│  │                              │                                        │  │
│  │           ┌──────────────────┴──────────────────┐                    │  │
│  │           ▼                                     ▼                    │  │
│  │  ┌─────────────────┐               ┌─────────────────┐              │  │
│  │  │ Procedure       │               │ Staff Training  │              │  │
│  │  │ Development     │               │                 │              │  │
│  │  │                 │               │ • Realistic data│              │  │
│  │  │ • Test scripts  │               │ • Safe to share │              │  │
│  │  │ • Validate logic│               │ • No NDA issues │              │  │
│  │  │ • Performance   │               │                 │              │  │
│  │  └─────────────────┘               └─────────────────┘              │  │
│  │                                                                       │  │
│  └───────────────────────────────────────────────────────────────────────┘  │
│                                                                             │
│  ┌───────────────────────────────────────────────────────────────────────┐  │
│  │                CLIENT ENGAGEMENT (Deployment)                         │  │
│  │                                                                       │  │
│  │  Developed procedures deployed on actual client data                  │  │
│  │                                                                       │  │
│  └───────────────────────────────────────────────────────────────────────┘  │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

### Implementation

**Fingerprint Library Manager:**

```yaml
# fingerprint_library.yaml
library:
  path: "/audit_firm/fingerprint_library/"

  industries:
    retail:
      - name: "retail_large_multinational"
        fingerprint: "retail_large_v2.dsf"
        description: "Large retail chain, 500+ stores, multi-currency"
        row_counts:
          journal_entries: 5_000_000
          vendors: 12_000
          customers: 2_000_000

      - name: "retail_ecommerce"
        fingerprint: "retail_ecom_v1.dsf"
        description: "E-commerce retailer, high volume, single currency"

    manufacturing:
      - name: "manufacturing_discrete"
        fingerprint: "mfg_discrete_v1.dsf"
        description: "Discrete manufacturing, BOM, inventory heavy"

      - name: "manufacturing_process"
        fingerprint: "mfg_process_v1.dsf"
        description: "Process manufacturing, batch tracking"

    financial_services:
      - name: "bank_regional"
        fingerprint: "bank_regional_v1.dsf"
        description: "Regional bank, deposits, loans, KYC data"
```

**Procedure Development Script:**

```python
# develop_audit_procedure.py
from datasynth_py import DataSynth

def develop_journal_entry_testing():
    """
    Develop journal entry testing procedures using synthetic data.
    """
    synth = DataSynth()

    # Generate test data from retail fingerprint
    result = synth.generate(
        fingerprint="./fingerprint_library/retail_large_v2.dsf",
        output={
            "format": "memory"  # Load directly into pandas
        },
        overrides={
            "scale": 0.1,  # 10% for development
            "anomaly_injection": {
                "fraud_rate": 0.02  # Include fraud patterns
            }
        }
    )

    journal_entries = result.tables["journal_entries"]

    # Develop and test audit procedures
    procedures = [
        test_duplicate_entries,
        test_round_dollar_amounts,
        test_weekend_postings,
        test_threshold_violations,
        test_unusual_accounts,
    ]

    for procedure in procedures:
        findings = procedure(journal_entries)
        print(f"{procedure.__name__}: {len(findings)} findings")

        # Validate against known anomalies
        validate_findings(findings, journal_entries)

def test_duplicate_entries(df):
    """Test for potential duplicate journal entries."""
    duplicates = df.groupby([
        'posting_date', 'amount', 'account_number'
    ]).filter(lambda x: len(x) > 1)
    return duplicates

def test_round_dollar_amounts(df):
    """Test for unusual concentration of round dollar amounts."""
    round_amounts = df[df['amount'] % 1000 == 0]
    if len(round_amounts) / len(df) > 0.20:  # More than 20%
        return round_amounts
    return pd.DataFrame()
```

---

## Pattern 5: CI/CD Integration Testing

### Scenario

An organization wants to run integration tests against realistic data in their CI/CD pipeline without exposing production data.

### Architecture

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                           CI/CD PIPELINE                                    │
│                                                                             │
│  ┌─────────────┐     ┌─────────────┐     ┌─────────────┐                   │
│  │ Code Push   │────▶│ Build       │────▶│ Unit Tests  │                   │
│  │             │     │             │     │             │                   │
│  └─────────────┘     └─────────────┘     └─────────────┘                   │
│                                                 │                           │
│                                                 ▼                           │
│  ┌───────────────────────────────────────────────────────────────────────┐ │
│  │                    INTEGRATION TEST STAGE                             │ │
│  │                                                                       │ │
│  │  ┌─────────────┐     ┌─────────────┐     ┌─────────────┐             │ │
│  │  │ Fingerprint │────▶│ DataSynth   │────▶│ Test        │             │ │
│  │  │ (Cached)    │     │ Generate    │     │ Database    │             │ │
│  │  └─────────────┘     └─────────────┘     └─────────────┘             │ │
│  │                                                 │                     │ │
│  │                                                 ▼                     │ │
│  │                                          ┌─────────────┐             │ │
│  │                                          │ Integration │             │ │
│  │                                          │ Tests       │             │ │
│  │                                          └─────────────┘             │ │
│  │                                                                       │ │
│  └───────────────────────────────────────────────────────────────────────┘ │
│                                                 │                           │
│                                                 ▼                           │
│  ┌─────────────┐     ┌─────────────┐     ┌─────────────┐                   │
│  │ Deploy to   │────▶│ E2E Tests   │────▶│ Deploy to   │                   │
│  │ Staging     │     │             │     │ Production  │                   │
│  └─────────────┘     └─────────────┘     └─────────────┘                   │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

### Implementation

**GitHub Actions Workflow:**

```yaml
# .github/workflows/integration-tests.yml
name: Integration Tests with Synthetic Data

on:
  push:
    branches: [main, develop]
  pull_request:
    branches: [main]

jobs:
  integration-tests:
    runs-on: ubuntu-latest

    services:
      postgres:
        image: postgres:15
        env:
          POSTGRES_DB: test_db
          POSTGRES_USER: test
          POSTGRES_PASSWORD: test
        ports:
          - 5432:5432

    steps:
      - uses: actions/checkout@v4

      - name: Cache fingerprint
        uses: actions/cache@v3
        with:
          path: ./test_fingerprint.dsf
          key: fingerprint-${{ hashFiles('fingerprint-version.txt') }}

      - name: Install DataSynth
        run: |
          curl -sSL https://datasynth.io/install.sh | sh
          datasynth-data --version

      - name: Generate test data
        run: |
          datasynth-data generate \
            --fingerprint ./test_fingerprint.dsf \
            --output ./test_data/ \
            --scale 0.01 \
            --seed ${{ github.run_number }} \
            --format csv

      - name: Load test data into database
        run: |
          for file in ./test_data/*.csv; do
            table=$(basename $file .csv)
            psql -h localhost -U test -d test_db \
              -c "\COPY $table FROM '$file' CSV HEADER"
          done
        env:
          PGPASSWORD: test

      - name: Run integration tests
        run: |
          npm run test:integration
        env:
          DATABASE_URL: postgresql://test:test@localhost:5432/test_db

      - name: Upload test results
        uses: actions/upload-artifact@v3
        if: always()
        with:
          name: test-results
          path: ./test-results/
```

**Docker Compose for Local Development:**

```yaml
# docker-compose.test.yml
version: '3.8'

services:
  datasynth:
    image: datasynth/datasynth:latest
    volumes:
      - ./fingerprint.dsf:/data/fingerprint.dsf:ro
      - ./synthetic_data:/output
    command: >
      generate
      --fingerprint /data/fingerprint.dsf
      --output /output/
      --scale 0.1
      --seed 42

  postgres:
    image: postgres:15
    environment:
      POSTGRES_DB: test_db
      POSTGRES_USER: test
      POSTGRES_PASSWORD: test
    volumes:
      - ./synthetic_data:/docker-entrypoint-initdb.d
    depends_on:
      datasynth:
        condition: service_completed_successfully

  app:
    build: .
    environment:
      DATABASE_URL: postgresql://test:test@postgres:5432/test_db
    depends_on:
      - postgres
```

---

## Pattern 6: Data Mesh / Federated Analytics

### Scenario

Multiple business units have their own data domains but need to share statistical properties for cross-domain analytics without centralizing sensitive data.

### Architecture

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                         FEDERATED FINGERPRINT ARCHITECTURE                  │
│                                                                             │
│  ┌─────────────────┐  ┌─────────────────┐  ┌─────────────────┐            │
│  │  Domain A       │  │  Domain B       │  │  Domain C       │            │
│  │  (Sales)        │  │  (Supply Chain) │  │  (Finance)      │            │
│  │                 │  │                 │  │                 │            │
│  │  ┌───────────┐  │  │  ┌───────────┐  │  │  ┌───────────┐  │            │
│  │  │ Real Data │  │  │  │ Real Data │  │  │  │ Real Data │  │            │
│  │  └─────┬─────┘  │  │  └─────┬─────┘  │  │  └─────┬─────┘  │            │
│  │        │        │  │        │        │  │        │        │            │
│  │        ▼        │  │        ▼        │  │        ▼        │            │
│  │  ┌───────────┐  │  │  ┌───────────┐  │  │  ┌───────────┐  │            │
│  │  │ .dsf      │  │  │  │ .dsf      │  │  │  │ .dsf      │  │            │
│  │  │(Domain A) │  │  │  │(Domain B) │  │  │  │(Domain C) │  │            │
│  │  └─────┬─────┘  │  │  └─────┬─────┘  │  │  └─────┬─────┘  │            │
│  │        │        │  │        │        │  │        │        │            │
│  └────────│────────┘  └────────│────────┘  └────────│────────┘            │
│           │                    │                    │                      │
│           └────────────────────┼────────────────────┘                      │
│                                │                                           │
│                                ▼                                           │
│                    ┌─────────────────────┐                                │
│                    │  FINGERPRINT        │                                │
│                    │  REGISTRY           │                                │
│                    │                     │                                │
│                    │  • Catalog          │                                │
│                    │  • Versioning       │                                │
│                    │  • Access Control   │                                │
│                    └──────────┬──────────┘                                │
│                               │                                            │
│                               ▼                                            │
│                    ┌─────────────────────┐                                │
│                    │  CENTRAL ANALYTICS  │                                │
│                    │                     │                                │
│                    │  Combine fingerprints│                               │
│                    │  Generate unified    │                               │
│                    │  synthetic dataset   │                               │
│                    └─────────────────────┘                                │
│                                                                            │
└────────────────────────────────────────────────────────────────────────────┘
```

### Implementation

**Fingerprint Registry API:**

```python
# fingerprint_registry.py
from fastapi import FastAPI, HTTPException
from pydantic import BaseModel
import boto3

app = FastAPI()
s3 = boto3.client('s3')

class FingerprintMetadata(BaseModel):
    domain: str
    name: str
    version: str
    description: str
    schema_tables: list[str]
    privacy_level: str

@app.post("/fingerprints/{domain}/{name}")
async def register_fingerprint(
    domain: str,
    name: str,
    metadata: FingerprintMetadata
):
    """Register a new fingerprint version."""
    key = f"fingerprints/{domain}/{name}/v{metadata.version}.dsf"
    # Store metadata in registry
    # ...
    return {"status": "registered", "key": key}

@app.get("/fingerprints/{domain}/{name}/latest")
async def get_latest_fingerprint(domain: str, name: str):
    """Get the latest fingerprint for a domain."""
    # Lookup latest version
    # Generate presigned URL for download
    url = s3.generate_presigned_url(
        'get_object',
        Params={'Bucket': 'fingerprint-registry', 'Key': key},
        ExpiresIn=3600
    )
    return {"download_url": url}

@app.post("/fingerprints/combine")
async def combine_fingerprints(fingerprint_keys: list[str]):
    """Combine multiple fingerprints for cross-domain analysis."""
    # Download and merge fingerprints
    # Return combined fingerprint
    pass
```

---

## Best Practices Summary

| Pattern | Key Considerations |
|---------|-------------------|
| **Cross-Border** | Use high privacy settings, document legal basis, sign fingerprints |
| **Vendor Testing** | Provide multiple scale options, include test anomalies |
| **ML Training** | Amplify anomaly rates for training, use consistent seeds |
| **Audit Development** | Build fingerprint library by industry, version procedures |
| **CI/CD** | Cache fingerprints, use deterministic seeds, small scale |
| **Data Mesh** | Central registry, versioning, access control |

---

## Next Steps

- [Fingerprint Specification](../reference/01-fingerprint-spec.md): Complete format reference
- [CLI Reference](../reference/02-cli-reference.md): Command-line documentation
- [Example Use Cases](../examples/): Detailed examples
