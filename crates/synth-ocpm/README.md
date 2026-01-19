# synth-ocpm

Object-Centric Process Mining (OCPM) models and generators.

## Overview

`synth-ocpm` provides OCEL 2.0 compliant event log generation:

- **OCEL 2.0 Models**: Events, objects, relationships per IEEE standard
- **Process Generators**: P2P (Procure-to-Pay), O2C (Order-to-Cash) flows
- **Export Formats**: OCEL 2.0 JSON, XML, and SQLite

## OCEL 2.0 Compliance

Implements the Object-Centric Event Log standard:

| Element | Description |
|---------|-------------|
| Events | Activities with timestamps and attributes |
| Objects | Business objects (PO, Invoice, Payment, etc.) |
| Object Types | Type definitions with attribute schemas |
| Relationships | Object-to-object relationships |
| Event-Object Links | Many-to-many event-object associations |

## Process Flows

### Procure-to-Pay (P2P)

```
Create PO → Approve PO → Release PO → Create GR → Post GR →
Receive Invoice → Verify Invoice → Post Invoice → Execute Payment
```

### Order-to-Cash (O2C)

```
Create SO → Check Credit → Release SO → Create Delivery →
Pick → Pack → Ship → Create Invoice → Post Invoice → Receive Payment
```

## Usage

```rust
use synth_ocpm::{OcpmGenerator, P2pDocuments};

let mut generator = OcpmGenerator::new(seed);
let documents = P2pDocuments::new("PO-001", "V-001", "1000", amount, "USD");

let result = generator.generate_p2p_case(&documents, start_time, &users);
```

## Export

```rust
use synth_ocpm::export::{Ocel2Exporter, ExportFormat};

let exporter = Ocel2Exporter::new(ExportFormat::Json);
exporter.export(&event_log, "output/ocel2.json")?;
```

## License

Apache-2.0 - See [LICENSE](../../LICENSE) for details.
