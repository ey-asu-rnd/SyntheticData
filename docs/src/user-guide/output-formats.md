# Output Formats

SyntheticData generates multiple file types organized into categories.

## Output Directory Structure

```
output/
├── master_data/          # Entity master records
├── transactions/         # Journal entries and documents
├── subledgers/           # Subsidiary ledger records
├── period_close/         # Trial balances and closing
├── consolidation/        # Elimination entries
├── fx/                   # Exchange rates
├── graphs/               # ML-ready graph exports
├── labels/               # Anomaly/fraud labels
└── controls/             # Internal control mappings
```

## File Formats

### CSV

Default format with standard conventions:
- UTF-8 encoding
- Comma-separated values
- Header row included
- Quoted strings when needed

**Example (journal_entries.csv):**
```csv
document_id,posting_date,company_code,account,description,debit,credit,is_fraud
abc-123,2024-01-15,1000,1100,Customer payment,"1000.00","0.00",false
abc-123,2024-01-15,1000,1200,Cash receipt,"0.00","1000.00",false
```

### JSON

Structured format with nested objects:

**Example (journal_entries.json):**
```json
[
  {
    "header": {
      "document_id": "abc-123",
      "posting_date": "2024-01-15",
      "company_code": "1000",
      "source": "Manual",
      "is_fraud": false
    },
    "lines": [
      {
        "account": "1100",
        "description": "Customer payment",
        "debit": "1000.00",
        "credit": "0.00"
      },
      {
        "account": "1200",
        "description": "Cash receipt",
        "debit": "0.00",
        "credit": "1000.00"
      }
    ]
  }
]
```

### ACDOCA (SAP HANA)

SAP Universal Journal format with simulation fields:

| Field | Description |
|-------|-------------|
| RCLNT | Client |
| RLDNR | Ledger |
| RBUKRS | Company code |
| GJAHR | Fiscal year |
| BELNR | Document number |
| DOCLN | Line item |
| RYEAR | Year |
| POPER | Posting period |
| RACCT | Account |
| DRCRK | Debit/Credit indicator |
| HSL | Amount in local currency |
| ZSIM_* | Simulation metadata fields |

## Master Data Files

### chart_of_accounts.csv

| Field | Description |
|-------|-------------|
| account_number | GL account code |
| account_name | Display name |
| account_type | Asset, Liability, Equity, Revenue, Expense |
| account_subtype | Detailed classification |
| is_control_account | Links to subledger |
| normal_balance | Debit or Credit |

### vendors.csv

| Field | Description |
|-------|-------------|
| vendor_id | Unique identifier |
| vendor_name | Company name |
| tax_id | Tax identification |
| payment_terms | Standard terms |
| currency | Transaction currency |
| is_intercompany | IC flag |

### customers.csv

| Field | Description |
|-------|-------------|
| customer_id | Unique identifier |
| customer_name | Company/person name |
| credit_limit | Maximum credit |
| credit_rating | Rating code |
| payment_behavior | Typical payment pattern |

### materials.csv

| Field | Description |
|-------|-------------|
| material_id | Unique identifier |
| description | Material name |
| material_type | Classification |
| valuation_method | FIFO, LIFO, Avg |
| standard_cost | Unit cost |

### employees.csv

| Field | Description |
|-------|-------------|
| employee_id | Unique identifier |
| name | Full name |
| department | Department code |
| manager_id | Hierarchy link |
| approval_limit | Maximum approval amount |
| transaction_codes | Authorized T-codes |

## Transaction Files

### journal_entries.csv

| Field | Description |
|-------|-------------|
| document_id | Entry identifier |
| company_code | Company |
| fiscal_year | Year |
| fiscal_period | Period |
| posting_date | Date posted |
| document_date | Original date |
| source | Transaction source |
| business_process | Process category |
| is_fraud | Fraud indicator |
| is_anomaly | Anomaly indicator |

### Line Items (embedded or separate)

