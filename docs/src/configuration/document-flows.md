# Document Flows

Document flow settings control P2P and O2C process generation.

## Configuration

```yaml
document_flows:
  p2p:
    enabled: true
    flow_rate: 0.3
    completion_rate: 0.95
    three_way_match:
      quantity_tolerance: 0.02
      price_tolerance: 0.01

  o2c:
    enabled: true
    flow_rate: 0.3
    completion_rate: 0.95
```

## Procure-to-Pay (P2P)

Purchase requisition through payment.

### Flow

```
Create PO → Goods Receipt → Vendor Invoice → Payment
```

### Configuration

```yaml
document_flows:
  p2p:
    enabled: true                    # Enable P2P generation
    flow_rate: 0.3                   # % of JEs from P2P
    completion_rate: 0.95            # % completing full flow

    stages:
      po_approval_rate: 0.9          # POs that get approved
      gr_rate: 0.98                  # POs with goods receipts
      invoice_rate: 0.95             # GRs with invoices
      payment_rate: 0.92             # Invoices that get paid

    three_way_match:
      enabled: true
      quantity_tolerance: 0.02       # 2% quantity variance allowed
      price_tolerance: 0.01          # 1% price variance allowed

    timing:
      po_to_gr_days:
        min: 1
        max: 30
      gr_to_invoice_days:
        min: 1
        max: 14
      invoice_to_payment_days:
        min: 10
        max: 60
```

### Generated Documents

| Document | Fields |
|----------|--------|
| **Purchase Order** | PO number, vendor, items, quantities, prices |
| **Goods Receipt** | GR number, PO reference, received quantities |
| **Vendor Invoice** | Invoice number, PO/GR reference, amounts |
| **Payment** | Payment number, invoice references |

### Three-Way Match

Validates PO, GR, and Invoice alignment:

```yaml
three_way_match:
  enabled: true
  quantity_tolerance: 0.02    # Allow 2% quantity variance
  price_tolerance: 0.01       # Allow 1% price variance

  variance_types:
    - over_receipt             # GR > PO quantity
    - under_receipt            # GR < PO quantity
    - price_variance           # Invoice price ≠ PO price
```

**Match outcomes:**
- `matched`: All within tolerance
- `quantity_variance`: Quantity outside tolerance
- `price_variance`: Price outside tolerance
- `blocked`: Manual review required

### Journal Entries Generated

| Stage | Debit | Credit |
|-------|-------|--------|
| Goods Receipt | Inventory | GR/IR Clearing |
| Invoice Receipt | GR/IR Clearing | Accounts Payable |
| Payment | Accounts Payable | Cash |

---

## Order-to-Cash (O2C)

Sales order through cash receipt.

### Flow

```
Create SO → Delivery → Customer Invoice → Customer Receipt
```

### Configuration

```yaml
document_flows:
  o2c:
    enabled: true                    # Enable O2C generation
    flow_rate: 0.3                   # % of JEs from O2C
    completion_rate: 0.95            # % completing full flow

    stages:
      so_approval_rate: 0.95         # SOs that get approved
      credit_check_pass_rate: 0.9    # Pass credit check
      delivery_rate: 0.98            # SOs with deliveries
      invoice_rate: 0.95             # Deliveries with invoices
      collection_rate: 0.85          # Invoices that get paid

    timing:
      so_to_delivery_days:
        min: 1
        max: 14
      delivery_to_invoice_days:
        min: 0
        max: 3
      invoice_to_payment_days:
        min: 15
        max: 90
```

### Generated Documents

| Document | Fields |
|----------|--------|
| **Sales Order** | SO number, customer, items, quantities, prices |
| **Delivery** | Delivery number, SO reference, shipped quantities |
| **Customer Invoice** | Invoice number, SO/delivery reference, amounts |
| **Customer Receipt** | Receipt number, invoice references |

### Credit Check

```yaml
o2c:
  credit_check:
    enabled: true
    check_credit_limit: true       # Verify customer limit
    check_overdue: true            # Check for past-due AR
    block_threshold: 0.9           # Block if >90% of limit used
```

### Journal Entries Generated

| Stage | Debit | Credit |
|-------|-------|--------|
| Delivery | Cost of Goods Sold | Inventory |
| Invoice | Accounts Receivable | Revenue |
| Receipt | Cash | Accounts Receivable |

---

## Combined Example

```yaml
document_flows:
  p2p:
    enabled: true
    flow_rate: 0.35
    completion_rate: 0.95

    three_way_match:
      quantity_tolerance: 0.02
      price_tolerance: 0.01

    timing:
      po_to_gr_days:
        min: 3
        max: 21
      gr_to_invoice_days:
        min: 1
        max: 10
      invoice_to_payment_days:
        min: 20
        max: 45

  o2c:
    enabled: true
    flow_rate: 0.35
    completion_rate: 0.90

    credit_check:
      enabled: true
      block_threshold: 0.85

    timing:
      so_to_delivery_days:
        min: 1
        max: 7
      delivery_to_invoice_days:
        min: 0
        max: 1
      invoice_to_payment_days:
        min: 30
        max: 60
```

## Document References

Documents maintain proper reference chains:

```
PO-001 → GR-001 → INV-001 → PAY-001
   │         │         │         │
   └─────────┴─────────┴─────────┘
              Document Chain
```

Reference types:
- `follows_from`: Next document in flow
- `payment_for`: Payment → Invoice link
- `reversal_of`: Credit memo / reversal

## Validation

| Check | Rule |
|-------|------|
| `flow_rate` | 0.0 - 1.0 |
| `completion_rate` | 0.0 - 1.0 |
| `tolerance` values | 0.0 - 1.0 |
| `timing.min` | ≥ 0 |
| `timing.max` | ≥ min |

## See Also

- [Master Data](master-data.md)
- [Financial Settings](financial-settings.md)
- [synth-generators](../crates/synth-generators.md)
