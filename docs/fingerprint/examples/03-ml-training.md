# Example: ML Training Pipeline

This example demonstrates how to use fingerprinting for machine learning model development with realistic training data.

---

## Scenario

**FinanceAI Corp** is developing fraud detection models for banking clients. They need:
- Realistic transaction data for model training
- Proper fraud/non-fraud label distribution
- Multiple data variants for model robustness testing
- No access to actual client data

---

## Solution Overview

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                         ML TRAINING PIPELINE                                │
│                                                                             │
│  ┌─────────────────────────────────────────────────────────────────────┐   │
│  │                    FINGERPRINT SOURCES                               │   │
│  │                                                                      │   │
│  │  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌──────────┐            │   │
│  │  │ Bank A   │  │ Bank B   │  │ Bank C   │  │ Synthetic │            │   │
│  │  │ .dsf     │  │ .dsf     │  │ .dsf     │  │ Baseline  │            │   │
│  │  │          │  │          │  │          │  │ .dsf      │            │   │
│  │  │ Retail   │  │ Business │  │ Mixed    │  │ Standard  │            │   │
│  │  │ Focus    │  │ Focus    │  │ Portfolio│  │ Patterns  │            │   │
│  │  └──────────┘  └──────────┘  └──────────┘  └──────────┘            │   │
│  │                                                                      │   │
│  └─────────────────────────────────────────────────────────────────────┘   │
│                              │                                              │
│                              ▼                                              │
│  ┌─────────────────────────────────────────────────────────────────────┐   │
│  │                    DATA GENERATION                                   │   │
│  │                                                                      │   │
│  │  ┌─────────────────────────────────────────────────────────────┐    │   │
│  │  │                 DataSynth Generator                          │    │   │
│  │  │                                                              │    │   │
│  │  │  • Base training data (normal fraud rate)                   │    │   │
│  │  │  • Upsampled fraud data (5x fraud rate)                     │    │   │
│  │  │  • Edge case data (extreme values, rare patterns)           │    │   │
│  │  │  • Adversarial data (model stress testing)                  │    │   │
│  │  │                                                              │    │   │
│  │  └─────────────────────────────────────────────────────────────┘    │   │
│  │                              │                                       │   │
│  │           ┌──────────────────┼──────────────────┐                   │   │
│  │           ▼                  ▼                  ▼                   │   │
│  │  ┌─────────────┐    ┌─────────────┐    ┌─────────────┐             │   │
│  │  │ Training    │    │ Validation  │    │ Test        │             │   │
│  │  │ Set (70%)   │    │ Set (15%)   │    │ Set (15%)   │             │   │
│  │  └─────────────┘    └─────────────┘    └─────────────┘             │   │
│  │                                                                      │   │
│  └─────────────────────────────────────────────────────────────────────┘   │
│                              │                                              │
│                              ▼                                              │
│  ┌─────────────────────────────────────────────────────────────────────┐   │
│  │                    MODEL TRAINING                                    │   │
│  │                                                                      │   │
│  │  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌──────────┐            │   │
│  │  │ Feature  │  │ Model    │  │ Hyper-   │  │ Model    │            │   │
│  │  │ Engineer │──│ Training │──│ parameter│──│ Registry │            │   │
│  │  │          │  │          │  │ Tuning   │  │          │            │   │
│  │  └──────────┘  └──────────┘  └──────────┘  └──────────┘            │   │
│  │                                                                      │   │
│  └─────────────────────────────────────────────────────────────────────┘   │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## Step 1: Fingerprint Library

### Organize Fingerprints by Use Case

```
fingerprint_library/
├── banking/
│   ├── retail_banking_large.dsf       # Large retail bank
│   ├── retail_banking_medium.dsf      # Medium retail bank
│   ├── business_banking.dsf           # Business-focused bank
│   ├── credit_union.dsf               # Credit union patterns
│   └── metadata.yaml                  # Library metadata
├── fraud_patterns/
│   ├── structuring.dsf                # Structuring patterns
│   ├── money_laundering.dsf           # AML patterns
│   ├── identity_fraud.dsf             # Identity fraud
│   └── metadata.yaml
└── library_config.yaml
```

### Library Configuration

