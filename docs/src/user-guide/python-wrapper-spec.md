# Python Wrapper Specification (In-Memory Configs)

This document specifies a Python wrapper that makes DataSynth usable out of the box without requiring persisted configuration files. The wrapper focuses on rich, structured configuration objects and reusable configuration blueprints so developers can generate data entirely in memory while still benefiting from the full DataSynth configuration model.

## Goals

- **Zero-file setup**: Instantiate and run DataSynth without writing YAML/JSON to disk.
- **Rich configuration**: Offer a Pythonic API that maps cleanly to the full DataSynth configuration schema.
- **Blueprints**: Provide reusable, parameterized configuration templates for common scenarios.
- **Interoperable**: Allow optional export to YAML/JSON for debugging or CLI parity.
- **Composable**: Enable programmatic composition, overrides, and validation.

## Non-goals

- Replacing the DataSynth CLI or server API.
- Hiding the underlying schema; the wrapper should expose all configuration knobs.
- Managing persistence beyond optional explicit export helpers.

## Package layout

```text
packages/
  datasynth_py/
    __init__.py
    client.py             # entrypoint wrapper
    config/
      __init__.py
      models.py           # typed config objects
      blueprints.py       # blueprint registry + builders
      validation.py       # schema validation helpers
    runtime/
      __init__.py
      session.py          # in-memory execution
```

## Core API surface

### `DataSynth` entrypoint

```python
from datasynth_py import DataSynth

synth = DataSynth()
```

**Responsibilities**

- Provide a `generate()` method that accepts rich configuration objects.
- Provide `blueprints` registry access for common starting points.
- Manage execution in memory, including optional output sinks.

### `generate()` signature

```python
result = synth.generate(
    config=Config(...),
    output=OutputSpec(...),
    seed=42,
)
```

**Behavior**

- Validates configuration objects.
- Converts configuration to DataSynth schema (internal model or JSON/YAML in-memory string).
- Executes the generator and returns result handles (paths, in-memory tables, or streams).

### Optional output handling

`OutputSpec` can include:

- `format` (e.g., `parquet`, `csv`, `jsonl`)
- `sink` (`memory`, `temp_dir`, `path`)
- `compression` settings

When `sink="memory"`, the wrapper returns in-memory table objects (pandas DataFrames by default).

## Configuration model

### Typed configuration objects

Provide typed dataclasses/Pydantic models mirroring the DataSynth YAML schema:

- `GlobalSettings`
- `CompanySettings`
- `TransactionSettings`
- `MasterDataSettings`
- `ComplianceSettings`
- `OutputSettings`

Example:

```python
from datasynth_py.config import Config, GlobalSettings, CompanySettings

config = Config(
    global_settings=GlobalSettings(
        locale="en_US",
        fiscal_year_start="2024-01-01",
        periods=12,
    ),
    companies=CompanySettings(count=5, industry="retail"),
)
```

### Overrides and layering

Allow configuration layering to support incremental overrides:

```python
config = base_config.override(
    companies={"count": 10},
    output={"format": "parquet"},
)
```

The wrapper merges overrides deeply, preserving nested settings.

## Blueprints

Blueprints provide preconfigured setups with parameters. The wrapper ships with a registry:

```python
from datasynth_py import blueprints

config = blueprints.retail_small(companies=3, transactions=5000)
```

### Blueprint characteristics

- **Parameterized**: Each blueprint accepts keyword overrides for key metrics.
- **Composable**: A blueprint can extend or wrap another blueprint.
- **Discoverable**: Registry lists available blueprints and metadata.

```python
blueprints.list()
# ["retail_small", "banking_medium", "saas_subscription", ...]
```

## Execution model

The wrapper runs the Rust engine in-process via FFI or uses the DataSynth runtime API:

- **In-memory config**: converted to serialized config strings without writing to disk.
- **Transient workspace**: uses a temporary directory only if required by runtime internals.
- **Deterministic runs**: `seed` controls RNG.

### Streaming generation

The wrapper exposes a streaming session that connects to `datasynth-server` over WebSockets while using REST endpoints to start, pause, resume, and stop streams.

## Examples

### Example 1: Minimal generation in memory

```python
from datasynth_py import DataSynth
from datasynth_py.config import Config, GlobalSettings, CompanySettings

config = Config(
    global_settings=GlobalSettings(locale="en_US", fiscal_year_start="2024-01-01"),
    companies=CompanySettings(count=2),
)

synth = DataSynth()
result = synth.generate(config=config, output={"format": "csv", "sink": "memory"})

# result.tables -> dict[str, pandas.DataFrame]
print(result.tables["transactions"].head())
```

### Example 2: Use a blueprint with overrides

```python
from datasynth_py import DataSynth, blueprints

synth = DataSynth()
config = blueprints.retail_small(companies=4, transactions=15000)

result = synth.generate(
    config=config,
    output={"format": "parquet", "sink": "temp_dir"},
    seed=7,
)

print(result.output_dir)
```

### Example 3: Layering overrides for a custom scenario

```python
from datasynth_py import DataSynth
from datasynth_py.config import Config, GlobalSettings, TransactionSettings

base = Config(global_settings=GlobalSettings(locale="en_GB"))
custom = base.override(
    transactions=TransactionSettings(
        count=25000,
        currency="GBP",
        anomaly_rate=0.02,
    )
)

synth = DataSynth()
result = synth.generate(config=custom, output={"format": "jsonl", "sink": "memory"})
```

### Example 4: Export configuration for debugging

```python
from datasynth_py import DataSynth
from datasynth_py.config import Config

synth = DataSynth()
config = Config(...)

print(config.to_yaml())
print(config.to_json())
```

### Example 5: Streaming events

```python
import asyncio

from datasynth_py import DataSynth
from datasynth_py.config import blueprints


async def main() -> None:
    synth = DataSynth(server_url="http://localhost:3000")
    config = blueprints.retail_small(companies=2, transactions=5000)
    session = synth.stream(config=config, events_per_second=50)

    async for event in session.events():
        print(event)
        break


asyncio.run(main())
```

## Decisions

- **In-memory table format**: pandas DataFrames are the default return type for memory sinks.
- **Validation errors**: configuration validation raises `ConfigValidationError` with structured error details.
