# Crate Reference

SyntheticData is organized as a Rust workspace with modular crates. This section provides detailed documentation for each crate.

## Workspace Structure

```
synth-cli          → Binary entry point (commands: generate, validate, init, info)
synth-server       → REST/gRPC/WebSocket server with auth, rate limiting
synth-ui           → Tauri/SvelteKit desktop UI
    ↓
synth-runtime      → Orchestration layer (GenerationOrchestrator coordinates workflow)
    ↓
synth-generators   → Data generators (JE, Document Flows, Subledgers, Anomalies)
    ↓
synth-graph        → Graph/network export (PyTorch Geometric, Neo4j, DGL)
    ↓
synth-config       → Configuration schema, validation, industry presets
    ↓
synth-core         → Domain models, traits, distributions, templates
    ↓
synth-output       → Output sinks (CSV, JSON, streaming)

synth-eval         → Evaluation framework (quality, coherence)
synth-ocpm         → Object-Centric Process Mining (OCEL 2.0)
synth-test-utils   → Testing utilities and fixtures
```

## Crate Categories

### Application Layer

| Crate | Description |
|-------|-------------|
| [synth-cli](synth-cli.md) | Command-line interface binary |
| [synth-server](synth-server.md) | REST/gRPC/WebSocket server |
| [synth-ui](synth-ui.md) | Desktop GUI application |

### Core Processing

| Crate | Description |
|-------|-------------|
| [synth-runtime](synth-runtime.md) | Generation orchestration |
| [synth-generators](synth-generators.md) | All data generators |
| [synth-graph](synth-graph.md) | ML graph export |

### Foundation

| Crate | Description |
|-------|-------------|
| [synth-core](synth-core.md) | Domain models and distributions |
| [synth-config](synth-config.md) | Configuration and validation |
| [synth-output](synth-output.md) | Output sinks |

### Supporting

| Crate | Description |
|-------|-------------|
| [synth-eval](synth-eval.md) | Quality evaluation |
| [synth-ocpm](synth-ocpm.md) | Process mining (OCEL 2.0) |
| [synth-test-utils](synth-test-utils.md) | Test utilities |

## Dependencies

The crates follow a strict dependency hierarchy:

1. **synth-core**: No internal dependencies (foundation)
2. **synth-config**: Depends on synth-core
3. **synth-output**: Depends on synth-core
4. **synth-generators**: Depends on synth-core, synth-config
5. **synth-graph**: Depends on synth-core, synth-generators
6. **synth-runtime**: Depends on synth-core, synth-config, synth-generators, synth-output, synth-graph
7. **synth-cli**: Depends on synth-runtime
8. **synth-server**: Depends on synth-runtime
9. **synth-ui**: Depends on synth-runtime (via Tauri)

## Building Individual Crates

```bash
# Build specific crate
cargo build -p synth-core
cargo build -p synth-generators

# Run tests for specific crate
cargo test -p synth-core
cargo test -p synth-generators

# Generate docs for specific crate
cargo doc -p synth-core --open
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
