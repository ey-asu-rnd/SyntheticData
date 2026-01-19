# Memory Management

How SyntheticData manages memory during generation.

## Overview

Large-scale data generation can consume significant memory. SyntheticData provides:

- **Memory Guard**: Cross-platform memory tracking
- **Soft/Hard Limits**: Configurable thresholds
- **Streaming Output**: Reduce memory pressure
- **Growth Rate Monitoring**: Detect runaway consumption

## Memory Guard

The `MemoryGuard` component tracks process memory usage:

```rust
pub struct MemoryGuard {
    config: MemoryGuardConfig,
    last_check: Instant,
    last_usage: u64,
}

pub struct MemoryGuardConfig {
    pub soft_limit: u64,           // Pause/slow threshold
    pub hard_limit: u64,           // Stop threshold
    pub check_interval_ms: u64,    // How often to check
    pub growth_rate_threshold: f64, // Bytes/sec warning
}

pub struct MemoryStatus {
    pub current_usage: u64,
    pub exceeds_soft_limit: bool,
    pub exceeds_hard_limit: bool,
    pub growth_rate: f64,
}
```

## Platform Support

| Platform | Method |
|----------|--------|
| Linux | `/proc/self/statm` |
| macOS | `ps` command |
| Windows | Stubbed (returns 0) |

### Linux Implementation

```rust
#[cfg(target_os = "linux")]
fn get_memory_usage() -> u64 {
    let statm = fs::read_to_string("/proc/self/statm").ok()?;
    let rss_pages: u64 = statm.split_whitespace().nth(1)?.parse().ok()?;
    let page_size = unsafe { libc::sysconf(libc::_SC_PAGESIZE) } as u64;
    rss_pages * page_size
}
```

### macOS Implementation

```rust
#[cfg(target_os = "macos")]
fn get_memory_usage() -> u64 {
    let output = Command::new("ps")
        .args(["-o", "rss=", "-p", &std::process::id().to_string()])
        .output()
        .ok()?;
    let rss_kb: u64 = String::from_utf8_lossy(&output.stdout)
        .trim()
        .parse()
        .ok()?;
    rss_kb * 1024
}
```

## Configuration

```yaml
global:
  memory_limit: 2147483648    # 2 GB hard limit
```

Or programmatically:

```rust
let config = MemoryGuardConfig {
    soft_limit: 1024 * 1024 * 1024,      // 1 GB
    hard_limit: 2 * 1024 * 1024 * 1024,  // 2 GB
    check_interval_ms: 1000,              // Check every second
    growth_rate_threshold: 100_000_000.0, // 100 MB/sec
};
```

## Usage in Generation

```rust
pub fn generate_with_memory_guard(&mut self) -> Result<()> {
    let guard = MemoryGuard::new(self.memory_config);

    loop {
        // Check memory
        let status = guard.check();

        if status.exceeds_hard_limit {
            // Stop generation
            return Err(Error::MemoryExceeded);
        }

        if status.exceeds_soft_limit {
            // Flush output and trigger GC
            self.sink.flush()?;
            self.state.clear_caches();
            continue;
        }

        if status.growth_rate > guard.config.growth_rate_threshold {
            // Slow down
            thread::sleep(Duration::from_millis(100));
        }

        // Generate batch
        let batch = self.generator.generate_batch(BATCH_SIZE)?;
        self.process_batch(batch)?;

        if self.is_complete() {
            break;
        }
    }

    Ok(())
}
```

## Memory Estimation

Estimate memory requirements before generation:

```rust
pub fn estimate_memory(config: &Config) -> MemoryEstimate {
    let entry_size = 512;  // Approximate bytes per entry
    let master_data_size = config.estimate_master_data_size();

    let peak = master_data_size
        + (config.transactions.target_count as u64 * entry_size);

    let streaming_peak = master_data_size
        + (BATCH_SIZE as u64 * entry_size);

    MemoryEstimate {
        batch_peak: peak,
        streaming_peak,
        recommended_limit: peak * 2,
    }
}
```

## Memory-Efficient Patterns

### Streaming Output

Write as you generate instead of accumulating:

```rust
// Memory-efficient
for entry in generator.generate_stream() {
    sink.write(&entry?)?;
}

// Memory-intensive (avoid for large volumes)
let all_entries = generator.generate_batch(1_000_000)?;
sink.write_batch(&all_entries)?;
```

### Batch Processing with Flush

```rust
const BATCH_SIZE: usize = 10_000;

let mut buffer = Vec::with_capacity(BATCH_SIZE);

for entry in generator.generate_stream() {
    buffer.push(entry?);

    if buffer.len() >= BATCH_SIZE {
        sink.write_batch(&buffer)?;
        buffer.clear();
    }
}

// Final flush
if !buffer.is_empty() {
    sink.write_batch(&buffer)?;
}
```

### Lazy Loading

Load master data on demand:

```rust
pub struct LazyRegistry {
    vendors: OnceCell<Vec<Vendor>>,
    vendor_loader: Box<dyn Fn() -> Vec<Vendor>>,
}

impl LazyRegistry {
    pub fn vendors(&self) -> &[Vendor] {
        self.vendors.get_or_init(|| (self.vendor_loader)())
    }
}
```

## Memory Limits by Component

Estimated memory usage:

| Component | Size (per item) | For 1M entries |
|-----------|-----------------|----------------|
| JournalEntry | ~512 bytes | ~500 MB |
| Document | ~1 KB | ~1 GB |
| Graph Node | ~128 bytes | ~128 MB |
| Graph Edge | ~64 bytes | ~64 MB |

## Monitoring

### Progress with Memory

```rust
orchestrator.run_with_progress(|progress| {
    let memory_mb = guard.check().current_usage / 1_000_000;
    println!(
        "[{:.1}%] {} entries | {} MB",
        progress.percent,
        progress.current,
        memory_mb
    );
});
```

### Server Endpoint

```bash
curl http://localhost:3000/health
```

```json
{
  "status": "healthy",
  "memory_usage_mb": 512,
  "memory_limit_mb": 2048,
  "memory_percent": 25.0
}
```

## Troubleshooting

### Out of Memory

**Symptoms:** Process killed, "out of memory" error

**Solutions:**
1. Reduce `target_count`
2. Enable streaming output
3. Increase system memory
4. Set appropriate `memory_limit`

### Slow Generation

**Symptoms:** Generation slows over time

**Cause:** Memory pressure triggering slowdown

**Solutions:**
1. Increase soft limit
2. Reduce batch size
3. Enable more aggressive flushing

### Memory Not Freed

**Symptoms:** Memory stays high after generation

**Cause:** Data retained in caches

**Solution:** Explicitly clear state:

```rust
orchestrator.clear_caches();
```

## See Also

- [Performance Tuning](../advanced/performance.md)
- [synth-runtime](../crates/synth-runtime.md)
- [Configuration](../configuration/global-settings.md)
