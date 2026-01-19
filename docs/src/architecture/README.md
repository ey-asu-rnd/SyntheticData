# Architecture

SyntheticData is designed as a modular, high-performance data generation system.

## Overview

```
┌─────────────────────────────────────────────────────────────────────┐
│                         Application Layer                            │
│   synth-cli │ synth-server │ synth-ui                               │
├─────────────────────────────────────────────────────────────────────┤
│                        Orchestration Layer                           │
│                         synth-runtime                                │
├─────────────────────────────────────────────────────────────────────┤
│                        Generation Layer                              │
│   synth-generators │ synth-graph                                    │
├─────────────────────────────────────────────────────────────────────┤
│                        Foundation Layer                              │
│   synth-core │ synth-config │ synth-output                          │
└─────────────────────────────────────────────────────────────────────┘
```

## Key Characteristics

| Characteristic | Description |
|----------------|-------------|
| **Modular** | 12 independent crates with clear boundaries |
| **Layered** | Strict dependency hierarchy prevents cycles |
| **High-Performance** | Parallel execution, memory-efficient streaming |
| **Deterministic** | Seeded RNG for reproducible output |
| **Type-Safe** | Rust's type system ensures correctness |

## Architecture Sections

| Section | Description |
|---------|-------------|
| [Workspace Layout](workspace-layout.md) | Crate organization and dependencies |
| [Domain Models](domain-models.md) | Core data structures |
| [Data Flow](data-flow.md) | How data moves through the system |
| [Generation Pipeline](generation-pipeline.md) | Step-by-step generation process |
| [Memory Management](memory-management.md) | Memory tracking and limits |
| [Design Decisions](design-decisions.md) | Key architectural choices |

## Design Principles

### Separation of Concerns

Each crate has a single responsibility:
- `synth-core`: Domain models and distributions
- `synth-config`: Configuration and validation
- `synth-generators`: Data generation logic
- `synth-output`: File writing
- `synth-runtime`: Orchestration

### Dependency Inversion

Core components define traits, implementations provided by higher layers:

```rust
// synth-core defines the trait
pub trait Generator<T> {
    fn generate_batch(&mut self, count: usize) -> Result<Vec<T>>;
}

// synth-generators implements it
impl Generator<JournalEntry> for JournalEntryGenerator {
    fn generate_batch(&mut self, count: usize) -> Result<Vec<JournalEntry>> {
        // Implementation
    }
}
```

### Configuration-Driven

All behavior controlled by configuration:

```yaml
transactions:
  target_count: 100000
  benford:
    enabled: true
```

### Memory Safety

Rust's ownership system prevents:
- Data races in parallel generation
- Memory leaks
- Buffer overflows

## Component Interactions

```
                    ┌─────────────┐
                    │   Config    │
                    └──────┬──────┘
                           │
        ┌──────────────────┼──────────────────┐
        │                  │                  │
        ▼                  ▼                  ▼
┌──────────────┐  ┌──────────────┐  ┌──────────────┐
│  JE Generator│  │ Doc Generator│  │ Master Data  │
└──────┬───────┘  └──────┬───────┘  └──────┬───────┘
       │                 │                 │
       └─────────────────┼─────────────────┘
                         │
                         ▼
                ┌──────────────┐
                │ Orchestrator │
                └──────┬───────┘
                       │
        ┌──────────────┼──────────────┐
        │              │              │
        ▼              ▼              ▼
   ┌─────────┐   ┌─────────┐   ┌─────────┐
   │   CSV   │   │  Graph  │   │  JSON   │
   └─────────┘   └─────────┘   └─────────┘
```

## Performance Architecture

### Parallel Execution

```rust
// Thread pool distributes work
let entries: Vec<JournalEntry> = (0..num_threads)
    .into_par_iter()
    .flat_map(|thread_id| {
        let mut gen = generator_for_thread(thread_id);
        gen.generate_batch(batch_size)
    })
    .collect();
```

### Streaming Output

```rust
// Memory-efficient streaming
for entry in generator.generate_stream() {
    sink.write(&entry)?;
}
```

### Memory Guards

```rust
// Memory limits enforced
let guard = MemoryGuard::new(config);
while !guard.check().exceeds_hard_limit {
    generate_batch();
}
```

## Extension Points

### Custom Generators

Implement the `Generator` trait:

```rust
impl Generator<CustomType> for CustomGenerator {
    fn generate_batch(&mut self, count: usize) -> Result<Vec<CustomType>> {
        // Custom logic
    }
}
```

### Custom Output Sinks

Implement the `Sink` trait:

```rust
impl Sink<JournalEntry> for CustomSink {
    fn write(&mut self, entry: &JournalEntry) -> Result<()> {
        // Custom output logic
    }
}
```

### Custom Distributions

Create specialized samplers:

```rust
impl AmountSampler for CustomAmountSampler {
    fn sample(&mut self) -> Decimal {
        // Custom distribution
    }
}
```

## See Also

- [Crate Reference](../crates/README.md)
- [synth-core](../crates/synth-core.md)
- [synth-runtime](../crates/synth-runtime.md)
