# Architecture: Fingerprint System Design

## System Overview

The Fingerprint System consists of three main components that work together to enable privacy-preserving synthetic data generation:

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                         FINGERPRINT ECOSYSTEM                               │
│                                                                             │
│  ┌─────────────────────┐  ┌─────────────────────┐  ┌─────────────────────┐ │
│  │                     │  │                     │  │                     │ │
│  │  EXTRACTION ENGINE  │  │   .dsf FILE FORMAT  │  │  GENERATION ENGINE  │ │
│  │                     │  │                     │  │                     │ │
│  │  datasynth-         │  │  Portable           │  │  datasynth-         │ │
│  │  fingerprint        │─▶│  Fingerprint        │─▶│  runtime            │ │
│  │                     │  │  Container          │  │  (fingerprint mode) │ │
│  │  • Schema Analysis  │  │                     │  │                     │ │
│  │  • Stats Extraction │  │  • Schema           │  │  • Config Synthesis │ │
│  │  • Privacy Filters  │  │  • Statistics       │  │  • Distribution Fit │ │
│  │  • Validation       │  │  • Correlations     │  │  • Rule Enforcement │ │
│  │                     │  │  • Rules            │  │                     │ │
│  └─────────────────────┘  │  • Privacy Audit    │  └─────────────────────┘ │
│                           │                     │                           │
│                           └─────────────────────┘                           │
│                                                                             │
│  ┌─────────────────────────────────────────────────────────────────────┐   │
│  │                        FIDELITY EVALUATOR                            │   │
│  │  datasynth-eval (fingerprint comparison mode)                        │   │
│  └─────────────────────────────────────────────────────────────────────┘   │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## Component Architecture

### 1. Extraction Engine (`datasynth-fingerprint`)

