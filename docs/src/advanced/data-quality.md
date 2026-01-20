# Data Quality Variations

Generate realistic data quality issues for testing robustness.

## Overview

Real-world data has imperfections. The data quality module introduces:

- Missing values (various patterns)
- Format variations
- Duplicates
- Typos and transcription errors
- Encoding issues

## Configuration

```yaml
data_quality:
  enabled: true

  missing_values:
    rate: 0.01
    pattern: mcar

  format_variations:
    date_formats: true
    amount_formats: true

  duplicates:
    rate: 0.001
    types: [exact, near, fuzzy]

  typos:
    rate: 0.005
    keyboard_aware: true
```

## Missing Values

### Patterns

| Pattern | Description |
|---------|-------------|
| `mcar` | Missing Completely At Random |
| `mar` | Missing At Random (conditional) |
| `mnar` | Missing Not At Random (value-dependent) |
| `systematic` | Entire field groups missing |

```yaml
data_quality:
  missing_values:
    rate: 0.01                       # 1% missing overall
    pattern: mcar

    # Pattern-specific settings
    mcar:
      uniform: true                  # Equal probability all fields

    mar:
      conditions:
        - field: vendor_name
          dependent_on: is_intercompany
          probability: 0.1

    mnar:
      conditions:
        - field: amount
          when_above: 100000         # Large amounts more likely missing
          probability: 0.05

    systematic:
      groups:
        - [address, city, country]   # All or none
```

### Field Targeting

```yaml
data_quality:
  missing_values:
    fields:
      description: 0.02              # 2% missing
      cost_center: 0.05              # 5% missing
      tax_code: 0.03                 # 3% missing
    exclude:
      - document_id                  # Never make missing
      - posting_date
      - account_number
```

## Format Variations

### Date Formats

```yaml
data_quality:
  format_variations:
    date_formats: true
    date_variations:
      iso: 0.6                       # 2024-01-15
      us: 0.2                        # 01/15/2024
      eu: 0.1                        # 15.01.2024
      long: 0.1                      # January 15, 2024
```

**Examples:**
- ISO: `2024-01-15`
- US: `01/15/2024`, `1/15/2024`
- EU: `15.01.2024`, `15/01/2024`
- Long: `January 15, 2024`

### Amount Formats

```yaml
data_quality:
  format_variations:
    amount_formats: true
    amount_variations:
      plain: 0.5                     # 1234.56
      us_comma: 0.3                  # 1,234.56
      eu_format: 0.1                 # 1.234,56
      currency_prefix: 0.05          # $1,234.56
      currency_suffix: 0.05          # 1.234,56 EUR
```

### Identifier Formats

```yaml
data_quality:
  format_variations:
    identifier_variations:
      case: 0.1                      # INV-001 vs inv-001
      padding: 0.1                   # 001 vs 1
      separator: 0.1                 # INV-001 vs INV_001 vs INV001
```

## Duplicates

### Duplicate Types

| Type | Description |
|------|-------------|
| `exact` | Identical records |
| `near` | Minor field differences |
| `fuzzy` | Multiple field variations |

```yaml
data_quality:
  duplicates:
    rate: 0.001                      # 0.1% duplicates
    types:
      exact: 0.4                     # 40% exact duplicates
      near: 0.4                      # 40% near duplicates
      fuzzy: 0.2                     # 20% fuzzy duplicates
```

### Near Duplicate Variations

```yaml
data_quality:
  duplicates:
    near:
      fields_to_vary: 1              # Change 1 field
      variations:
        - field: posting_date
          offset_days: [-1, 0, 1]
        - field: amount
          variance: 0.001            # 0.1% difference
```

### Fuzzy Duplicate Variations

```yaml
data_quality:
  duplicates:
    fuzzy:
      fields_to_vary: 3              # Change multiple fields
      include_typos: true
```

## Typos

### Typo Types

| Type | Description |
|------|-------------|
| Substitution | Adjacent key pressed |
| Transposition | Characters swapped |
| Insertion | Extra character |
| Deletion | Missing character |
| OCR errors | Scan-related (0/O, 1/l) |
| Homophones | Sound-alike substitution |

