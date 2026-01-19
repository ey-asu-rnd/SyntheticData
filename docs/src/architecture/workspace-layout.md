# Workspace Layout

SyntheticData is organized as a Rust workspace with 12 crates.

## Crate Hierarchy

```
synth-cli          → Binary entry point
synth-server       → REST/gRPC/WebSocket server
synth-ui           → Desktop application
    │
    ▼
synth-runtime      → Generation orchestration
    │
    ├─────────────────┐
    ▼                 ▼
synth-generators  synth-graph
    │                 │
    └────────┬────────┘
             ▼
    ┌────────┴────────┐
    ▼                 ▼
synth-config     synth-output
    │
    ▼
synth-core         → Foundation layer

synth-eval         → Evaluation (standalone)
synth-ocpm         → Process mining (standalone)
synth-test-utils   → Testing utilities
```

## Dependency Matrix

| Crate | Depends On |
|-------|------------|
| synth-core | (none) |
| synth-config | synth-core |
| synth-output | synth-core |
| synth-generators | synth-core, synth-config |
| synth-graph | synth-core, synth-generators |
| synth-runtime | synth-core, synth-config, synth-generators, synth-output, synth-graph |
| synth-cli | synth-runtime |
| synth-server | synth-runtime |
| synth-ui | synth-runtime (via Tauri) |
| synth-eval | synth-core |
| synth-ocpm | synth-core |
| synth-test-utils | synth-core |

## Directory Structure

```
SyntheticData/
├── Cargo.toml              # Workspace manifest
├── crates/
│   ├── synth-core/
│   │   ├── Cargo.toml
│   │   ├── README.md
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── models/
│   │       ├── distributions/
│   │       ├── traits/
│   │       └── templates/
│   ├── synth-config/
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── schema.rs
│   │       ├── validation.rs
│   │       └── presets/
│   ├── synth-generators/
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
│   ├── synth-output/
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── csv_sink.rs
│   │       └── json_sink.rs
│   ├── synth-graph/
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── builders/
│   │       ├── exporters/
│   │       └── features/
│   ├── synth-runtime/
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── orchestrator.rs
│   │       └── progress.rs
│   ├── synth-cli/
│   │   ├── Cargo.toml
│   │   └── src/
│   │       └── main.rs
│   ├── synth-server/
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── main.rs
│   │       ├── rest/
│   │       ├── grpc/
│   │       └── websocket/
│   ├── synth-ui/
│   │   ├── package.json
│   │   ├── src/              # Svelte frontend
│   │   └── src-tauri/        # Rust backend
│   ├── synth-eval/
│   ├── synth-ocpm/
│   └── synth-test-utils/
├── benches/                  # Benchmark suite
├── docs/                     # This documentation
└── tests/                    # Integration tests
```

## Crate Purposes

### Application Layer

| Crate | Purpose |
|-------|---------|
| **synth-cli** | Command-line interface |
| **synth-server** | REST/gRPC/WebSocket API |
| **synth-ui** | Desktop application |

### Processing Layer

| Crate | Purpose |
|-------|---------|
| **synth-runtime** | Orchestrates generation workflow |
| **synth-generators** | All data generation logic |
| **synth-graph** | Graph construction and export |

### Foundation Layer

| Crate | Purpose |
|-------|---------|
| **synth-core** | Domain models, traits, distributions |
| **synth-config** | Configuration schema and validation |
| **synth-output** | Output sinks (CSV, JSON) |

### Supporting Crates

| Crate | Purpose |
|-------|---------|
| **synth-eval** | Quality evaluation framework |
| **synth-ocpm** | OCEL 2.0 process mining |
| **synth-test-utils** | Test fixtures and assertions |

## Build Commands

```bash
# Build entire workspace
cargo build --release

# Build specific crate
cargo build -p synth-core
cargo build -p synth-generators

# Run tests
cargo test
cargo test -p synth-core

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
# synth-core
[features]
templates = ["serde_yaml"]

# synth-output
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
   synth-core = { path = "../synth-core" }
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