The extraction engine analyzes real data and produces privacy-safe fingerprints.

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                          EXTRACTION ENGINE                                  │
│                                                                             │
│  ┌──────────┐                                                               │
│  │  Input   │  CSV, Parquet, Database Connection, JSON                      │
│  │  Sources │                                                               │
│  └────┬─────┘                                                               │
│       │                                                                     │
│       ▼                                                                     │
│  ┌─────────────────────────────────────────────────────────────────────┐   │
│  │                        DATA READERS                                  │   │
│  │  ┌─────────┐  ┌─────────┐  ┌─────────┐  ┌─────────┐                 │   │
│  │  │   CSV   │  │ Parquet │  │ Database│  │  JSON   │                 │   │
│  │  │ Reader  │  │ Reader  │  │ Reader  │  │ Reader  │                 │   │
│  │  └────┬────┘  └────┬────┘  └────┬────┘  └────┬────┘                 │   │
│  │       └───────┬────┴───────────┴────────────┘                        │   │
│  │               ▼                                                      │   │
│  │  ┌─────────────────────────────────────────────────────────────┐    │   │
│  │  │                    UNIFIED DATA INTERFACE                    │    │   │
│  │  │  • Streaming iterator over records                          │    │   │
│  │  │  • Schema introspection                                     │    │   │
│  │  │  • Sampling support                                         │    │   │
│  │  └─────────────────────────────────────────────────────────────┘    │   │
│  └─────────────────────────────────────────────────────────────────────┘   │
│       │                                                                     │
│       ▼                                                                     │
│  ┌─────────────────────────────────────────────────────────────────────┐   │
│  │                         EXTRACTORS                                   │   │
│  │                                                                      │   │
│  │  ┌─────────────────┐  ┌─────────────────┐  ┌─────────────────┐     │   │
│  │  │ Schema Extractor│  │ Stats Extractor │  │ Correlation     │     │   │
│  │  │                 │  │                 │  │ Extractor       │     │   │
│  │  │ • Tables        │  │ • Distributions │  │                 │     │   │
│  │  │ • Columns       │  │ • Percentiles   │  │ • Pairwise      │     │   │
│  │  │ • Types         │  │ • Null rates    │  │ • Conditionals  │     │   │
│  │  │ • Relationships │  │ • Cardinality   │  │ • Copulas       │     │   │
│  │  │ • Constraints   │  │ • Patterns      │  │                 │     │   │
│  │  └────────┬────────┘  └────────┬────────┘  └────────┬────────┘     │   │
│  │           │                    │                    │               │   │
│  │  ┌────────┴────────┐  ┌────────┴────────┐  ┌────────┴────────┐     │   │
│  │  │ Integrity       │  │ Rules Extractor │  │ Anomaly         │     │   │
│  │  │ Extractor       │  │                 │  │ Extractor       │     │   │
│  │  │                 │  │ • Balance eqs   │  │                 │     │   │
│  │  │ • FK coverage   │  │ • Thresholds    │  │ • Anomaly rates │     │   │
│  │  │ • Cardinalities │  │ • Constraints   │  │ • Type dist.    │     │   │
│  │  │ • Orphan rates  │  │ • SoD patterns  │  │ • Patterns      │     │   │
│  │  └────────┬────────┘  └────────┬────────┘  └────────┬────────┘     │   │
│  │           └───────────────┬────┴────────────────────┘               │   │
│  └───────────────────────────┼─────────────────────────────────────────┘   │
│                              ▼                                              │
│  ┌─────────────────────────────────────────────────────────────────────┐   │
│  │                       PRIVACY ENGINE                                 │   │
│  │                                                                      │   │
│  │  ┌─────────────────┐  ┌─────────────────┐  ┌─────────────────┐     │   │
│  │  │ Differential    │  │ Suppression     │  │ Outlier         │     │   │
│  │  │ Privacy         │  │ Engine          │  │ Handler         │     │   │
│  │  │                 │  │                 │  │                 │     │   │
│  │  │ • Laplace noise │  │ • k-anonymity   │  │ • Winsorization │     │   │
│  │  │ • Exponential   │  │ • Rare category │  │ • Capping       │     │   │
│  │  │   mechanism     │  │   merging       │  │ • Exclusion     │     │   │
│  │  │ • Budget track  │  │ • Cell suppress │  │                 │     │   │
│  │  └─────────────────┘  └─────────────────┘  └─────────────────┘     │   │
│  │                              │                                       │   │
│  │                              ▼                                       │   │
│  │  ┌─────────────────────────────────────────────────────────────┐    │   │
│  │  │                    PRIVACY AUDITOR                           │    │   │
│  │  │  • Verify no PII leakage                                    │    │   │
│  │  │  • Check minimum group sizes                                │    │   │
│  │  │  • Validate epsilon budget                                  │    │   │
│  │  │  • Generate audit trail                                     │    │   │
│  │  └─────────────────────────────────────────────────────────────┘    │   │
│  └─────────────────────────────────────────────────────────────────────┘   │
│                              │                                              │
│                              ▼                                              │
│  ┌─────────────────────────────────────────────────────────────────────┐   │
│  │                       .dsf FILE WRITER                               │   │
│  │  • Serialize fingerprint components                                  │   │
│  │  • Compute checksums                                                 │   │
│  │  • Optional encryption/signing                                       │   │
│  └─────────────────────────────────────────────────────────────────────┘   │
│                              │                                              │
│                              ▼                                              │
│                    ┌─────────────────┐                                      │
│                    │  .dsf Output    │                                      │
│                    └─────────────────┘                                      │
└─────────────────────────────────────────────────────────────────────────────┘
```

#### Extractor Components

| Component | Purpose | Outputs |
|-----------|---------|---------|
| **Schema Extractor** | Analyze data structure | Tables, columns, types, relationships |
| **Stats Extractor** | Compute distributions | Distribution parameters, percentiles, patterns |
| **Correlation Extractor** | Find dependencies | Correlation matrices, conditionals, copulas |
| **Integrity Extractor** | Analyze relationships | FK coverage, cardinalities, orphan rates |
| **Rules Extractor** | Infer business rules | Balance equations, thresholds, constraints |
| **Anomaly Extractor** | Profile anomalies | Anomaly rates, type distribution, patterns |

---

### 2. Fingerprint File Format (.dsf)

The `.dsf` (DataSynth Fingerprint) format is a portable container for fingerprint data.

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                         .dsf FILE STRUCTURE                                 │
│                                                                             │
│  ┌─────────────────────────────────────────────────────────────────────┐   │
│  │  MANIFEST (manifest.json)                                            │   │
│  │  ┌─────────────────────────────────────────────────────────────┐    │   │
│  │  │  {                                                           │    │   │
│  │  │    "version": "1.0.0",                                       │    │   │
│  │  │    "format": "datasynth_fingerprint",                        │    │   │
│  │  │    "created_at": "2024-12-15T10:30:00Z",                     │    │   │
│  │  │    "source_hash": "sha256:abc...",  // Hash of source paths  │    │   │
│  │  │    "privacy_level": "dp_epsilon_1.0",                        │    │   │
│  │  │    "checksums": { ... }                                      │    │   │
│  │  │  }                                                           │    │   │
│  │  └─────────────────────────────────────────────────────────────┘    │   │
│  └─────────────────────────────────────────────────────────────────────┘   │
│                                                                             │
│  ┌─────────────────────────────────────────────────────────────────────┐   │
│  │  SCHEMA FINGERPRINT (schema.yaml)                                    │   │
│  │  ┌─────────────────────────────────────────────────────────────┐    │   │
│  │  │  tables:                                                     │    │   │
│  │  │    - name: journal_entries                                   │    │   │
│  │  │      row_count: 1247832  # Noised                           │    │   │
│  │  │      columns: [...]                                          │    │   │
│  │  │  relationships: [...]                                        │    │   │
│  │  └─────────────────────────────────────────────────────────────┘    │   │
│  └─────────────────────────────────────────────────────────────────────┘   │
│                                                                             │
│  ┌─────────────────────────────────────────────────────────────────────┐   │
│  │  STATISTICS FINGERPRINT (statistics.yaml)                            │   │
│  │  ┌─────────────────────────────────────────────────────────────┐    │   │
│  │  │  numeric_columns: [...]                                      │    │   │
│  │  │  categorical_columns: [...]                                  │    │   │
│  │  │  temporal_columns: [...]                                     │    │   │
│  │  └─────────────────────────────────────────────────────────────┘    │   │
│  └─────────────────────────────────────────────────────────────────────┘   │
│                                                                             │
│  ┌─────────────────────────────────────────────────────────────────────┐   │
│  │  CORRELATION FINGERPRINT (correlations.yaml)                         │   │
│  │  ┌─────────────────────────────────────────────────────────────┐    │   │
│  │  │  matrix: { ... }                                             │    │   │
│  │  │  conditionals: [...]                                         │    │   │
│  │  │  copulas: [...]                                              │    │   │
│  │  └─────────────────────────────────────────────────────────────┘    │   │
│  └─────────────────────────────────────────────────────────────────────┘   │
│                                                                             │
│  ┌─────────────────────────────────────────────────────────────────────┐   │
│  │  INTEGRITY FINGERPRINT (integrity.yaml)                              │   │
│  │  ┌─────────────────────────────────────────────────────────────┐    │   │
│  │  │  foreign_keys: [...]                                         │    │   │
│  │  │  cardinality_patterns: [...]                                 │    │   │
│  │  │  temporal_ordering: [...]                                    │    │   │
│  │  └─────────────────────────────────────────────────────────────┘    │   │
│  └─────────────────────────────────────────────────────────────────────┘   │
│                                                                             │
│  ┌─────────────────────────────────────────────────────────────────────┐   │
│  │  RULES FINGERPRINT (rules.yaml)                                      │   │
│  │  ┌─────────────────────────────────────────────────────────────┐    │   │
│  │  │  balance_constraints: [...]                                  │    │   │
│  │  │  value_constraints: [...]                                    │    │   │
│  │  │  approval_patterns: { ... }                                  │    │   │
│  │  └─────────────────────────────────────────────────────────────┘    │   │
│  └─────────────────────────────────────────────────────────────────────┘   │
│                                                                             │
│  ┌─────────────────────────────────────────────────────────────────────┐   │
│  │  ANOMALY FINGERPRINT (anomalies.yaml)                                │   │
│  │  ┌─────────────────────────────────────────────────────────────┐    │   │
│  │  │  overall_rate: 0.023                                         │    │   │
│  │  │  by_type: { fraud: {...}, errors: {...} }                    │    │   │
│  │  │  temporal_patterns: { ... }                                  │    │   │
│  │  └─────────────────────────────────────────────────────────────┘    │   │
│  └─────────────────────────────────────────────────────────────────────┘   │
│                                                                             │
│  ┌─────────────────────────────────────────────────────────────────────┐   │
│  │  PRIVACY AUDIT (privacy_audit.json)                                  │   │
│  │  ┌─────────────────────────────────────────────────────────────┐    │   │
│  │  │  {                                                           │    │   │
│  │  │    "checks_performed": [...],                                │    │   │
│  │  │    "redacted_fields": [...],                                 │    │   │
│  │  │    "warnings": [...]                                         │    │   │
│  │  │  }                                                           │    │   │
│  │  └─────────────────────────────────────────────────────────────┘    │   │
│  └─────────────────────────────────────────────────────────────────────┘   │
│                                                                             │
│  ┌─────────────────────────────────────────────────────────────────────┐   │
│  │  SIGNATURE (signature.sig) - Optional                                │   │
│  │  ┌─────────────────────────────────────────────────────────────┐    │   │
│  │  │  Ed25519 signature over all other files                      │    │   │
│  │  └─────────────────────────────────────────────────────────────┘    │   │
│  └─────────────────────────────────────────────────────────────────────┘   │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

### 3. Generation Engine (Fingerprint Mode)

The generation engine transforms fingerprints into DataSynth configurations and generates data.

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                       GENERATION ENGINE (Fingerprint Mode)                  │
│                                                                             │
│  ┌─────────────────┐                                                        │
│  │  .dsf Input     │                                                        │
│  └────────┬────────┘                                                        │
│           │                                                                 │
│           ▼                                                                 │
│  ┌─────────────────────────────────────────────────────────────────────┐   │
│  │                     FINGERPRINT READER                               │   │
│  │  • Parse manifest                                                    │   │
│  │  • Validate checksums                                                │   │
│  │  • Verify signature (if present)                                     │   │
│  │  • Load fingerprint components                                       │   │
│  └─────────────────────────────────────────────────────────────────────┘   │
│           │                                                                 │
│           ▼                                                                 │
│  ┌─────────────────────────────────────────────────────────────────────┐   │
│  │                    CONFIG SYNTHESIZER                                │   │
│  │                                                                      │   │
│  │  Fingerprint ────────────────────────▶ DataSynth Config              │   │
│  │                                                                      │   │
│  │  ┌───────────────────┐    ┌───────────────────┐                     │   │
│  │  │ Schema Mapping    │    │ Global Settings   │                     │   │
│  │  │                   │    │                   │                     │   │
│  │  │ FP tables ──────▶ │    │ • row_count ────▶ │ transactions.count  │   │
│  │  │ FP columns ─────▶ │    │ • date_range ──▶ │ global.start/end    │   │
│  │  │ FP relations ───▶ │    │ • patterns ────▶ │ temporal settings   │   │
│  │  └───────────────────┘    └───────────────────┘                     │   │
│  │                                                                      │   │
│  │  ┌───────────────────┐    ┌───────────────────┐                     │   │
│  │  │ Distribution Map  │    │ Anomaly Settings  │                     │   │
│  │  │                   │    │                   │                     │   │
│  │  │ FP dist params ─▶ │    │ FP anomaly ─────▶ │ anomaly_injection   │   │
│  │  │ FP percentiles ─▶ │    │   rates           │   settings          │   │
│  │  │ FP patterns ────▶ │    │ FP patterns ───▶ │                     │   │
│  │  └───────────────────┘    └───────────────────┘                     │   │
│  └─────────────────────────────────────────────────────────────────────┘   │
│           │                                                                 │
│           ▼                                                                 │
│  ┌─────────────────────────────────────────────────────────────────────┐   │
│  │                   DISTRIBUTION FITTER                                │   │
│  │                                                                      │   │
│  │  For each numeric column in fingerprint:                             │   │
│  │                                                                      │   │
│  │  ┌─────────────────────────────────────────────────────────────┐    │   │
│  │  │  FP Distribution              Generator Distribution         │    │   │
│  │  │  ────────────────              ──────────────────────        │    │   │
│  │  │  LogNormal(μ, σ)      ──▶      AmountSampler with params     │    │   │
│  │  │  Mixture(...)         ──▶      Custom mixture sampler        │    │   │
│  │  │  Empirical(hist)      ──▶      Histogram-based sampler       │    │   │
│  │  │  Pareto(α)            ──▶      Heavy-tail sampler            │    │   │
│  │  └─────────────────────────────────────────────────────────────┘    │   │
│  │                                                                      │   │
│  │  For each categorical column:                                        │   │
│  │  ┌─────────────────────────────────────────────────────────────┐    │   │
│  │  │  FP Frequencies               Generator Weights              │    │   │
│  │  │  ──────────────               ─────────────────              │    │   │
│  │  │  cat_1: 0.34          ──▶     WeightedChoice with freqs      │    │   │
│  │  │  cat_2: 0.23                                                 │    │   │
│  │  └─────────────────────────────────────────────────────────────┘    │   │
│  └─────────────────────────────────────────────────────────────────────┘   │
│           │                                                                 │
│           ▼                                                                 │
│  ┌─────────────────────────────────────────────────────────────────────┐   │
│  │                  CORRELATION ENFORCER                                │   │
│  │                                                                      │   │
│  │  ┌─────────────────────────────────────────────────────────────┐    │   │
│  │  │  Strategy: Gaussian Copula                                   │    │   │
│  │  │                                                              │    │   │
│  │  │  1. Generate correlated uniform variates from copula         │    │   │
│  │  │  2. Transform to target marginal distributions               │    │   │
│  │  │  3. Apply conditional adjustments                            │    │   │
│  │  └─────────────────────────────────────────────────────────────┘    │   │
│  │                                                                      │   │
│  │  ┌─────────────────────────────────────────────────────────────┐    │   │
│  │  │  Conditional Dependencies                                    │    │   │
│  │  │                                                              │    │   │
│  │  │  if business_process == "AP":                                │    │   │
│  │  │      amount *= 1.2  # From fingerprint conditional           │    │   │
│  │  └─────────────────────────────────────────────────────────────┘    │   │
│  └─────────────────────────────────────────────────────────────────────┘   │
│           │                                                                 │
│           ▼                                                                 │
│  ┌─────────────────────────────────────────────────────────────────────┐   │
│  │                    RULE ENFORCER                                     │   │
│  │                                                                      │   │
│  │  ┌─────────────────────────────────────────────────────────────┐    │   │
│  │  │  Balance Constraints                                         │    │   │
│  │  │  • Ensure debits = credits (existing DataSynth behavior)     │    │   │
│  │  │  • Verify trial balance balances                             │    │   │
│  │  └─────────────────────────────────────────────────────────────┘    │   │
│  │                                                                      │   │
│  │  ┌─────────────────────────────────────────────────────────────┐    │   │
│  │  │  Approval Thresholds                                         │    │   │
│  │  │  • Map FP thresholds to approval config                      │    │   │
│  │  │  • Preserve threshold distribution                           │    │   │
│  │  └─────────────────────────────────────────────────────────────┘    │   │
│  │                                                                      │   │
│  │  ┌─────────────────────────────────────────────────────────────┐    │   │
│  │  │  Referential Integrity                                       │    │   │
│  │  │  • Generate entities with correct cardinalities              │    │   │
│  │  │  • Maintain FK relationships                                 │    │   │
│  │  └─────────────────────────────────────────────────────────────┘    │   │
│  └─────────────────────────────────────────────────────────────────────┘   │
│           │                                                                 │
│           ▼                                                                 │
│  ┌─────────────────────────────────────────────────────────────────────┐   │
│  │                EXISTING DATASYNTH PIPELINE                           │   │
│  │                                                                      │   │
│  │  GenerationOrchestrator with fingerprint-derived config              │   │
│  │                                                                      │   │
│  │  ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌──────────┐               │   │
│  │  │ Master   │ │ Document │ │ JE       │ │ Anomaly  │               │   │
│  │  │ Data Gen │ │ Flow Gen │ │ Generator│ │ Injector │               │   │
│  │  └──────────┘ └──────────┘ └──────────┘ └──────────┘               │   │
│  │                                                                      │   │
│  └─────────────────────────────────────────────────────────────────────┘   │
│           │                                                                 │
│           ▼                                                                 │
│  ┌─────────────────┐                                                        │
│  │ Synthetic Data  │                                                        │
│  └─────────────────┘                                                        │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

### 4. Fidelity Evaluator

The fidelity evaluator compares synthetic data against the original fingerprint.

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                         FIDELITY EVALUATOR                                  │
│                                                                             │
│  ┌─────────────────┐        ┌─────────────────┐                            │
│  │  .dsf           │        │  Synthetic      │                            │
│  │  Fingerprint    │        │  Data           │                            │
│  └────────┬────────┘        └────────┬────────┘                            │
│           │                          │                                      │
│           └──────────┬───────────────┘                                      │
│                      │                                                      │
│                      ▼                                                      │
│  ┌─────────────────────────────────────────────────────────────────────┐   │
│  │                    COMPARISON ENGINE                                 │   │
│  │                                                                      │   │
│  │  ┌─────────────────────────────────────────────────────────────┐    │   │
│  │  │  STATISTICAL COMPARISONS                                     │    │   │
│  │  │                                                              │    │   │
│  │  │  For each numeric column:                                    │    │   │
│  │  │  • KS test: synthetic dist vs FP dist                        │    │   │
│  │  │  • Wasserstein distance                                      │    │   │
│  │  │  • Percentile comparison                                     │    │   │
│  │  │  • Benford's Law MAD                                         │    │   │
│  │  │                                                              │    │   │
│  │  │  For each categorical column:                                │    │   │
│  │  │  • Chi-squared test                                          │    │   │
│  │  │  • Jensen-Shannon divergence                                 │    │   │
│  │  └─────────────────────────────────────────────────────────────┘    │   │
│  │                                                                      │   │
│  │  ┌─────────────────────────────────────────────────────────────┐    │   │
│  │  │  STRUCTURAL COMPARISONS                                      │    │   │
│  │  │                                                              │    │   │
│  │  │  • Schema match (100% required)                              │    │   │
│  │  │  • Row count ratio                                           │    │   │
│  │  │  • Null rate differences                                     │    │   │
│  │  │  • Cardinality ratios                                        │    │   │
│  │  │  • FK coverage rates                                         │    │   │
│  │  └─────────────────────────────────────────────────────────────┘    │   │
│  │                                                                      │   │
│  │  ┌─────────────────────────────────────────────────────────────┐    │   │
│  │  │  CORRELATION COMPARISONS                                     │    │   │
│  │  │                                                              │    │   │
│  │  │  • Correlation matrix RMSE                                   │    │   │
│  │  │  • Individual correlation differences                        │    │   │
│  │  │  • Conditional distribution checks                           │    │   │
│  │  └─────────────────────────────────────────────────────────────┘    │   │
│  │                                                                      │   │
│  │  ┌─────────────────────────────────────────────────────────────┐    │   │
│  │  │  RULE COMPLIANCE                                             │    │   │
│  │  │                                                              │    │   │
│  │  │  • Balance equation satisfaction                             │    │   │
│  │  │  • Approval threshold distribution                           │    │   │
│  │  │  • Temporal ordering compliance                              │    │   │
│  │  └─────────────────────────────────────────────────────────────┘    │   │
│  │                                                                      │   │
│  │  ┌─────────────────────────────────────────────────────────────┐    │   │
│  │  │  ANOMALY PROFILE                                             │    │   │
│  │  │                                                              │    │   │
│  │  │  • Overall anomaly rate comparison                           │    │   │
│  │  │  • Anomaly type distribution                                 │    │   │
│  │  │  • Temporal pattern matching                                 │    │   │
│  │  └─────────────────────────────────────────────────────────────┘    │   │
│  └─────────────────────────────────────────────────────────────────────┘   │
│                      │                                                      │
│                      ▼                                                      │
│  ┌─────────────────────────────────────────────────────────────────────┐   │
│  │                    FIDELITY REPORT                                   │   │
│  │                                                                      │   │
│  │  ┌─────────────────────────────────────────────────────────────┐    │   │
│  │  │  Overall Fidelity Score: 94.7%                               │    │   │
│  │  │                                                              │    │   │
│  │  │  Statistical:   96.2%  ████████████████████░░░               │    │   │
│  │  │  Structural:    100.0% ████████████████████████              │    │   │
│  │  │  Correlations:  91.5%  ██████████████████░░░░░░              │    │   │
│  │  │  Rules:         98.3%  ███████████████████████░              │    │   │
│  │  │  Anomalies:     87.4%  █████████████████░░░░░░░              │    │   │
│  │  │                                                              │    │   │
│  │  │  Recommendations:                                            │    │   │
│  │  │  • Increase correlation enforcement for amount/line_count    │    │   │
│  │  │  • Adjust anomaly injection rate (+0.3%)                     │    │   │
│  │  └─────────────────────────────────────────────────────────────┘    │   │
│  └─────────────────────────────────────────────────────────────────────┘   │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## Crate Structure

```
crates/
├── datasynth-fingerprint/          # NEW: Fingerprint extraction
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs
│       ├── extract/                # Extraction logic
│       │   ├── mod.rs
│       │   ├── schema_extractor.rs
│       │   ├── stats_extractor.rs
│       │   ├── correlation_extractor.rs
│       │   ├── integrity_extractor.rs
│       │   ├── rules_extractor.rs
│       │   └── anomaly_extractor.rs
│       ├── privacy/                # Privacy mechanisms
│       │   ├── mod.rs
│       │   ├── differential_privacy.rs
│       │   ├── suppression.rs
│       │   ├── noise.rs
│       │   └── audit.rs
│       ├── models/                 # Fingerprint models
│       │   ├── mod.rs
│       │   ├── fingerprint.rs
│       │   ├── schema.rs
│       │   ├── statistics.rs
│       │   ├── correlations.rs
│       │   ├── integrity.rs
│       │   ├── rules.rs
│       │   └── anomalies.rs
│       ├── io/                     # File I/O
│       │   ├── mod.rs
│       │   ├── reader.rs
│       │   ├── writer.rs
│       │   └── validation.rs
│       └── generation/             # Fingerprint → Config
│           ├── mod.rs
│           ├── config_synthesizer.rs
│           ├── distribution_fitter.rs
│           └── constraint_enforcer.rs
│
├── datasynth-core/                 # Existing: Add fingerprint types
│   └── src/
│       └── models/
│           └── fingerprint.rs      # Shared fingerprint types
│
├── datasynth-runtime/              # Existing: Add fingerprint mode
│   └── src/
│       └── orchestrator.rs         # Add from_fingerprint()
│
├── datasynth-eval/                 # Existing: Add fingerprint comparison
│   └── src/
│       └── fingerprint/            # NEW: Fingerprint comparison
│           ├── mod.rs
│           └── fidelity.rs
│
└── datasynth-cli/                  # Existing: Add fingerprint commands
    └── src/
        └── commands/
            └── fingerprint.rs      # NEW: fingerprint subcommand
