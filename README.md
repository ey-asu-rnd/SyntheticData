# SyntheticData

[![Crates.io](https://img.shields.io/crates/v/datasynth-core.svg)](https://crates.io/crates/datasynth-core)
[![Documentation](https://docs.rs/datasynth-core/badge.svg)](https://docs.rs/datasynth-core)
[![License](https://img.shields.io/badge/license-Apache%202.0-green.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-1.75%2B-orange.svg)](https://www.rust-lang.org)
[![CI](https://github.com/ey-asu-rnd/SyntheticData/actions/workflows/ci.yml/badge.svg)](https://github.com/ey-asu-rnd/SyntheticData/actions/workflows/ci.yml)

A high-performance, configurable synthetic data generator for enterprise financial simulation. SyntheticData produces realistic, interconnected General Ledger journal entries, Chart of Accounts, SAP HANA-compatible ACDOCA event logs, document flows, subledger records, banking/KYC/AML transactions, OCEL 2.0 process mining data, and ML-ready graph exports at scale.

**Developed by [Ernst & Young Ltd.](https://www.ey.com/ch), Zurich, Switzerland**

---

## Table of Contents

- [SyntheticData](#syntheticdata)
  - [Table of Contents](#table-of-contents)
  - [Overview](#overview)
  - [Key Features](#key-features)
    - [Core Data Generation](#core-data-generation)
    - [Enterprise Simulation](#enterprise-simulation)
    - [Machine Learning \& Analytics](#machine-learning--analytics)
    - [Production Features](#production-features)
  - [Architecture](#architecture)
  - [Installation](#installation)
    - [From Source](#from-source)
    - [Requirements](#requirements)
  - [Quick Start](#quick-start)
    - [Demo Mode](#demo-mode)
  - [Configuration](#configuration)
  - [Output Structure](#output-structure)
  - [Use Cases](#use-cases)
  - [Performance](#performance)
  - [Server Usage](#server-usage)
  - [Desktop UI](#desktop-ui)
  - [Documentation](#documentation)
  - [License](#license)
  - [Support](#support)
  - [Acknowledgments](#acknowledgments)

---

## Overview

SyntheticData generates coherent enterprise financial data that mirrors the characteristics of real corporate accounting systems. The generated data is suitable for:

- **Machine Learning Model Development**: Training fraud detection, anomaly detection, and graph neural network models
- **Audit Analytics Testing**: Validating audit procedures and analytical tools with realistic data patterns
- **SOX Compliance Testing**: Testing internal controls and segregation of duties monitoring systems
- **System Integration Testing**: Load and stress testing for ERP and accounting platforms
- **Process Mining**: Generating realistic event logs for process discovery and conformance checking
- **Training and Education**: Providing realistic accounting data for professional development

The generator produces statistically accurate data based on empirical research from real-world general ledger patterns, ensuring that synthetic datasets exhibit the same characteristics as production data—including Benford's Law compliance, temporal patterns, and document flow integrity.

---

## Key Features

### Core Data Generation

| Feature | Description |
|---------|-------------|
| **Statistical Distributions** | Line item counts, amounts, and patterns based on empirical GL research |
| **Benford's Law Compliance** | First-digit distribution following Benford's Law with configurable fraud patterns |
| **Industry Presets** | Manufacturing, Retail, Financial Services, Healthcare, Technology, and more |
| **Chart of Accounts** | Small (~100), Medium (~400), Large (~2500) account structures |
| **Temporal Patterns** | Month-end, quarter-end, year-end volume spikes with working hour modeling |
| **Regional Calendars** | Holiday calendars for US, DE, GB, CN, JP, IN with lunar calendar support |

### Enterprise Simulation

- **Master Data Management**: Vendors, customers, materials, fixed assets, employees with temporal validity
- **Document Flow Engine**: Complete P2P (Procure-to-Pay) and O2C (Order-to-Cash) processes
- **Intercompany Transactions**: IC matching, transfer pricing, consolidation eliminations
- **Balance Coherence**: Opening balances, running balance tracking, trial balance generation
- **Subledger Simulation**: AR, AP, Fixed Assets, Inventory with GL reconciliation
- **Currency & FX**: Realistic exchange rates, currency translation, CTA generation
- **Period Close Engine**: Monthly close, depreciation runs, accruals, year-end closing
- **Banking/KYC/AML**: Customer personas, KYC profiles, AML typologies (structuring, funnel, mule, layering)
- **Process Mining**: OCEL 2.0 event logs with object-centric relationships
- **Audit Simulation**: ISA-compliant engagements, workpapers, findings, risk assessments

### Machine Learning & Analytics

- **Graph Export**: PyTorch Geometric, Neo4j, and DGL formats with train/val/test splits
- **Anomaly Injection**: 20+ fraud types, errors, process issues with full labeling
- **Data Quality Variations**: Missing values, format variations, duplicates, typos

### Production Features

- **REST & gRPC APIs**: Streaming generation with authentication and rate limiting
- **Desktop UI**: Cross-platform Tauri/SvelteKit application
- **Resource Guards**: Memory, disk, and CPU monitoring with graceful degradation
- **Evaluation Framework**: Auto-tuning with configuration recommendations
- **Deterministic Generation**: Seeded RNG for reproducible output

---

## Architecture

SyntheticData is organized as a Rust workspace with 14 modular crates:

```
datasynth-cli          Command-line interface (binary: datasynth-data)
datasynth-server       REST/gRPC/WebSocket server with auth and rate limiting
datasynth-ui           Tauri/SvelteKit desktop application
    │
datasynth-runtime      Orchestration layer (parallel execution, resource guards)
    │
datasynth-generators   Data generators (JE, documents, subledgers, anomalies, audit)
datasynth-banking      KYC/AML banking transaction generator
datasynth-ocpm         Object-Centric Process Mining (OCEL 2.0)
    │
datasynth-graph        Graph/network export (PyTorch Geometric, Neo4j, DGL)
datasynth-eval         Evaluation framework with auto-tuning
    │
datasynth-config       Configuration schema, validation, industry presets
    │
datasynth-core         Domain models, traits, distributions, resource guards
    │
datasynth-output       Output sinks (CSV, JSON, Parquet, streaming)
datasynth-test-utils   Test utilities, fixtures, mocks
```

See individual crate READMEs for detailed documentation.

---

## Installation

### From crates.io

```bash
# Install the CLI tool
cargo install datasynth-cli

# Or add individual crates to your project
cargo add datasynth-core datasynth-generators datasynth-config
```

### From Source

```bash
git clone https://github.com/ey-asu-rnd/SyntheticData.git
cd SyntheticData
cargo build --release
```

The binary is available at `target/release/datasynth-data`.

### Available Crates

| Crate | Description |
|-------|-------------|
| [`datasynth-core`](https://crates.io/crates/datasynth-core) | Domain models, traits, distributions |
| [`datasynth-config`](https://crates.io/crates/datasynth-config) | Configuration schema and validation |
| [`datasynth-generators`](https://crates.io/crates/datasynth-generators) | Data generators |
| [`datasynth-banking`](https://crates.io/crates/datasynth-banking) | KYC/AML banking transactions |
| [`datasynth-graph`](https://crates.io/crates/datasynth-graph) | Graph/network export |
| [`datasynth-eval`](https://crates.io/crates/datasynth-eval) | Evaluation framework |
| [`datasynth-runtime`](https://crates.io/crates/datasynth-runtime) | Orchestration layer |
| [`datasynth-cli`](https://crates.io/crates/datasynth-cli) | Command-line interface |
| [`datasynth-server`](https://crates.io/crates/datasynth-server) | REST/gRPC server |

### Requirements

- Rust 1.75 or later
- For the desktop UI: Node.js 18+ and platform-specific Tauri dependencies

---

## Quick Start

```bash
# Generate a configuration file for a manufacturing company
datasynth-data init --industry manufacturing --complexity medium -o config.yaml

# Validate the configuration
datasynth-data validate --config config.yaml

# Generate synthetic data
datasynth-data generate --config config.yaml --output ./output

# View available presets and options
datasynth-data info
```

### Demo Mode

```bash
# Quick demo with default settings
datasynth-data generate --demo --output ./demo-output
```

---

## Configuration

SyntheticData uses YAML configuration files with comprehensive options:

```yaml
global:
  seed: 42                        # For reproducible generation
  industry: manufacturing
  start_date: 2024-01-01
  period_months: 12
  group_currency: USD

companies:
  - code: "1000"
    name: "Headquarters"
    currency: USD
    country: US
    volume_weight: 1.0            # Transaction volume weight

transactions:
  target_count: 100000
  benford:
    enabled: true

fraud:
  enabled: true
  fraud_rate: 0.005               # 0.5% fraud rate

anomaly_injection:
  enabled: true
  total_rate: 0.02
  generate_labels: true           # For supervised learning

graph_export:
  enabled: true
  formats:
    - pytorch_geometric
    - neo4j

output:
  format: csv
  compression: none
```

See the [Configuration Guide](docs/configuration.md) for complete documentation.

---

## Output Structure

```
output/
├── master_data/          Vendors, customers, materials, assets, employees
├── transactions/         Journal entries, purchase orders, invoices, payments
├── subledgers/           AR, AP, FA, inventory detail records
├── period_close/         Trial balances, accruals, closing entries
├── consolidation/        Eliminations, currency translation
├── fx/                   Exchange rates, CTA adjustments
├── banking/              KYC profiles, bank transactions, AML typology labels
├── process_mining/       OCEL 2.0 event logs, process variants
├── audit/                Engagements, workpapers, findings, risk assessments
├── graphs/               PyTorch Geometric, Neo4j, DGL exports
├── labels/               Anomaly, fraud, and data quality labels for ML
└── controls/             Internal control mappings, SoD rules
```

---

## Use Cases

| Use Case | Description |
|----------|-------------|
| **Fraud Detection ML** | Train supervised models with labeled fraud patterns |
| **Graph Neural Networks** | Entity relationship graphs for anomaly detection |
| **AML/KYC Testing** | Banking transaction data with structuring, layering, mule patterns |
| **Audit Analytics** | Test audit procedures with known control exceptions |
| **Process Mining** | OCEL 2.0 event logs for process discovery |
| **ERP Testing** | Load testing with realistic transaction volumes |
| **SOX Compliance** | Test internal control monitoring systems |
| **Data Quality ML** | Train models to detect missing values, typos, duplicates |

---

## Performance

| Metric | Performance |
|--------|-------------|
| Single-threaded throughput | ~100,000+ entries/second |
| Parallel scaling | Linear with available cores |
| Memory efficiency | Streaming generation for large volumes |

---

## Server Usage

```bash
# Start REST/gRPC server
cargo run -p datasynth-server -- --port 3000 --worker-threads 4

# API endpoints
curl http://localhost:3000/api/config
curl -X POST http://localhost:3000/api/stream/start
```

WebSocket streaming available at `ws://localhost:3000/ws/events`.

---

## Desktop UI

```bash
cd crates/datasynth-ui
npm install
npm run tauri dev
```

The desktop application provides visual configuration, real-time streaming, and preset management.

---

## Python Wrapper

A Python wrapper is available for programmatic access:

```bash
cd python
pip install -e ".[all]"
```

```python
from datasynth_py import DataSynth
from datasynth_py.config import blueprints

config = blueprints.retail_small(companies=4, transactions=10000)
synth = DataSynth()
result = synth.generate(config=config, output={"format": "csv", "sink": "temp_dir"})
print(result.output_dir)
```

See the [Python Wrapper Guide](docs/src/user-guide/python-wrapper.md) for complete documentation.

---

## Documentation

- [Configuration Guide](docs/configuration.md)
- [API Reference](docs/api.md)
- [Architecture Overview](docs/architecture.md)
- [Python Wrapper Guide](docs/src/user-guide/python-wrapper.md)
- [Contributing Guidelines](CONTRIBUTING.md)

---

## License

Copyright 2024-2026 Michael Ivertowski, Ernst & Young Ltd., Zurich, Switzerland

Licensed under the Apache License, Version 2.0. See [LICENSE](LICENSE) for details.

---

## Support

Commercial support, custom development, and enterprise licensing are available upon request. Please contact the author at [michael.ivertowski@ch.ey.com](mailto:michael.ivertowski@ch.ey.com) for inquiries.

---

## Acknowledgments

This project incorporates research on statistical distributions in accounting data and implements industry-standard patterns for enterprise financial systems.

---

*SyntheticData is provided "as is" without warranty of any kind. It is intended for testing, development, and educational purposes. Generated data should not be used as a substitute for real financial records.*
