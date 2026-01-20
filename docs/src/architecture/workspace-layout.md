# Workspace Layout

SyntheticData is organized as a Rust workspace with 12 crates.

## Crate Hierarchy

```
datasynth-cli          → Binary entry point
datasynth-server       → REST/gRPC/WebSocket server
datasynth-ui           → Desktop application
    │
    ▼
datasynth-runtime      → Generation orchestration
    │
    ├─────────────────┐
    ▼                 ▼
datasynth-generators  datasynth-graph
    │                 │
    └────────┬────────┘
             ▼
    ┌────────┴────────┐
    ▼                 ▼
datasynth-config     datasynth-output
    │
    ▼
datasynth-core         → Foundation layer

datasynth-eval         → Evaluation (standalone)
datasynth-ocpm         → Process mining (standalone)
datasynth-test-utils   → Testing utilities
```

## Dependency Matrix

| Crate | Depends On |
|-------|------------|
| datasynth-core | (none) |
| datasynth-config | datasynth-core |
| datasynth-output | datasynth-core |
| datasynth-generators | datasynth-core, datasynth-config |
| datasynth-graph | datasynth-core, datasynth-generators |
| datasynth-runtime | datasynth-core, datasynth-config, datasynth-generators, datasynth-output, datasynth-graph |
| datasynth-cli | datasynth-runtime |
| datasynth-server | datasynth-runtime |
| datasynth-ui | datasynth-runtime (via Tauri) |
| datasynth-eval | datasynth-core |
| datasynth-ocpm | datasynth-core |
| datasynth-test-utils | datasynth-core |

## Directory Structure

```
SyntheticData/
├── Cargo.toml              # Workspace manifest
├── crates/
│   ├── datasynth-core/
│   │   ├── Cargo.toml
│   │   ├── README.md
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── models/
│   │       ├── distributions/
│   │       ├── traits/
│   │       └── templates/
│   ├── datasynth-config/
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── schema.rs
│   │       ├── validation.rs
│   │       └── presets/
│   ├── datasynth-generators/
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── je_generator.rs
│   │       ├── master_data/
│   │       ├── document_flow/
│   │       ├── intercompany/
│   │       ├── balance/
│   │       ├── subledger/
│   │       ├── fx/
│   │       ├── period_close/
│   │       ├── anomaly/
│   │       └── data_quality/
│   ├── datasynth-output/
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── csv_sink.rs
│   │       └── json_sink.rs
│   ├── datasynth-graph/
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── builders/
│   │       ├── exporters/
│   │       └── features/
│   ├── datasynth-runtime/
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── orchestrator.rs
│   │       └── progress.rs
│   ├── datasynth-cli/
│   │   ├── Cargo.toml
│   │   └── src/
│   │       └── main.rs
│   ├── datasynth-server/
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── main.rs
│   │       ├── rest/
│   │       ├── grpc/
│   │       └── websocket/
│   ├── datasynth-ui/
│   │   ├── package.json
│   │   ├── src/              # Svelte frontend
│   │   └── src-tauri/        # Rust backend
│   ├── datasynth-eval/
│   ├── datasynth-ocpm/
│   └── datasynth-test-utils/
├── benches/                  # Benchmark suite
├── docs/                     # This documentation
└── tests/                    # Integration tests
```

## Crate Purposes

### Application Layer

| Crate | Purpose |
|-------|---------|
| **datasynth-cli** | Command-line interface |
| **datasynth-server** | REST/gRPC/WebSocket API |
| **datasynth-ui** | Desktop application |

### Processing Layer

| Crate | Purpose |
|-------|---------|
| **datasynth-runtime** | Orchestrates generation workflow |
| **datasynth-generators** | All data generation logic |
| **datasynth-graph** | Graph construction and export |

### Foundation Layer

| Crate | Purpose |
|-------|---------|
| **datasynth-core** | Domain models, traits, distributions |
| **datasynth-config** | Configuration schema and validation |
| **datasynth-output** | Output sinks (CSV, JSON) |

### Supporting Crates

| Crate | Purpose |
|-------|---------|
| **datasynth-eval** | Quality evaluation framework |
| **datasynth-ocpm** | OCEL 2.0 process mining |
| **datasynth-test-utils** | Test fixtures and assertions |

## Build Commands

```bash
# Build entire workspace
cargo build --release

# Build specific crate
cargo build -p datasynth-core
cargo build -p datasynth-generators

# Run tests
cargo test
cargo test -p datasynth-core

# Generate documentation
cargo doc --workspace --no-deps

# Run benchmarks
cargo bench
```

## Feature Flags

Workspace-level features:

```toml
[workspace.features]
default = ["full"]
full = ["server", "ui", "graph"]
server = []
ui = []
graph = []
```

Crate-level features:

```toml
# datasynth-core
[features]
templates = ["serde_yaml"]

# datasynth-output
[features]
compression = ["flate2", "zstd"]
```

## Adding a New Crate

1. Create directory: `crates/synth-newcrate/`
2. Add `Cargo.toml`:
   ```toml
   [package]
   name = "synth-newcrate"
   version = "0.1.0"
   edition = "2021"

   [dependencies]
   datasynth-core = { path = "../datasynth-core" }
   ```
3. Add to workspace `Cargo.toml`:
   ```toml
   [workspace]
   members = [
       # ...
       "crates/synth-newcrate",
   ]
   ```
4. Create `src/lib.rs`

## See Also

- [Crate Reference](../crates/README.md)
- [Domain Models](domain-models.md)
- [Data Flow](data-flow.md)