```

---

## Data Flow Summary

```
┌──────────────────────────────────────────────────────────────────────────────┐
│                              DATA FLOW                                        │
│                                                                              │
│  ORGANIZATION A                           ORGANIZATION B                      │
│  (Data Owner)                             (Data Consumer)                     │
│                                                                              │
│  ┌────────────┐                                                              │
│  │ Real Data  │                                                              │
│  │ (Private)  │                                                              │
│  └─────┬──────┘                                                              │
│        │                                                                     │
│        ▼                                                                     │
│  ┌────────────────┐                                                          │
│  │ datasynth-     │                                                          │
│  │ fingerprint    │                                                          │
│  │ extract        │                                                          │
│  └─────┬──────────┘                                                          │
│        │                                                                     │
│        ▼                                                                     │
│  ┌────────────────┐          TRANSFER          ┌────────────────┐           │
│  │ .dsf File      │─────────────────────────▶ │ .dsf File      │           │
│  │ (Privacy-Safe) │         (Safe!)            │ (Received)     │           │
│  └────────────────┘                            └─────┬──────────┘           │
│                                                      │                       │
│                                                      ▼                       │
│                                                ┌────────────────┐            │
│                                                │ datasynth-data │            │
│                                                │ generate       │            │
│                                                │ --fingerprint  │            │
│                                                └─────┬──────────┘            │
│                                                      │                       │
│                                                      ▼                       │
│                                                ┌────────────────┐            │
│                                                │ Synthetic Data │            │
│                                                │ (No PII)       │            │
│                                                └─────┬──────────┘            │
│                                                      │                       │
│                                                      ▼                       │
│                                                ┌────────────────┐            │
│                                                │ • ML Training  │            │
│                                                │ • Testing      │            │
│                                                │ • Analytics    │            │
│                                                │ • Development  │            │
│                                                └────────────────┘            │
│                                                                              │
└──────────────────────────────────────────────────────────────────────────────┘
```

---

## Next Steps

- [Privacy Model](./03-privacy-model.md): Deep dive into privacy guarantees
- [Fidelity Model](./04-fidelity-model.md): Understanding quality metrics
- [Fingerprint Specification](../reference/01-fingerprint-spec.md): Complete format reference
