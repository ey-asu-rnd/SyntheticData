# Master Data

Master data settings control generation of business entities.

## Configuration

```yaml
master_data:
  vendors:
    count: 200
    intercompany_ratio: 0.05

  customers:
    count: 500
    intercompany_ratio: 0.05

  materials:
    count: 1000

  fixed_assets:
    count: 100

  employees:
    count: 50
    hierarchy_depth: 4
```

## Vendors

Supplier master data configuration.

```yaml
master_data:
  vendors:
    count: 200                    # Number of vendors
    intercompany_ratio: 0.05      # IC vendor percentage

    payment_terms:
      - code: "NET30"
        days: 30
        weight: 0.5
      - code: "NET60"
        days: 60
        weight: 0.3
      - code: "NET10"
        days: 10
        weight: 0.2

    behavior:
      late_payment_rate: 0.1      # % with late payment tendency
      discount_usage_rate: 0.3    # % that take early pay discounts
```

### Generated Fields

| Field | Description |
|-------|-------------|
| `vendor_id` | Unique identifier |
| `vendor_name` | Company name |
| `tax_id` | Tax identification number |
| `payment_terms` | Default payment terms |
| `currency` | Transaction currency |
| `bank_account` | Bank details |
| `is_intercompany` | IC vendor flag |
| `valid_from` | Temporal validity start |

## Customers

Customer master data configuration.

```yaml
master_data:
  customers:
    count: 500                    # Number of customers
    intercompany_ratio: 0.05      # IC customer percentage

    credit_rating:
      - code: "AAA"
        limit_multiplier: 10.0
        weight: 0.1
      - code: "AA"
        limit_multiplier: 5.0
        weight: 0.2
      - code: "A"
        limit_multiplier: 2.0
        weight: 0.4
      - code: "B"
        limit_multiplier: 1.0
        weight: 0.3

    payment_behavior:
      on_time_rate: 0.7           # % that pay on time
      early_payment_rate: 0.1     # % that pay early
      late_payment_rate: 0.2      # % that pay late
```

### Generated Fields

| Field | Description |
|-------|-------------|
| `customer_id` | Unique identifier |
| `customer_name` | Company/person name |
| `credit_limit` | Maximum credit |
| `credit_rating` | Rating code |
| `payment_behavior` | Payment tendency |
| `currency` | Transaction currency |
| `is_intercompany` | IC customer flag |

## Materials

Product/material master data.

```yaml
master_data:
  materials:
    count: 1000                   # Number of materials

    types:
      raw_material: 0.3
      work_in_progress: 0.1
      finished_goods: 0.4
      services: 0.2

    valuation:
      - method: fifo
        weight: 0.3
      - method: weighted_average
        weight: 0.5
      - method: standard_cost
        weight: 0.2
```

### Generated Fields

| Field | Description |
|-------|-------------|
| `material_id` | Unique identifier |
| `description` | Material name |
| `material_type` | Classification |
| `unit_of_measure` | UOM |
| `valuation_method` | Costing method |
| `standard_cost` | Unit cost |
| `gl_account` | Inventory account |

## Fixed Assets

Capital asset master data.

```yaml
master_data:
  fixed_assets:
    count: 100                    # Number of assets

    categories:
      buildings: 0.1
      machinery: 0.3
      vehicles: 0.2
      furniture: 0.2
      it_equipment: 0.2

    depreciation:
      - method: straight_line
        weight: 0.7
      - method: declining_balance
        weight: 0.2
      - method: units_of_production
        weight: 0.1
```

### Generated Fields

| Field | Description |
|-------|-------------|
| `asset_id` | Unique identifier |
| `description` | Asset name |
| `asset_class` | Category |
| `acquisition_date` | Purchase date |
| `acquisition_cost` | Original cost |
| `useful_life` | Years |
| `depreciation_method` | Method |
| `salvage_value` | Residual value |

## Employees

User/employee master data.

```yaml
master_data:
  employees:
    count: 50                     # Number of employees
    hierarchy_depth: 4            # Org chart depth

    roles:
      - name: "AP Clerk"
        approval_limit: 5000
        weight: 0.3
      - name: "AP Manager"
        approval_limit: 50000
        weight: 0.1
      - name: "AR Clerk"
        approval_limit: 5000
        weight: 0.3
      - name: "Controller"
        approval_limit: 500000
        weight: 0.1
      - name: "CFO"
        approval_limit: 999999999
        weight: 0.05

    transaction_codes:
      - "FB01"     # Post document
      - "FB50"     # Enter GL
      - "F-28"     # Post incoming payment
      - "F-53"     # Post outgoing payment
```

### Generated Fields

| Field | Description |
|-------|-------------|
| `employee_id` | Unique identifier |
| `name` | Full name |
| `department` | Department code |
| `role` | Job role |
| `manager_id` | Supervisor reference |
| `approval_limit` | Max approval amount |
| `transaction_codes` | Authorized T-codes |

## Examples

### Small Company

```yaml
master_data:
  vendors:
    count: 50
  customers:
    count: 100
  materials:
    count: 200
  fixed_assets:
    count: 20
  employees:
    count: 10
    hierarchy_depth: 2
```

### Large Enterprise

```yaml
master_data:
  vendors:
    count: 2000
    intercompany_ratio: 0.1
  customers:
    count: 10000
    intercompany_ratio: 0.1
  materials:
    count: 50000
  fixed_assets:
    count: 5000
  employees:
    count: 500
    hierarchy_depth: 8
```

## Validation

| Check | Rule |
|-------|------|
| `count` | > 0 |
| `intercompany_ratio` | 0.0 - 1.0 |
| `hierarchy_depth` | ≥ 1 |
| Distribution weights | Sum = 1.0 |

## See Also

- [Document Flows](document-flows.md)
- [Compliance](compliance.md)
- [synth-generators](../crates/synth-generators.md)
