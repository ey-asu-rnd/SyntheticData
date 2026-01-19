<div class="hero-section">

# SyntheticData

<p class="subtitle">High-Performance Synthetic Enterprise Financial Data Generator</p>

<div class="badges">

[![Version](https://img.shields.io/badge/version-0.1.0-blue.svg)](https://github.com/ey-asu-rnd/SyntheticData)
[![License](https://img.shields.io/badge/license-Apache%202.0-green.svg)](https://github.com/ey-asu-rnd/SyntheticData/blob/main/LICENSE)
[![Rust](https://img.shields.io/badge/rust-1.75%2B-orange.svg)](https://www.rust-lang.org)

</div>

<p class="attribution">Developed by <a href="https://www.ey.com/ch">Ernst & Young Ltd.</a>, Zurich, Switzerland</p>

</div>

## What is SyntheticData?

SyntheticData is a configurable synthetic data generator that produces realistic, interconnected enterprise financial data. It generates General Ledger journal entries, Chart of Accounts, SAP HANA-compatible ACDOCA event logs, document flows, subledger records, and ML-ready graph exports at scale.

The generator produces statistically accurate data based on empirical research from real-world general ledger patterns, ensuring that synthetic datasets exhibit the same characteristics as production data—including Benford's Law compliance, temporal patterns, and document flow integrity.

## Quick Links

| Section | Description |
|---------|-------------|
| [Getting Started](getting-started/README.md) | Installation, quick start guide, and demo mode |
| [User Guide](user-guide/README.md) | CLI reference, server API, desktop UI |
| [Configuration](configuration/README.md) | Complete YAML schema and presets |
| [Architecture](architecture/README.md) | System design and data flow |
| [Crate Reference](crates/README.md) | Detailed crate documentation |
| [Advanced Topics](advanced/README.md) | Anomaly injection, graph export, performance |
| [Use Cases](use-cases/README.md) | Fraud detection, audit, compliance |

## Key Features

### Core Data Generation

| Feature | Description |
|---------|-------------|
| **Statistical Distributions** | Line item counts, amounts, and patterns based on empirical GL research |
| **Benford's Law Compliance** | First-digit distribution following Benford's Law with configurable fraud patterns |
| **Industry Presets** | Manufacturing, Retail, Financial Services, Healthcare, Technology |
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

### Machine Learning & Analytics

- **Graph Export**: PyTorch Geometric, Neo4j, and DGL formats with train/val/test splits
- **Anomaly Injection**: 20+ fraud types, errors, process issues with full labeling
- **Data Quality Variations**: Missing values, format variations, duplicates, typos

### Production Features

- **REST & gRPC APIs**: Streaming generation with authentication and rate limiting
- **Desktop UI**: Cross-platform Tauri/SvelteKit application
- **Memory Management**: Configurable limits with OOM prevention
- **Deterministic Generation**: Seeded RNG for reproducible output

## Performance

| Metric | Performance |
|--------|-------------|
| Single-threaded throughput | ~100,000+ entries/second |
| Parallel scaling | Linear with available cores |
| Memory efficiency | Streaming generation for large volumes |

## Use Cases

| Use Case | Description |
|----------|-------------|
| **Fraud Detection ML** | Train supervised models with labeled fraud patterns |
| **Graph Neural Networks** | Entity relationship graphs for anomaly detection |
| **Audit Analytics** | Test audit procedures with known control exceptions |
| **Process Mining** | OCEL 2.0 event logs for process discovery |
| **ERP Testing** | Load testing with realistic transaction volumes |
| **SOX Compliance** | Test internal control monitoring systems |

## Quick Start

```bash
# Install from source
git clone https://github.com/ey-asu-rnd/SyntheticData.git
cd SyntheticData
cargo build --release

# Run demo mode
./target/release/synth-data generate --demo --output ./output

# Or create a custom configuration
./target/release/synth-data init --industry manufacturing --complexity medium -o config.yaml
./target/release/synth-data generate --config config.yaml --output ./output
```

## License

Copyright 2024-2026 Michael Ivertowski, Ernst & Young Ltd., Zurich, Switzerland

Licensed under the Apache License, Version 2.0. See [LICENSE](https://github.com/ey-asu-rnd/SyntheticData/blob/main/LICENSE) for details.

## Support

Commercial support, custom development, and enterprise licensing are available upon request. Please contact the author at [michael.ivertowski@ch.ey.com](mailto:michael.ivertowski@ch.ey.com) for inquiries.

---

*SyntheticData is provided "as is" without warranty of any kind. It is intended for testing, development, and educational purposes. Generated data should not be used as a substitute for real financial records.*