| Field | Description |
|-------|-------------|
| line_number | Sequence |
| account_number | GL account |
| cost_center | Cost center |
| profit_center | Profit center |
| debit_amount | Debit |
| credit_amount | Credit |
| description | Line description |

### Document Flow Files

**purchase_orders.csv:**
- Order header with vendor, dates, status
- Line items with materials, quantities, prices

**goods_receipts.csv:**
- Receipt linked to PO
- Quantities received, variances

**vendor_invoices.csv:**
- Invoice with three-way match status
- Payment terms, due date

**payments.csv:**
- Payment documents
- Bank references, cleared invoices

## Subledger Files

### ar_open_items.csv

| Field | Description |
|-------|-------------|
| customer_id | Customer reference |
| invoice_number | Document number |
| invoice_date | Date issued |
| due_date | Payment due |
| original_amount | Invoice total |
| open_amount | Remaining balance |
| aging_bucket | 0-30, 31-60, 61-90, 90+ |

### ap_open_items.csv

Similar structure for payables.

### fa_register.csv

| Field | Description |
|-------|-------------|
| asset_id | Asset identifier |
| description | Asset name |
| acquisition_date | Purchase date |
| acquisition_cost | Original cost |
| useful_life_years | Depreciation period |
| depreciation_method | Straight-line, etc. |
| accumulated_depreciation | Total depreciation |
| net_book_value | Current value |

### inventory_positions.csv

| Field | Description |
|-------|-------------|
| material_id | Material reference |
| warehouse | Location |
| quantity | Units on hand |
| unit_cost | Current cost |
| total_value | Extended value |

## Period Close Files

### trial_balances/YYYY_MM.csv

| Field | Description |
|-------|-------------|
| account_number | GL account |
| account_name | Description |
| opening_balance | Period start |
| period_debits | Total debits |
| period_credits | Total credits |
| closing_balance | Period end |

### accruals.csv

Accrual entries with reversal dates.

### depreciation.csv

Monthly depreciation entries per asset.

## Graph Export Files

### PyTorch Geometric

```
graphs/transaction_network/pytorch_geometric/
├── node_features.pt    # [num_nodes, features]
├── edge_index.pt       # [2, num_edges]
├── edge_attr.pt        # [num_edges, edge_features]
├── labels.pt           # Node/edge labels
├── train_mask.pt       # Training split
├── val_mask.pt         # Validation split
└── test_mask.pt        # Test split
```

### Neo4j

```
graphs/entity_relationship/neo4j/
├── nodes_account.csv
├── nodes_entity.csv
├── nodes_user.csv
├── edges_transaction.csv
├── edges_approval.csv
└── import.cypher        # Import script
```

## Label Files

### anomaly_labels.csv

| Field | Description |
|-------|-------------|
| document_id | Entry reference |
| anomaly_id | Unique anomaly ID |
| anomaly_type | Classification |
| anomaly_category | Fraud, Error, Process, etc. |
| severity | Low, Medium, High |
| description | Human-readable explanation |

### fraud_labels.csv

| Field | Description |
|-------|-------------|
| document_id | Entry reference |
| fraud_type | Specific fraud pattern |
| detection_difficulty | Easy, Medium, Hard |
| description | Fraud scenario description |

## Control Files

### internal_controls.csv

| Field | Description |
|-------|-------------|
| control_id | Unique identifier |
| control_name | Description |
| control_type | Preventive, Detective |
| frequency | Continuous, Daily, etc. |
| assertions | Completeness, Accuracy, etc. |

### sod_rules.csv

Segregation of duties conflict definitions.

## Compression Options

| Option | Extension | Use Case |
|--------|-----------|----------|
| none | .csv/.json | Development, small datasets |
| gzip | .csv.gz | General compression |
| zstd | .csv.zst | High performance |

## Configuration

```yaml
output:
  format: csv              # csv or json
  compression: none        # none, gzip, zstd
  compression_level: 6     # 1-9 (if compression enabled)
```

## See Also

- [Configuration](../configuration/output-settings.md)
- [Graph Export](../advanced/graph-export.md)
- [Anomaly Injection](../advanced/anomaly-injection.md)