```yaml
data_quality:
  typos:
    rate: 0.005                      # 0.5% of string fields
    keyboard_aware: true             # Use QWERTY layout

    types:
      substitution: 0.35             # Adjacnet → Adjacent
      transposition: 0.25            # Recieve → Receive
      insertion: 0.15                # Shippping → Shipping
      deletion: 0.15                 # Shiping → Shipping
      ocr_errors: 0.05               # O → 0, l → 1
      homophones: 0.05               # their → there
```

### Field Targeting

```yaml
data_quality:
  typos:
    fields:
      description: 0.02              # More likely in descriptions
      vendor_name: 0.01
      customer_name: 0.01
    exclude:
      - account_number               # Never introduce typos
      - document_id
```

## Encoding Issues

```yaml
data_quality:
  encoding:
    enabled: true
    rate: 0.001

    issues:
      mojibake: 0.4                  # UTF-8/Latin-1 confusion
      missing_chars: 0.3             # Characters dropped
      bom_issues: 0.2                # BOM artifacts
      html_entities: 0.1             # &amp; instead of &
```

**Examples:**
- Mojibake: `Müller` → `MÃ¼ller`
- Missing: `Zürich` → `Zrich`
- HTML: `R&D` → `R&amp;D`

## ML Training Labels

The data quality module generates labels for ML model training:

### QualityIssueLabel

```rust
pub struct QualityIssueLabel {
    pub issue_id: String,
    pub issue_type: LabeledIssueType,
    pub issue_subtype: Option<QualityIssueSubtype>,
    pub document_id: String,
    pub field_name: String,
    pub original_value: Option<String>,
    pub modified_value: Option<String>,
    pub severity: u8,  // 1-5
    pub processor: String,
    pub metadata: HashMap<String, String>,
}
```

### Issue Types

| Type | Severity | Description |
|------|----------|-------------|
| `MissingValue` | 3 | Field is null/empty |
| `Typo` | 2 | Character-level errors |
| `FormatVariation` | 1 | Different formatting |
| `Duplicate` | 4 | Duplicate record |
| `EncodingIssue` | 3 | Character encoding problems |
| `Inconsistency` | 3 | Cross-field inconsistency |
| `OutOfRange` | 4 | Value outside expected range |
| `InvalidReference` | 5 | Reference to non-existent entity |

### Subtypes

Each issue type has detailed subtypes:

- **Typo**: Substitution, Transposition, Insertion, Deletion, DoubleChar, CaseError, OcrError, Homophone
- **FormatVariation**: DateFormat, AmountFormat, IdentifierFormat, TextFormat
- **Duplicate**: ExactDuplicate, NearDuplicate, FuzzyDuplicate, CrossSystemDuplicate
- **EncodingIssue**: Mojibake, MissingChars, Bom, ControlChars, HtmlEntities

## Output

### quality_issues.csv

| Field | Description |
|-------|-------------|
| `document_id` | Affected record |
| `field_name` | Field with issue |
| `issue_type` | missing, typo, duplicate, etc. |
| `original_value` | Value before modification |
| `modified_value` | Value after modification |

### quality_labels.csv (ML Training)

| Field | Description |
|-------|-------------|
| `issue_id` | Unique issue identifier |
| `issue_type` | LabeledIssueType enum |
| `issue_subtype` | Detailed subtype |
| `document_id` | Affected document |
| `field_name` | Affected field |
| `original_value` | Original value |
| `modified_value` | Modified value |
| `severity` | 1-5 severity score |
| `processor` | Which processor injected |

## Example Configurations

### Testing Data Pipelines

```yaml
data_quality:
  enabled: true

  missing_values:
    rate: 0.02
    pattern: mcar

  format_variations:
    date_formats: true
    amount_formats: true

  typos:
    rate: 0.01
    keyboard_aware: true
```

### Testing Deduplication

```yaml
data_quality:
  enabled: true

  duplicates:
    rate: 0.05                       # High duplicate rate
    types:
      exact: 0.3
      near: 0.4
      fuzzy: 0.3
```

### Testing OCR Processing

```yaml
data_quality:
  enabled: true

  typos:
    rate: 0.03
    types:
      ocr_errors: 0.8                # Mostly OCR-style errors
      substitution: 0.2
```

## See Also

- [Anomaly Injection](anomaly-injection.md)
- [Output Formats](../user-guide/output-formats.md)
- [datasynth-generators](../crates/datasynth-generators.md)
