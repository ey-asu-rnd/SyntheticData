# Quick Start

This guide walks you through generating your first synthetic financial dataset.

## Overview

The typical workflow is:

1. **Initialize** a configuration file
2. **Validate** the configuration
3. **Generate** synthetic data
4. **Review** the output

## Step 1: Initialize Configuration

Create a configuration file for your industry and complexity needs:

```bash
# Manufacturing company with medium complexity (~400 accounts)
synth-data init --industry manufacturing --complexity medium -o config.yaml
```

### Available Industry Presets

| Industry | Description |
|----------|-------------|
| `manufacturing` | Production, inventory, cost accounting |
| `retail` | Sales, inventory, customer transactions |
| `financial_services` | Banking, investments, regulatory reporting |
| `healthcare` | Patient revenue, medical supplies, compliance |
| `technology` | R&D, SaaS revenue, deferred revenue |

### Complexity Levels

| Level | Accounts | Description |
|-------|----------|-------------|
| `small` | ~100 | Simple chart of accounts |
| `medium` | ~400 | Typical mid-size company |
| `large` | ~2500 | Enterprise-scale structure |

## Step 2: Review Configuration

Open `config.yaml` to review and customize:

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
    volume_weight: 1.0

transactions:
  target_count: 100000            # Number of journal entries

fraud:
  enabled: true
  fraud_rate: 0.005               # 0.5% fraud transactions

output:
  format: csv
  compression: none
```

See the [Configuration Guide](../configuration/README.md) for all options.

## Step 3: Validate Configuration

Check your configuration for errors:

```bash
synth-data validate --config config.yaml
```

The validator checks:
- Required fields are present
- Values are within valid ranges
- Distribution weights sum to 1.0
- Dates are consistent

## Step 4: Generate Data

Run the generation:

```bash
synth-data generate --config config.yaml --output ./output
```

You'll see a progress bar:

```
Generating synthetic data...
[████████████████████████████████] 100000/100000 entries
Generation complete in 1.2s
```

## Step 5: Explore Output

The output directory contains organized subdirectories:

```
output/
├── master_data/
│   ├── vendors.csv
│   ├── customers.csv
│   ├── materials.csv
│   └── employees.csv
├── transactions/
│   ├── journal_entries.csv
│   ├── acdoca.csv
│   ├── purchase_orders.csv
│   └── vendor_invoices.csv
├── subledgers/
│   ├── ar_open_items.csv
│   └── ap_open_items.csv
├── period_close/
│   └── trial_balances/
├── labels/
│   ├── anomaly_labels.csv
│   └── fraud_labels.csv
└── controls/
    └── internal_controls.csv
```

## Common Customizations

### Generate More Data

```yaml
transactions:
  target_count: 1000000           # 1 million entries
```

### Enable Graph Export

```yaml
graph_export:
  enabled: true
  formats:
    - pytorch_geometric
    - neo4j
```

### Add Anomaly Injection

```yaml
anomaly_injection:
  enabled: true
  total_rate: 0.02                # 2% anomaly rate
  generate_labels: true           # For ML training
```

### Multiple Companies

```yaml
companies:
  - code: "1000"
    name: "Headquarters"
    currency: USD
    volume_weight: 0.6

  - code: "2000"
    name: "European Subsidiary"
    currency: EUR
    volume_weight: 0.4
```

## Next Steps

- Explore [Demo Mode](demo-mode.md) for built-in presets
- Learn the [CLI Reference](../user-guide/cli-reference.md)
- Review [Output Formats](../user-guide/output-formats.md)
- See [Configuration](../configuration/README.md) for all options

## Quick Reference

```bash
# Common commands
synth-data init --industry <industry> --complexity <level> -o config.yaml
synth-data validate --config config.yaml
synth-data generate --config config.yaml --output ./output
synth-data generate --demo --output ./demo-output
synth-data info                   # Show available presets
```
