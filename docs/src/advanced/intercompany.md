# Intercompany Processing

Generate matched intercompany transactions and elimination entries.

## Overview

Intercompany features simulate multi-entity corporate structures:

- IC transaction pairs (seller/buyer)
- Transfer pricing
- IC reconciliation
- Consolidation eliminations

## Prerequisites

Multiple companies must be defined:

```yaml
companies:
  - code: "1000"
    name: "Parent Company"
    is_parent: true
    volume_weight: 0.5

  - code: "2000"
    name: "Subsidiary"
    parent_code: "1000"
    volume_weight: 0.5
```

## Configuration

```yaml
intercompany:
  enabled: true

  transaction_types:
    goods_sale: 0.4
    service_provided: 0.2
    loan: 0.15
    dividend: 0.1
    management_fee: 0.1
    royalty: 0.05

  transfer_pricing:
    method: cost_plus
    markup_range:
      min: 0.03
      max: 0.10

  elimination:
    enabled: true
    timing: quarterly
```

## IC Transaction Types

### Goods Sale

Internal sale of inventory between entities.

```
Seller (1000):
    Dr Intercompany Receivable   1,100
        Cr IC Revenue            1,100
    Dr IC COGS                     800
        Cr Inventory               800

Buyer (2000):
    Dr Inventory                 1,100
        Cr Intercompany Payable  1,100
```

### Service Provided

Internal services (IT, HR, legal).

```
Provider (1000):
    Dr IC Receivable               500
        Cr IC Service Revenue      500

Receiver (2000):
    Dr Service Expense             500
        Cr IC Payable              500
```

### Loan

Intercompany financing.

```
Lender (1000):
    Dr IC Loan Receivable       10,000
        Cr Cash                 10,000

Borrower (2000):
    Dr Cash                     10,000
        Cr IC Loan Payable      10,000
```

### Dividend

Upstream dividend payment.

```
Subsidiary (2000):
    Dr Retained Earnings         5,000
        Cr Cash                  5,000

Parent (1000):
    Dr Cash                      5,000
        Cr Dividend Income       5,000
```

### Management Fee

Corporate overhead allocation.

```
Parent (1000):
    Dr IC Receivable             1,000
        Cr Mgmt Fee Revenue      1,000

Subsidiary (2000):
    Dr Mgmt Fee Expense          1,000
        Cr IC Payable            1,000
```

### Royalty

IP licensing fees.

```
Licensor (1000):
    Dr IC Receivable               750
        Cr Royalty Revenue         750

Licensee (2000):
    Dr Royalty Expense             750
        Cr IC Payable              750
```

## Transfer Pricing

### Methods

| Method | Description |
|--------|-------------|
| `cost_plus` | Cost + markup percentage |
| `resale_minus` | Resale price - margin |
| `comparable_uncontrolled` | Market price |

```yaml
transfer_pricing:
  method: cost_plus
  markup_range:
    min: 0.03                        # 3% minimum markup
    max: 0.10                        # 10% maximum markup

  # OR for resale minus
  method: resale_minus
  margin_range:
    min: 0.15
    max: 0.25
```

### Arm's Length Pricing

Prices generated to be defensible:

```rust
fn calculate_transfer_price(cost: Decimal, method: &TransferPricingMethod) -> Decimal {
    match method {
        TransferPricingMethod::CostPlus { markup } => {
            cost * (Decimal::ONE + markup)
        }
        TransferPricingMethod::ResaleMinus { margin, resale_price } => {
            resale_price * (Decimal::ONE - margin)
        }
        TransferPricingMethod::Comparable { market_price } => {
            market_price
        }
    }
}
```

## IC Matching

### Matched Pair Structure

```rust
pub struct ICMatchedPair {
    pub pair_id: String,
    pub seller_company: String,
    pub buyer_company: String,
    pub seller_entry_id: Uuid,
    pub buyer_entry_id: Uuid,
    pub transaction_type: ICTransactionType,
    pub amount: Decimal,
    pub currency: String,
    pub transaction_date: NaiveDate,
}
```

### Match Validation

```yaml
intercompany:
  matching:
    enabled: true
    tolerance: 0.01                  # 1% variance allowed
    mismatch_rate: 0.02              # 2% intentional mismatches
```

**Match statuses:**
- `matched`: Amounts reconcile
- `timing_difference`: Different posting dates
- `fx_difference`: Currency conversion variance
- `unmatched`: No matching entry

## Eliminations

### Timing

```yaml
intercompany:
  elimination:
    timing: quarterly                # monthly, quarterly, annual
```

### Elimination Types

**Revenue/Expense Elimination:**
```
Elimination entry:
    Dr IC Revenue (1000)           1,100
        Cr IC Expense (2000)       1,100
```

**Unrealized Profit Elimination:**
```
If buyer still holds inventory:
    Dr IC Revenue                    300
        Cr Inventory                 300
```

**Receivable/Payable Elimination:**
```
    Dr IC Payable (2000)          10,000
        Cr IC Receivable (1000)   10,000
```

## Output Files

### ic_pairs.csv

| Field | Description |
|-------|-------------|
| `pair_id` | Unique pair identifier |
| `seller_company` | Selling entity |
| `buyer_company` | Buying entity |
| `seller_entry_id` | Seller's JE document ID |
| `buyer_entry_id` | Buyer's JE document ID |
| `transaction_type` | Type of IC transaction |
| `amount` | Transaction amount |
| `match_status` | Match result |

### eliminations.csv

| Field | Description |
|-------|-------------|
| `elimination_id` | Unique ID |
| `ic_pair_id` | Reference to IC pair |
| `elimination_type` | Revenue, profit, balance |
| `debit_company` | Company debited |
| `credit_company` | Company credited |
| `amount` | Elimination amount |
| `period` | Fiscal period |

## Example Configuration

### Multi-National Structure

```yaml
companies:
  - code: "1000"
    name: "US Headquarters"
    currency: USD
    country: US
    is_parent: true
    volume_weight: 0.4

  - code: "2000"
    name: "European Hub"
    currency: EUR
    country: DE
    parent_code: "1000"
    volume_weight: 0.3

  - code: "3000"
    name: "Asia Pacific"
    currency: JPY
    country: JP
    parent_code: "1000"
    volume_weight: 0.3

intercompany:
  enabled: true

  transaction_types:
    goods_sale: 0.5
    service_provided: 0.2
    management_fee: 0.15
    royalty: 0.15

  transfer_pricing:
    method: cost_plus
    markup_range:
      min: 0.05
      max: 0.12

  elimination:
    enabled: true
    timing: quarterly
```

## See Also

- [Companies Configuration](../configuration/companies.md)
- [Financial Settings](../configuration/financial-settings.md)
- [Period Close](period-close.md)
