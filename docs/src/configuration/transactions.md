# Transactions

Transaction settings control journal entry generation.

## Configuration

```yaml
transactions:
  target_count: 100000

  line_items:
    distribution: empirical
    min_lines: 2
    max_lines: 20

  amounts:
    min: 100
    max: 1000000
    distribution: log_normal
    round_number_bias: 0.15

  sources:
    manual: 0.3
    automated: 0.5
    recurring: 0.15
    adjustment: 0.05

  benford:
    enabled: true

  temporal:
    month_end_spike: 2.5
    quarter_end_spike: 3.0
    year_end_spike: 4.0
    working_hours_only: true
```

## Fields

### target_count

Total number of journal entries to generate.

| Property | Value |
|----------|-------|
| Type | `u64` |
| Required | Yes |

```yaml
transactions:
  target_count: 10000      # Small dataset
  target_count: 100000     # Medium dataset
  target_count: 1000000    # Large dataset
```

### line_items

Controls the number of line items per journal entry.

#### distribution

| Value | Description |
|-------|-------------|
| `empirical` | Based on real-world GL research |
| `uniform` | Equal probability for all counts |
| `custom` | User-defined probabilities |

**Empirical distribution** (default):
- 2 lines: 60.68%
- 3 lines: 5.24%
- 4 lines: 17.32%
- Even counts: 88% preference

```yaml
line_items:
  distribution: empirical
```

**Custom distribution:**

```yaml
line_items:
  distribution: custom
  custom_distribution:
    2: 0.50
    3: 0.10
    4: 0.20
    5: 0.10
    6: 0.10
```

#### min_lines / max_lines

| Property | Value |
|----------|-------|
| Type | `u32` |
| Default | 2 / 20 |

```yaml
line_items:
  min_lines: 2
  max_lines: 10
```

### amounts

Controls transaction amounts.

#### min / max

| Property | Value |
|----------|-------|
| Type | `f64` |
| Required | Yes |

```yaml
amounts:
  min: 100           # Minimum amount
  max: 1000000       # Maximum amount
```

#### distribution

| Value | Description |
|-------|-------------|
| `log_normal` | Log-normal distribution (realistic) |
| `uniform` | Equal probability across range |
| `custom` | User-defined |

```yaml
amounts:
  distribution: log_normal
```

#### round_number_bias

Preference for round numbers (100, 500, 1000, etc.).

| Property | Value |
|----------|-------|
| Type | `f64` |
| Range | 0.0 - 1.0 |
| Default | 0.15 |

```yaml
amounts:
  round_number_bias: 0.15    # 15% round numbers
  round_number_bias: 0.0     # No round number bias
```

### sources

Transaction source distribution (weights must sum to 1.0).

| Source | Description |
|--------|-------------|
| `manual` | Manual journal entries |
| `automated` | System-generated |
| `recurring` | Scheduled recurring entries |
| `adjustment` | Period-end adjustments |

```yaml
sources:
  manual: 0.3
  automated: 0.5
  recurring: 0.15
  adjustment: 0.05
```

### benford

Benford's Law compliance for first-digit distribution.

```yaml
benford:
  enabled: true       # Follow P(d) = log10(1 + 1/d)
  enabled: false      # Disable Benford compliance
```

**Expected distribution (enabled):**

| Digit | Probability |
|-------|-------------|
| 1 | 30.1% |
| 2 | 17.6% |
| 3 | 12.5% |
| 4 | 9.7% |
| 5 | 7.9% |
| 6 | 6.7% |
| 7 | 5.8% |
| 8 | 5.1% |
| 9 | 4.6% |

### temporal

Temporal patterns for transaction timing.

#### Spikes

Volume multipliers for period ends:

```yaml
temporal:
  month_end_spike: 2.5    # 2.5x volume at month end
  quarter_end_spike: 3.0  # 3.0x at quarter end
  year_end_spike: 4.0     # 4.0x at year end
```

#### Working Hours

Restrict transactions to business hours:

```yaml
temporal:
  working_hours_only: true   # Mon-Fri, 8am-6pm
  working_hours_only: false  # Any time
```

## Examples

### High Volume Retail

```yaml
transactions:
  target_count: 500000

  line_items:
    distribution: empirical
    min_lines: 2
    max_lines: 6

  amounts:
    min: 10
    max: 50000
    distribution: log_normal
    round_number_bias: 0.3

  sources:
    manual: 0.1
    automated: 0.8
    recurring: 0.08
    adjustment: 0.02

  temporal:
    month_end_spike: 1.5
    quarter_end_spike: 2.0
    year_end_spike: 5.0
```

### Low Volume Manual

```yaml
transactions:
  target_count: 5000

  line_items:
    distribution: empirical

  amounts:
    min: 1000
    max: 10000000

  sources:
    manual: 0.6
    automated: 0.2
    recurring: 0.1
    adjustment: 0.1

  temporal:
    month_end_spike: 3.0
    quarter_end_spike: 4.0
    year_end_spike: 5.0
    working_hours_only: true
```

### Testing/Development

```yaml
transactions:
  target_count: 1000

  line_items:
    distribution: uniform
    min_lines: 2
    max_lines: 4

  amounts:
    min: 100
    max: 10000
    distribution: uniform
    round_number_bias: 0.0

  sources:
    manual: 1.0

  benford:
    enabled: false
```

## Validation

| Check | Rule |
|-------|------|
| `target_count` | > 0 |
| `min_lines` | ≥ 2 |
| `max_lines` | ≥ min_lines |
| `amounts.min` | > 0 |
| `amounts.max` | > min |
| `round_number_bias` | 0.0 - 1.0 |
| `sources` | Sum = 1.0 (±0.01) |
| `*_spike` | ≥ 1.0 |

## See Also

- [Configuration Overview](README.md)
- [Document Flows](document-flows.md)
- [Anomaly Injection](../advanced/anomaly-injection.md)