```yaml
# fingerprint_library/library_config.yaml

library:
  name: "FinanceAI Training Library"
  version: "2.0"

  sources:
    - id: "retail_large"
      path: "banking/retail_banking_large.dsf"
      description: "Large retail bank, 10M+ accounts"
      characteristics:
        account_types: ["checking", "savings", "credit_card"]
        transaction_volume: "high"
        fraud_rate: 0.002
        customer_base: "consumer"

    - id: "business"
      path: "banking/business_banking.dsf"
      description: "Business banking focus"
      characteristics:
        account_types: ["commercial", "treasury", "merchant"]
        transaction_volume: "medium"
        fraud_rate: 0.003
        customer_base: "business"

    - id: "structuring"
      path: "fraud_patterns/structuring.dsf"
      description: "Known structuring patterns"
      characteristics:
        pattern_type: "structuring"
        threshold_amounts: [9900, 9950, 9999]

  presets:
    standard_training:
      sources: ["retail_large", "business"]
      mix_ratio: [0.7, 0.3]
      fraud_rate: 0.002

    fraud_heavy:
      sources: ["retail_large", "structuring"]
      mix_ratio: [0.5, 0.5]
      fraud_rate: 0.05

    stress_test:
      sources: ["retail_large"]
      fraud_rate: 0.10
      edge_cases: true
```

---

## Step 2: Data Generation Script

### Multi-Dataset Generator

```python
#!/usr/bin/env python3
# ml_data_generator.py

"""
Generate multiple training datasets from fingerprints for ML model development.
"""

import argparse
import yaml
from pathlib import Path
from datasynth_py import DataSynth, Config
from datetime import datetime
import hashlib

class MLDataGenerator:
    def __init__(self, library_path: str):
        self.library_path = Path(library_path)
        self.config = self._load_library_config()
        self.synth = DataSynth()

    def _load_library_config(self) -> dict:
        config_path = self.library_path / "library_config.yaml"
        with open(config_path) as f:
            return yaml.safe_load(f)

    def generate_training_data(
        self,
        preset: str,
        output_dir: str,
        scale: float = 1.0,
        seed: int = None
    ) -> dict:
        """Generate training, validation, and test datasets."""

        preset_config = self.config["library"]["presets"][preset]
        output_path = Path(output_dir)
        output_path.mkdir(parents=True, exist_ok=True)

        # Use deterministic seed if not provided
        if seed is None:
            seed = int(hashlib.md5(preset.encode()).hexdigest()[:8], 16)

        results = {}

        # Generate training data (70%)
        print(f"Generating training data...")
        results["train"] = self._generate_split(
            preset_config=preset_config,
            output_path=output_path / "train",
            scale=scale * 0.7,
            seed=seed,
            fraud_multiplier=preset_config.get("fraud_rate", 0.002) / 0.002 * 2.5
        )

        # Generate validation data (15%)
        print(f"Generating validation data...")
        results["validation"] = self._generate_split(
            preset_config=preset_config,
            output_path=output_path / "validation",
            scale=scale * 0.15,
            seed=seed + 1,
            fraud_multiplier=1.0  # Normal fraud rate
        )

        # Generate test data (15%)
        print(f"Generating test data...")
        results["test"] = self._generate_split(
            preset_config=preset_config,
            output_path=output_path / "test",
            scale=scale * 0.15,
            seed=seed + 2,
            fraud_multiplier=1.0  # Normal fraud rate
        )

        # Save generation metadata
        self._save_metadata(output_path, preset, results, seed)

        return results

    def _generate_split(
        self,
        preset_config: dict,
        output_path: Path,
        scale: float,
        seed: int,
        fraud_multiplier: float
    ) -> dict:
        """Generate a single data split."""

        sources = preset_config["sources"]
        mix_ratio = preset_config.get("mix_ratio", [1.0 / len(sources)] * len(sources))

        # Generate from each source and combine
        all_data = []
        for source_id, ratio in zip(sources, mix_ratio):
            source = self._get_source(source_id)
            fp_path = self.library_path / source["path"]

            result = self.synth.generate(
                fingerprint=str(fp_path),
                output={
                    "format": "memory"
                },
                overrides={
                    "scale": scale * ratio,
                    "seed": seed,
                    "anomaly_injection": {
                        "rate_multiplier": fraud_multiplier
                    }
                }
            )
            all_data.append(result.tables)

        # Combine and shuffle
        combined = self._combine_tables(all_data)

        # Save to disk
        output_path.mkdir(parents=True, exist_ok=True)
        for table_name, df in combined.items():
            df.to_parquet(output_path / f"{table_name}.parquet", index=False)

        return {
            "path": str(output_path),
            "tables": list(combined.keys()),
            "row_counts": {k: len(v) for k, v in combined.items()}
        }

    def _get_source(self, source_id: str) -> dict:
        for source in self.config["library"]["sources"]:
            if source["id"] == source_id:
                return source
        raise ValueError(f"Source not found: {source_id}")

    def _combine_tables(self, all_data: list) -> dict:
        """Combine tables from multiple sources."""
        import pandas as pd

        combined = {}
        for data in all_data:
            for table_name, df in data.items():
                if table_name in combined:
                    combined[table_name] = pd.concat(
                        [combined[table_name], df],
                        ignore_index=True
                    )
                else:
                    combined[table_name] = df

        # Shuffle each table
        for table_name in combined:
            combined[table_name] = combined[table_name].sample(
                frac=1,
                random_state=42
            ).reset_index(drop=True)

        return combined

    def _save_metadata(
        self,
        output_path: Path,
        preset: str,
        results: dict,
        seed: int
    ):
        """Save generation metadata for reproducibility."""
        metadata = {
            "generated_at": datetime.utcnow().isoformat(),
            "preset": preset,
            "seed": seed,
            "library_version": self.config["library"]["version"],
            "splits": results
        }

        with open(output_path / "metadata.yaml", "w") as f:
            yaml.dump(metadata, f, default_flow_style=False)


def main():
    parser = argparse.ArgumentParser(description="Generate ML training data")
    parser.add_argument("--library", required=True, help="Fingerprint library path")
    parser.add_argument("--preset", required=True, help="Generation preset")
    parser.add_argument("--output", required=True, help="Output directory")
    parser.add_argument("--scale", type=float, default=1.0, help="Scale factor")
    parser.add_argument("--seed", type=int, help="Random seed")

    args = parser.parse_args()

    generator = MLDataGenerator(args.library)
    results = generator.generate_training_data(
        preset=args.preset,
        output_dir=args.output,
        scale=args.scale,
        seed=args.seed
    )

    print("\nGeneration complete!")
    for split, info in results.items():
        print(f"\n{split}:")
        for table, count in info["row_counts"].items():
            print(f"  {table}: {count:,} rows")


if __name__ == "__main__":
    main()
```

