# Development Setup

Set up your local development environment for SyntheticData.

## Prerequisites

### Required

- **Rust**: 1.75 or later (install via [rustup](https://rustup.rs/))
- **Git**: For version control
- **Cargo**: Included with Rust

### Optional

- **Node.js 18+**: For desktop UI development (synth-ui)
- **Protocol Buffers**: For gRPC development
- **mdBook**: For documentation development

## Installation

### 1. Clone the Repository

```bash
git clone https://github.com/EY-ASU-RnD/SyntheticData.git
cd SyntheticData
```

### 2. Install Rust Toolchain

```bash
# Install rustup if not present
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Install stable toolchain
rustup install stable
rustup default stable

# Add useful components
rustup component add clippy rustfmt
```

### 3. Build the Project

```bash
# Debug build (faster compilation)
cargo build

# Release build (optimized)
cargo build --release

# Check without building
cargo check
```

### 4. Run Tests

```bash
# Run all tests
cargo test

# Run tests with output
cargo test -- --nocapture

# Run specific crate tests
cargo test -p synth-core
cargo test -p synth-generators
```

## IDE Setup

### VS Code

Recommended extensions:

```json
{
  "recommendations": [
    "rust-lang.rust-analyzer",
    "tamasfe.even-better-toml",
    "serayuzgur.crates",
    "vadimcn.vscode-lldb"
  ]
}
```

Settings for the project:

```json
{
  "rust-analyzer.cargo.features": "all",
  "rust-analyzer.checkOnSave.command": "clippy",
  "editor.formatOnSave": true
}
```

### JetBrains (RustRover/IntelliJ)

1. Install the Rust plugin
2. Open the project directory
3. Configure Cargo settings under Preferences > Languages & Frameworks > Rust

## Desktop UI Setup

For developing the Tauri/SvelteKit desktop UI:

```bash
# Navigate to UI crate
cd crates/synth-ui

# Install Node.js dependencies
npm install

# Run development server
npm run dev

# Run Tauri desktop app
npm run tauri dev

# Build production
npm run build
```

## Documentation Setup

For working on documentation:

```bash
# Install mdBook
cargo install mdbook

# Build documentation
cd docs
mdbook build

# Serve with live reload
mdbook serve --open

# Generate Rust API docs
cargo doc --workspace --no-deps --open
```

## Project Structure

```
SyntheticData/
├── crates/
│   ├── synth-cli/          # CLI binary
│   ├── synth-core/         # Core models and traits
│   ├── synth-config/       # Configuration schema
│   ├── synth-generators/   # Data generators
│   ├── synth-output/       # Output sinks
│   ├── synth-graph/        # Graph export
│   ├── synth-runtime/      # Orchestration
│   ├── synth-server/       # REST/gRPC server
│   ├── synth-ui/           # Desktop UI
│   └── synth-ocpm/         # OCEL 2.0 export
├── benches/                # Benchmarks
├── docs/                   # Documentation
├── configs/                # Example configs
└── templates/              # Data templates
```

## Environment Variables

| Variable | Description | Default |
|----------|-------------|---------|
| `RUST_LOG` | Log level (trace, debug, info, warn, error) | `info` |
| `SYNTH_CONFIG_PATH` | Default config search path | Current directory |
| `SYNTH_TEMPLATE_PATH` | Template files location | `./templates` |

## Debugging

### VS Code Launch Configuration

```json
{
  "version": "0.2.0",
  "configurations": [
    {
      "type": "lldb",
      "request": "launch",
      "name": "Debug CLI",
      "cargo": {
        "args": ["build", "--bin=synth-data", "--package=synth-cli"]
      },
      "args": ["generate", "--demo", "--output", "./output"],
      "cwd": "${workspaceFolder}"
    }
  ]
}
```

### Logging

Enable debug logging:

```bash
RUST_LOG=debug cargo run --release -- generate --demo --output ./output
```

Module-specific logging:

```bash
RUST_LOG=synth_generators=debug,synth_core=info cargo run ...
```

## Common Issues

### Build Failures

```bash
# Clean and rebuild
cargo clean
cargo build

# Update dependencies
cargo update
```

### Test Failures

```bash
# Run tests with backtrace
RUST_BACKTRACE=1 cargo test

# Run single test with output
cargo test test_name -- --nocapture
```

### Memory Issues

For large generation volumes, increase system limits:

```bash
# Linux: Increase open file limit
ulimit -n 65536

# Check memory usage during generation
/usr/bin/time -v synth-data generate --config config.yaml --output ./output
```

## Next Steps

- Review [Code Style](code-style.md) guidelines
- Read [Testing](testing.md) practices
- Learn the [Pull Request](pull-requests.md) process
