# ERP Load Testing

Generate high-volume data for ERP system testing.

## Overview

SyntheticData generates realistic transaction volumes for:

- Load testing
- Stress testing
- Performance benchmarking
- System integration testing

## Configuration

### High Volume Generation

```yaml
global:
  seed: 42
  industry: manufacturing
  start_date: 2024-01-01
  period_months: 12
  worker_threads: 8                  # Maximize parallelism

transactions:
  target_count: 1000000              # 1 million entries

  line_items:
    distribution: empirical

  amounts:
    min: 100
    max: 10000000
    distribution: log_normal

  sources:
    manual: 0.15
    automated: 0.65
    recurring: 0.15
    adjustment: 0.05

  temporal:
    month_end_spike: 2.5
    quarter_end_spike: 3.0
    year_end_spike: 4.0

document_flows:
  p2p:
    enabled: true
    flow_rate: 0.35
  o2c:
    enabled: true
    flow_rate: 0.35

master_data:
  vendors:
    count: 2000
  customers:
    count: 5000
  materials:
    count: 10000

output:
  format: csv
  compression: none                  # Fastest for import
```

### SAP ACDOCA Format

```yaml
output:
  files:
    journal_entries: false
    acdoca: true                     # SAP Universal Journal format
```

## Volume Sizing

### Transaction Volume Guidelines

| Company Size | Annual Entries | Per Day | Configuration |
|--------------|----------------|---------|---------------|
| Small | 10,000 | ~30 | `target_count: 10000` |
| Medium | 100,000 | ~300 | `target_count: 100000` |
| Large | 1,000,000 | ~3,000 | `target_count: 1000000` |
| Enterprise | 10,000,000 | ~30,000 | `target_count: 10000000` |

### Master Data Guidelines

| Size | Vendors | Customers | Materials |
|------|---------|-----------|-----------|
| Small | 100 | 200 | 500 |
| Medium | 500 | 1,000 | 5,000 |
| Large | 2,000 | 10,000 | 50,000 |
| Enterprise | 10,000 | 100,000 | 500,000 |

## Load Testing Scenarios

### 1. Steady State Load

Normal daily operation:

```yaml
transactions:
  target_count: 100000

  temporal:
    month_end_spike: 1.0             # No spikes
    quarter_end_spike: 1.0
    year_end_spike: 1.0
    working_hours_only: true
```

### 2. Peak Period Load

Month-end closing:

```yaml
global:
  start_date: 2024-01-25
  period_months: 1                   # Focus on month-end

transactions:
  target_count: 50000

  temporal:
    month_end_spike: 5.0             # 5x normal volume
```

### 3. Year-End Stress

Year-end closing simulation:

```yaml
global:
  start_date: 2024-12-01
  period_months: 1

transactions:
  target_count: 200000

  temporal:
    month_end_spike: 3.0
    quarter_end_spike: 4.0
    year_end_spike: 10.0             # Extreme spike
```

### 4. Batch Import

Large batch import testing:

```yaml
transactions:
  target_count: 500000

  sources:
    automated: 1.0                   # All system-generated

output:
  compression: none                  # For fastest import
```

## Performance Monitoring

### Generation Metrics

```bash
# Time generation
time datasynth-data generate --config config.yaml --output ./output

# Monitor memory
/usr/bin/time -v datasynth-data generate --config config.yaml --output ./output

# Watch progress
datasynth-data generate --config config.yaml --output ./output -v
```

### Import Metrics

Track these during ERP import:

| Metric | Description |
|--------|-------------|
| Import rate | Records per second |
| Memory usage | Peak RAM during import |
| CPU utilization | Processor load |
| I/O throughput | Disk read/write speed |
| Lock contention | Database lock waits |

## Data Import Strategies

### SAP S/4HANA

```bash
# Generate ACDOCA format
datasynth-data generate --config config.yaml --output ./output

# Use SAP Data Services or LSMW for import
# Output: output/transactions/acdoca.csv
```

### Oracle EBS

```sql
-- Create staging table
CREATE TABLE XX_JE_STAGING (
    document_id VARCHAR2(36),
    posting_date DATE,
    account VARCHAR2(20),
    debit NUMBER,
    credit NUMBER
);

-- Load via SQL*Loader
LOAD DATA
INFILE 'journal_entries.csv'
INTO TABLE XX_JE_STAGING
FIELDS TERMINATED BY ','
```

### Microsoft Dynamics

```powershell
# Use Data Management Framework
# Import journal_entries.csv via Data Entity
```

## Validation

### Post-Import Checks

```sql
-- Verify record count
SELECT COUNT(*) FROM journal_entries;

-- Verify balance
SELECT SUM(debit) - SUM(credit) AS imbalance
FROM journal_entries;

-- Check date range
SELECT MIN(posting_date), MAX(posting_date)
FROM journal_entries;
```

### Reconciliation

```python
import pandas as pd

# Compare source and target
source = pd.read_csv('output/transactions/journal_entries.csv')
target = pd.read_sql('SELECT * FROM journal_entries', connection)

# Verify counts
assert len(source) == len(target), "Record count mismatch"

# Verify totals
assert abs(source['debit_amount'].sum() - target['debit'].sum()) < 0.01
```

## Batch Processing

### Chunked Generation

For very large volumes, generate in chunks:

```bash
# Generate 10 batches of 1M each
for i in {1..10}; do
    datasynth-data generate \
        --config config.yaml \
        --output ./output/batch_$i \
        --seed $((42 + i))
done
```

### Parallel Import

```bash
# Import chunks in parallel
for batch in ./output/batch_*; do
    import_job $batch &
done
wait
```

## Performance Tips

### Generation Speed

1. **Increase threads:** `worker_threads: 16`
2. **Disable unnecessary features:** Turn off graph export, anomalies
3. **Use fast storage:** NVMe SSD
4. **Reduce complexity:** Smaller COA, fewer master records

### Import Speed

1. **Disable triggers:** During bulk import
2. **Drop indexes:** Recreate after import
3. **Increase batch size:** Larger commits
4. **Parallel loading:** Multiple import streams

## See Also

- [Performance Tuning](../advanced/performance.md)
- [Output Formats](../user-guide/output-formats.md)
- [Configuration](../configuration/README.md)