---

## Step 3: Model Training Pipeline

### MLflow Training Script

```python
#!/usr/bin/env python3
# train_model.py

"""
Train fraud detection model using synthetic data.
"""

import pandas as pd
import numpy as np
from sklearn.ensemble import IsolationForest, RandomForestClassifier
from sklearn.preprocessing import StandardScaler
from sklearn.metrics import (
    precision_recall_fscore_support,
    roc_auc_score,
    confusion_matrix
)
import mlflow
import mlflow.sklearn
from pathlib import Path
import yaml
import argparse


class FraudModelTrainer:
    def __init__(self, data_dir: str):
        self.data_dir = Path(data_dir)
        self.metadata = self._load_metadata()

    def _load_metadata(self) -> dict:
        with open(self.data_dir / "metadata.yaml") as f:
            return yaml.safe_load(f)

    def load_data(self, split: str) -> pd.DataFrame:
        """Load data for a specific split."""
        split_path = self.data_dir / split / "transactions.parquet"
        return pd.read_parquet(split_path)

    def engineer_features(self, df: pd.DataFrame) -> pd.DataFrame:
        """Create features for fraud detection."""

        features = pd.DataFrame()

        # Amount features
        features["amount"] = df["amount"]
        features["amount_log"] = np.log1p(df["amount"])
        features["amount_is_round"] = (df["amount"] % 100 == 0).astype(int)

        # Threshold proximity (structuring detection)
        features["dist_to_10k"] = np.abs(df["amount"] - 10000)
        features["is_near_10k"] = (features["dist_to_10k"] < 500).astype(int)

        # Temporal features
        features["hour"] = pd.to_datetime(df["transaction_time"]).dt.hour
        features["day_of_week"] = pd.to_datetime(df["transaction_time"]).dt.dayofweek
        features["is_weekend"] = (features["day_of_week"] >= 5).astype(int)
        features["is_night"] = ((features["hour"] >= 22) | (features["hour"] <= 5)).astype(int)

        # Account features
        if "account_age_days" in df.columns:
            features["account_age_days"] = df["account_age_days"]
            features["is_new_account"] = (df["account_age_days"] < 30).astype(int)

        # Transaction type encoding
        if "transaction_type" in df.columns:
            type_dummies = pd.get_dummies(df["transaction_type"], prefix="type")
            features = pd.concat([features, type_dummies], axis=1)

        return features

    def train_supervised(self, train_features, train_labels, val_features, val_labels):
        """Train supervised model (when labels available)."""

        with mlflow.start_run(run_name="supervised_rf"):
            # Log data info
            mlflow.log_param("data_source", "synthetic_fingerprint")
            mlflow.log_param("preset", self.metadata["preset"])
            mlflow.log_param("seed", self.metadata["seed"])
            mlflow.log_param("train_size", len(train_features))
            mlflow.log_param("train_fraud_rate", train_labels.mean())

            # Scale features
            scaler = StandardScaler()
            train_scaled = scaler.fit_transform(train_features)
            val_scaled = scaler.transform(val_features)

            # Train model
            model = RandomForestClassifier(
                n_estimators=100,
                max_depth=10,
                class_weight="balanced",
                random_state=42,
                n_jobs=-1
            )
            model.fit(train_scaled, train_labels)

            # Evaluate
            val_pred = model.predict(val_scaled)
            val_prob = model.predict_proba(val_scaled)[:, 1]

            precision, recall, f1, _ = precision_recall_fscore_support(
                val_labels, val_pred, average="binary"
            )
            auc = roc_auc_score(val_labels, val_prob)

            # Log metrics
            mlflow.log_metric("val_precision", precision)
            mlflow.log_metric("val_recall", recall)
            mlflow.log_metric("val_f1", f1)
            mlflow.log_metric("val_auc", auc)

            # Log model
            mlflow.sklearn.log_model(model, "model")
            mlflow.sklearn.log_model(scaler, "scaler")

            # Log feature importance
            importance = pd.DataFrame({
                "feature": train_features.columns,
                "importance": model.feature_importances_
            }).sort_values("importance", ascending=False)

            importance.to_csv("feature_importance.csv", index=False)
            mlflow.log_artifact("feature_importance.csv")

            print(f"Validation Results:")
            print(f"  Precision: {precision:.3f}")
            print(f"  Recall: {recall:.3f}")
            print(f"  F1: {f1:.3f}")
            print(f"  AUC: {auc:.3f}")

            return model, scaler

    def train_unsupervised(self, train_features, val_features, val_labels):
        """Train unsupervised anomaly detection."""

        with mlflow.start_run(run_name="unsupervised_if"):
            # Log params
            mlflow.log_param("data_source", "synthetic_fingerprint")
            mlflow.log_param("model_type", "isolation_forest")
            mlflow.log_param("train_size", len(train_features))

            # Scale
            scaler = StandardScaler()
            train_scaled = scaler.fit_transform(train_features)
            val_scaled = scaler.transform(val_features)

            # Train
            model = IsolationForest(
                n_estimators=100,
                contamination=0.02,
                random_state=42,
                n_jobs=-1
            )
            model.fit(train_scaled)

            # Evaluate
            val_pred = model.predict(val_scaled)
            val_pred_binary = (val_pred == -1).astype(int)

            precision, recall, f1, _ = precision_recall_fscore_support(
                val_labels, val_pred_binary, average="binary"
            )

            # Log metrics
            mlflow.log_metric("val_precision", precision)
            mlflow.log_metric("val_recall", recall)
            mlflow.log_metric("val_f1", f1)

            # Log model
            mlflow.sklearn.log_model(model, "model")

            print(f"Validation Results:")
            print(f"  Precision: {precision:.3f}")
            print(f"  Recall: {recall:.3f}")
            print(f"  F1: {f1:.3f}")

            return model, scaler

    def evaluate_on_test(self, model, scaler, test_features, test_labels):
        """Final evaluation on test set."""

        test_scaled = scaler.transform(test_features)

        if hasattr(model, "predict_proba"):
            test_pred = model.predict(test_scaled)
            test_prob = model.predict_proba(test_scaled)[:, 1]
        else:
            test_pred = model.predict(test_scaled)
            test_pred = (test_pred == -1).astype(int)
            test_prob = -model.score_samples(test_scaled)  # Anomaly scores

        precision, recall, f1, _ = precision_recall_fscore_support(
            test_labels, test_pred, average="binary"
        )

        print(f"\nTest Results:")
        print(f"  Precision: {precision:.3f}")
        print(f"  Recall: {recall:.3f}")
        print(f"  F1: {f1:.3f}")

        cm = confusion_matrix(test_labels, test_pred)
        print(f"\nConfusion Matrix:")
        print(cm)


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--data-dir", required=True)
    parser.add_argument("--model-type", choices=["supervised", "unsupervised"], default="supervised")
    args = parser.parse_args()

    mlflow.set_experiment("fraud_detection_synthetic")

    trainer = FraudModelTrainer(args.data_dir)

    # Load data
    print("Loading data...")
    train_df = trainer.load_data("train")
    val_df = trainer.load_data("validation")
    test_df = trainer.load_data("test")

    # Engineer features
    print("Engineering features...")
    train_features = trainer.engineer_features(train_df)
    val_features = trainer.engineer_features(val_df)
    test_features = trainer.engineer_features(test_df)

    train_labels = train_df["is_fraud"]
    val_labels = val_df["is_fraud"]
    test_labels = test_df["is_fraud"]

    # Train
    print(f"\nTraining {args.model_type} model...")
    if args.model_type == "supervised":
        model, scaler = trainer.train_supervised(
            train_features, train_labels,
            val_features, val_labels
        )
    else:
        model, scaler = trainer.train_unsupervised(
            train_features,
            val_features, val_labels
        )

    # Test
    print("\nEvaluating on test set...")
    trainer.evaluate_on_test(model, scaler, test_features, test_labels)


if __name__ == "__main__":
    main()
```

