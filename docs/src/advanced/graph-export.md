# Graph Export

Export transaction data as ML-ready graphs.

## Overview

Graph export transforms financial data into network representations:

- Transaction networks (accounts and entities)
- Approval networks (users and approvals)
- Entity relationship graphs (ownership)

## Configuration

```yaml
graph_export:
  enabled: true

  formats:
    - pytorch_geometric
    - neo4j
    - dgl

  graphs:
    - transaction_network
    - approval_network
    - entity_relationship

  split:
    train: 0.7
    val: 0.15
    test: 0.15
    stratify: is_anomaly

  features:
    temporal: true
    amount: true
    structural: true
    categorical: true
```

## Graph Types

### Transaction Network

Accounts and entities as nodes, transactions as edges.

```
     ┌──────────┐
     │ Account  │
     │  1100    │
     └────┬─────┘
          │ $1000
          ▼
     ┌──────────┐
     │ Customer │
     │  C-001   │
     └──────────┘
```

**Nodes:**
- GL accounts
- Vendors
- Customers
- Cost centers

**Edges:**
- Journal entry lines
- Payments
- Invoices

### Approval Network

Users as nodes, approval relationships as edges.

```
     ┌──────────┐
     │  Clerk   │
     │  U-001   │
     └────┬─────┘
          │ approved
          ▼
     ┌──────────┐
     │ Manager  │
     │  U-002   │
     └──────────┘
```

**Nodes:** Employees/users
**Edges:** Approval actions

### Entity Relationship Network

Legal entities with ownership relationships.

```
     ┌──────────┐
     │  Parent  │
     │  1000    │
     └────┬─────┘
          │ 100%
          ▼
     ┌──────────┐
     │   Sub    │
     │  2000    │
     └──────────┘
```

**Nodes:** Companies
**Edges:** Ownership, IC transactions

## Export Formats

### PyTorch Geometric

```
output/graphs/transaction_network/pytorch_geometric/
├── node_features.pt    # [num_nodes, num_features]
├── edge_index.pt       # [2, num_edges]
├── edge_attr.pt        # [num_edges, num_edge_features]
├── labels.pt           # Labels
├── train_mask.pt       # Boolean training mask
├── val_mask.pt         # Boolean validation mask
└── test_mask.pt        # Boolean test mask
```

**Loading in Python:**

```python
import torch
from torch_geometric.data import Data

# Load tensors
node_features = torch.load('node_features.pt')
edge_index = torch.load('edge_index.pt')
edge_attr = torch.load('edge_attr.pt')
labels = torch.load('labels.pt')
train_mask = torch.load('train_mask.pt')

# Create PyG Data object
data = Data(
    x=node_features,
    edge_index=edge_index,
    edge_attr=edge_attr,
    y=labels,
    train_mask=train_mask,
)

print(f"Nodes: {data.num_nodes}")
print(f"Edges: {data.num_edges}")
```

### Neo4j

```
output/graphs/transaction_network/neo4j/
├── nodes_account.csv
├── nodes_vendor.csv
├── nodes_customer.csv
├── edges_transaction.csv
├── edges_payment.csv
└── import.cypher
```

**Import script (import.cypher):**

```cypher
// Load accounts
LOAD CSV WITH HEADERS FROM 'file:///nodes_account.csv' AS row
CREATE (:Account {
    id: row.id,
    name: row.name,
    type: row.type
});

// Load transactions
LOAD CSV WITH HEADERS FROM 'file:///edges_transaction.csv' AS row
MATCH (from:Account {id: row.from_id})
MATCH (to:Account {id: row.to_id})
CREATE (from)-[:TRANSACTION {
    amount: toFloat(row.amount),
    date: date(row.posting_date),
    is_anomaly: toBoolean(row.is_anomaly)
}]->(to);
```

### DGL (Deep Graph Library)

```
output/graphs/transaction_network/dgl/
├── graph.bin           # Serialized DGL graph
├── node_feats.npy      # Node features
├── edge_feats.npy      # Edge features
└── labels.npy          # Labels
```

**Loading in Python:**

```python
import dgl
import numpy as np

# Load graph
graph = dgl.load_graphs('graph.bin')[0][0]

# Load features
graph.ndata['feat'] = torch.tensor(np.load('node_feats.npy'))
graph.edata['feat'] = torch.tensor(np.load('edge_feats.npy'))
graph.ndata['label'] = torch.tensor(np.load('labels.npy'))
```

## Features

### Temporal Features

```yaml
features:
  temporal: true
```

| Feature | Description |
|---------|-------------|
| `weekday` | Day of week (0-6) |
| `period` | Fiscal period (1-12) |
| `is_month_end` | Last 3 days of month |
| `is_quarter_end` | Last week of quarter |
| `is_year_end` | Last month of year |
| `hour` | Hour of posting |

### Amount Features

```yaml
features:
  amount: true
```

| Feature | Description |
|---------|-------------|
| `log_amount` | log10(amount) |
| `benford_prob` | Expected first-digit probability |
| `is_round_number` | Ends in 00, 000, etc. |
| `amount_zscore` | Standard deviations from mean |

### Structural Features

```yaml
features:
  structural: true
```

| Feature | Description |
|---------|-------------|
| `line_count` | Number of JE lines |
| `unique_accounts` | Distinct accounts used |
| `has_intercompany` | IC transaction flag |
| `debit_credit_ratio` | Total debits / credits |

### Categorical Features

```yaml
features:
  categorical: true
```

One-hot encoded:
- `business_process`: Manual, P2P, O2C, etc.
- `source_type`: System, User, Recurring
- `account_type`: Asset, Liability, etc.

## Train/Val/Test Splits

```yaml
split:
  train: 0.7                         # 70% training
  val: 0.15                          # 15% validation
  test: 0.15                         # 15% test
  stratify: is_anomaly               # Maintain anomaly ratio
  random_seed: 42                    # Reproducible splits
```

**Stratification options:**
- `is_anomaly`: Balanced anomaly detection
- `is_fraud`: Balanced fraud detection
- `account_type`: Balanced by account type
- `null`: Random (no stratification)

## GNN Training Example

```python
import torch
from torch_geometric.nn import GCNConv

class AnomalyGNN(torch.nn.Module):
    def __init__(self, num_features, hidden_dim):
        super().__init__()
        self.conv1 = GCNConv(num_features, hidden_dim)
        self.conv2 = GCNConv(hidden_dim, 2)  # Binary classification

    def forward(self, data):
        x, edge_index = data.x, data.edge_index
        x = self.conv1(x, edge_index).relu()
        x = self.conv2(x, edge_index)
        return x

# Train
model = AnomalyGNN(data.num_features, 64)
optimizer = torch.optim.Adam(model.parameters(), lr=0.01)

for epoch in range(100):
    model.train()
    optimizer.zero_grad()
    out = model(data)
    loss = F.cross_entropy(out[data.train_mask], data.y[data.train_mask])
    loss.backward()
    optimizer.step()
```

## See Also

- [Anomaly Injection](anomaly-injection.md)
- [Fraud Detection Use Case](../use-cases/fraud-detection.md)
- [synth-graph Crate](../crates/synth-graph.md)
