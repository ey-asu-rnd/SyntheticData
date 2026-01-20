# Crate Reference

SyntheticData is organized as a Rust workspace with modular crates. This section provides detailed documentation for each crate.

## Workspace Structure

```
datasynth-cli          → Binary entry point (commands: generate, validate, init, info)
datasynth-server       → REST/gRPC/WebSocket server with auth, rate limiting
datasynth-ui           → Tauri/SvelteKit desktop UI
    ↓
datasynth-runtime      → Orchestration layer (GenerationOrchestrator coordinates workflow)
    ↓
datasynth-generators   → Data generators (JE, Document Flows, Subledgers, Anomalies)
    ↓
datasynth-graph        → Graph/network export (PyTorch Geometric, Neo4j, DGL)
    ↓
datasynth-config       → Configuration schema, validation, industry presets
    ↓
datasynth-core         → Domain models, traits, distributions, templates
    ↓
datasynth-output       → Output sinks (CSV, JSON, streaming)

datasynth-eval         → Evaluation framework (quality, coherence)
datasynth-ocpm         → Object-Centric Process Mining (OCEL 2.0)
datasynth-test-utils   → Testing utilities and fixtures
```

## Crate Categories

### Application Layer

| Crate | Description |
|-------|-------------|
| [datasynth-cli](datasynth-cli.md) | Command-line interface binary |
| [datasynth-server](datasynth-server.md) | REST/gRPC/WebSocket server |
| [datasynth-ui](datasynth-ui.md) | Desktop GUI application |

### Core Processing

| Crate | Description |
|-------|-------------|
| [datasynth-runtime](datasynth-runtime.md) | Generation orchestration |
| [datasynth-generators](datasynth-generators.md) | All data generators |
| [datasynth-graph](datasynth-graph.md) | ML graph export |

### Foundation

| Crate | Description |
|-------|-------------|
| [datasynth-core](datasynth-core.md) | Domain models and distributions |
| [datasynth-config](datasynth-config.md) | Configuration and validation |
| [datasynth-output](datasynth-output.md) | Output sinks |

### Supporting

| Crate | Description |
|-------|-------------|
| [datasynth-eval](datasynth-eval.md) | Quality evaluation |
| [datasynth-ocpm](datasynth-ocpm.md) | Process mining (OCEL 2.0) |
| [datasynth-test-utils](datasynth-test-utils.md) | Test utilities |

## Dependencies

The crates follow a strict dependency hierarchy:

1. **datasynth-core**: No internal dependencies (foundation)
2. **datasynth-config**: Depends on datasynth-core
3. **datasynth-output**: Depends on datasynth-core
4. **datasynth-generators**: Depends on datasynth-core, datasynth-config
5. **datasynth-graph**: Depends on datasynth-core, datasynth-generators
6. **datasynth-runtime**: Depends on datasynth-core, datasynth-config, datasynth-generators, datasynth-output, datasynth-graph
7. **datasynth-cli**: Depends on datasynth-runtime
8. **datasynth-server**: Depends on datasynth-runtime
9. **datasynth-ui**: Depends on datasynth-runtime (via Tauri)

## Building Individual Crates

```bash
# Build specific crate
cargo build -p datasynth-core
cargo build -p datasynth-generators

# Run tests for specific crate
cargo test -p datasynth-core
cargo test -p datasynth-generators

# Generate docs for specific crate
cargo doc -p datasynth-core --open
```

## API Documentation

For detailed Rust API documentation, generate and view rustdoc:

```bash
cargo doc --workspace --no-deps --open
```

## See Also

- [Architecture Overview](../architecture/README.md)
- [Workspace Layout](../architecture/workspace-layout.md)
- [Domain Models](../architecture/domain-models.md)
