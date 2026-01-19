# synth-graph

Graph/network export for synthetic accounting data with ML-ready formats.

## Overview

`synth-graph` provides graph construction and export capabilities:

- **Graph Builders**: Transaction, approval, and entity relationship graphs
- **ML Export**: PyTorch Geometric, Neo4j, and DGL formats
- **Feature Engineering**: Temporal, amount, structural, and categorical features
- **Data Splits**: Train/validation/test split generation

## Graph Types

| Graph | Nodes | Edges | Use Case |
|-------|-------|-------|----------|
| Transaction Network | Accounts, Entities | Transactions | Anomaly detection |
| Approval Network | Users | Approvals | SoD analysis |
| Entity Relationship | Legal Entities | Ownership | Consolidation analysis |

## Export Formats

### PyTorch Geometric

```
graphs/transaction_network/pytorch_geometric/
├── node_features.pt    # [num_nodes, num_features]
├── edge_index.pt       # [2, num_edges]
├── edge_attr.pt        # [num_edges, num_edge_features]
├── labels.pt           # [num_nodes] or [num_edges]
├── train_mask.pt       # Boolean mask
├── val_mask.pt
└── test_mask.pt
```

### Neo4j

```
graphs/entity_relationship/neo4j/
├── nodes_account.csv
├── nodes_entity.csv
├── edges_transaction.csv
├── edges_ownership.csv
└── import.cypher
```

### DGL (Deep Graph Library)

```
graphs/approval_network/dgl/
├── graph.bin           # DGL graph object
├── node_feats.npy      # Node features
├── edge_feats.npy      # Edge features
└── labels.npy          # Labels
```

## Feature Categories

| Category | Features |
|----------|----------|
| Temporal | weekday, period, is_month_end, is_quarter_end, is_year_end |
| Amount | log(amount), benford_probability, is_round_number |
| Structural | line_count, unique_accounts, has_intercompany |
| Categorical | business_process (one-hot), source_type (one-hot) |

## Key Types

### Graph Models

```rust
pub struct Graph {
    pub nodes: Vec<Node>,
    pub edges: Vec<Edge>,
    pub node_features: Option<Array2<f32>>,
    pub edge_features: Option<Array2<f32>>,
}

pub enum Node {
    Account(AccountNode),
    Entity(EntityNode),
    User(UserNode),
    Transaction(TransactionNode),
}

pub enum Edge {
    Transaction(TransactionEdge),
    Approval(ApprovalEdge),
    Ownership(OwnershipEdge),
}
```

### Split Configuration

```rust
pub struct SplitConfig {
    pub train_ratio: f64,     // e.g., 0.7
    pub val_ratio: f64,       // e.g., 0.15
    pub test_ratio: f64,      // e.g., 0.15
    pub stratify_by: Option<String>,
    pub random_seed: u64,
}
```

## Usage Examples

### Building Transaction Graph

```rust
use synth_graph::{TransactionGraphBuilder, GraphConfig};

let builder = TransactionGraphBuilder::new(GraphConfig::default());
let graph = builder.build(&journal_entries)?;

println!("Nodes: {}", graph.nodes.len());
println!("Edges: {}", graph.edges.len());
```

### PyTorch Geometric Export

```rust
use synth_graph::{PyTorchGeometricExporter, SplitConfig};

let exporter = PyTorchGeometricExporter::new("output/graphs");

let split = SplitConfig {
    train_ratio: 0.7,
    val_ratio: 0.15,
    test_ratio: 0.15,
    stratify_by: Some("is_anomaly".to_string()),
    random_seed: 42,
};

exporter.export(&graph, split)?;
```

### Neo4j Export

```rust
use synth_graph::Neo4jExporter;

let exporter = Neo4jExporter::new("output/graphs/neo4j");
exporter.export(&graph)?;

// Generates import script:
// LOAD CSV WITH HEADERS FROM 'file:///nodes_account.csv' AS row
// CREATE (:Account {id: row.id, name: row.name, ...})
```

### Feature Engineering

```rust
use synth_graph::features::{FeatureExtractor, FeatureConfig};

let extractor = FeatureExtractor::new(FeatureConfig {
    temporal: true,
    amount: true,
    structural: true,
    categorical: true,
});

let node_features = extractor.extract_node_features(&entries)?;
let edge_features = extractor.extract_edge_features(&entries)?;
```

## Graph Construction

### Transaction Network

Accounts and entities become nodes; transactions become edges.

```rust
// Nodes:
// - Each GL account is a node
// - Each vendor/customer is a node

// Edges:
// - Each journal entry line creates an edge
// - Edge connects account to entity
// - Edge features: amount, date, fraud flag
```

### Approval Network

Users become nodes; approval relationships become edges.

```rust
// Nodes:
// - Each user/employee is a node
// - Node features: approval_limit, department, role

// Edges:
// - Approval actions create edges
// - Edge features: amount, threshold, escalation
```

### Entity Relationship Network

Legal entities become nodes; ownership and IC relationships become edges.

```rust
// Nodes:
// - Each company/legal entity is a node
// - Node features: currency, country, parent_flag

// Edges:
// - Ownership relationships (parent → subsidiary)
// - IC transaction relationships
// - Edge features: ownership_percent, transaction_volume
```

## ML Integration

### Loading in PyTorch

```python
import torch
from torch_geometric.data import Data

# Load exported data
node_features = torch.load('node_features.pt')
edge_index = torch.load('edge_index.pt')
edge_attr = torch.load('edge_attr.pt')
labels = torch.load('labels.pt')
train_mask = torch.load('train_mask.pt')

data = Data(
    x=node_features,
    edge_index=edge_index,
    edge_attr=edge_attr,
    y=labels,
    train_mask=train_mask,
)
```

### Loading in Neo4j

```bash
# Import using generated script
neo4j-admin import \
    --nodes=nodes_account.csv \
    --nodes=nodes_entity.csv \
    --relationships=edges_transaction.csv
```

## Configuration

```yaml
graph_export:
  enabled: true
  formats:
    - pytorch_geometric
    - neo4j
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

## See Also

- [Graph Export](../advanced/graph-export.md)
- [Fraud Detection Use Case](../use-cases/fraud-detection.md)
- [synth-generators](synth-generators.md)
