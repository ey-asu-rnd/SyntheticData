# datasynth-runtime

Runtime orchestration, parallel execution, and memory management.

## Overview

`datasynth-runtime` provides the execution layer for SyntheticData:

- **GenerationOrchestrator**: Coordinates the complete generation workflow
- **Parallel Execution**: Multi-threaded generation with Rayon
- **Memory Management**: Integration with memory guard for OOM prevention
- **Progress Tracking**: Real-time progress reporting with pause/resume

## Key Components

| Component | Description |
|-----------|-------------|
| `GenerationOrchestrator` | Main workflow coordinator |
| `EnhancedOrchestrator` | Extended orchestrator with all enterprise features |
| `ParallelExecutor` | Thread pool management |
| `ProgressTracker` | Progress bars and status reporting |

## Generation Workflow

1. **Initialize**: Load configuration, validate settings
2. **Master Data**: Generate vendors, customers, materials, assets
3. **Opening Balances**: Create coherent opening balance sheet
4. **Transactions**: Generate journal entries with document flows
5. **Period Close**: Run monthly/quarterly/annual close processes
6. **Anomalies**: Inject configured anomalies and data quality issues
7. **Export**: Write outputs and generate ML labels

## Usage

```rust
use datasynth_runtime::GenerationOrchestrator;

let orchestrator = GenerationOrchestrator::new(config)?;

// Full generation
orchestrator.run()?;

// With progress callback
orchestrator.run_with_progress(|progress| {
    println!("Generated: {}/{}", progress.completed, progress.total);
})?;
```

## Pause/Resume

On Unix systems, send `SIGUSR1` to toggle pause:

```bash
kill -USR1 $(pgrep datasynth-data)
```

## License

Apache-2.0 - See [LICENSE](../../LICENSE) for details.