---

## Step 4: CI/CD Integration

### GitHub Actions Workflow

```yaml
# .github/workflows/ml-pipeline.yml

name: ML Training Pipeline

on:
  push:
    branches: [main]
    paths:
      - 'ml/**'
      - 'fingerprint_library/**'
  schedule:
    - cron: '0 6 * * 1'  # Weekly Monday 6 AM

jobs:
  generate-data:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Set up Python
        uses: actions/setup-python@v4
        with:
          python-version: '3.11'

      - name: Install dependencies
        run: |
          pip install datasynth-py[all]
          pip install -r ml/requirements.txt

      - name: Generate training data
        run: |
          python ml/ml_data_generator.py \
            --library ./fingerprint_library \
            --preset standard_training \
            --output ./ml_data \
            --scale 0.1 \
            --seed ${{ github.run_number }}

      - name: Upload data artifact
        uses: actions/upload-artifact@v3
        with:
          name: ml-data
          path: ml_data/

  train-model:
    needs: generate-data
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Download data
        uses: actions/download-artifact@v3
        with:
          name: ml-data
          path: ml_data/

      - name: Train supervised model
        run: |
          python ml/train_model.py \
            --data-dir ./ml_data \
            --model-type supervised

      - name: Train unsupervised model
        run: |
          python ml/train_model.py \
            --data-dir ./ml_data \
            --model-type unsupervised

      - name: Upload model artifacts
        uses: actions/upload-artifact@v3
        with:
          name: models
          path: mlruns/
```

---

## Summary

This example demonstrates:

1. **Fingerprint Library**: Organized collection of fingerprints for different use cases
2. **Preset-based Generation**: Standardized data generation presets for consistency
3. **Multi-Split Generation**: Training, validation, and test splits with appropriate fraud rates
4. **Feature Engineering**: Standard feature engineering pipeline
5. **Model Training**: Both supervised and unsupervised approaches
6. **CI/CD Integration**: Automated pipeline for model training

Key benefits:
- **Reproducibility**: Deterministic data generation with seeds
- **Scalability**: Generate any amount of training data
- **Privacy**: No real customer data in the ML pipeline
- **Flexibility**: Multiple presets for different training scenarios
