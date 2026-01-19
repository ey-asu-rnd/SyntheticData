# synth-ui

Cross-platform desktop application for synthetic data generation.

## Overview

`synth-ui` provides a graphical interface for SyntheticData:

- **Visual Configuration**: Comprehensive UI for all configuration sections
- **Real-time Streaming**: Live generation viewer with WebSocket
- **Preset Management**: One-click industry preset application
- **Validation Feedback**: Real-time configuration validation

## Technology Stack

| Component | Technology |
|-----------|------------|
| Backend | Tauri 2.0 (Rust) |
| Frontend | SvelteKit + Svelte 5 |
| Styling | TailwindCSS |
| State | Svelte stores with runes |

## Prerequisites

### Linux

```bash
# Ubuntu/Debian
sudo apt install libgtk-3-dev libwebkit2gtk-4.1-dev libappindicator3-dev librsvg2-dev

# Fedora
sudo dnf install gtk3-devel webkit2gtk4.1-devel libappindicator-gtk3-devel librsvg2-devel

# Arch
sudo pacman -S webkit2gtk-4.1 base-devel curl wget file openssl appmenu-gtk-module gtk3 librsvg libvips
```

### macOS

No additional dependencies required (uses built-in WebKit).

### Windows

WebView2 runtime (usually pre-installed on Windows 10/11).

## Development

```bash
cd crates/synth-ui

# Install dependencies
npm install

# Development server (frontend only)
npm run dev

# Desktop app development
npm run tauri dev

# Production build
npm run build
npm run tauri build
```

## Configuration Sections

The UI provides 15+ configuration pages:

| Section | Description |
|---------|-------------|
| Global | Industry, dates, seed, performance |
| Transactions | Line items, amounts, sources |
| Master Data | Vendors, customers, materials |
| Document Flows | P2P, O2C configuration |
| Financial | Balance, subledger, FX, period close |
| Compliance | Fraud, controls, approval |
| Analytics | Graph export, anomaly, data quality |
| Output | Formats, compression |

## Project Structure

```
synth-ui/
├── src/                    # Svelte frontend
│   ├── routes/             # SvelteKit pages
│   │   ├── +page.svelte    # Dashboard
│   │   ├── config/         # Configuration pages (15+ sections)
│   │   └── stream/         # Generation streaming viewer
│   └── lib/
│       ├── components/     # Reusable UI components
│       ├── stores/         # Svelte stores
│       └── utils/          # Utilities
├── src-tauri/              # Rust backend
│   ├── src/
│   │   ├── lib.rs          # Tauri commands
│   │   └── main.rs         # App entry point
│   └── Cargo.toml
├── e2e/                    # Playwright E2E tests
└── package.json
```

## Server Connection

The UI connects to `synth-server` at `http://localhost:3000` by default:

```bash
# Start the server first
cargo run -p synth-server

# Then run the UI
npm run tauri dev
```

## Testing

```bash
# Unit tests (165 tests)
npm test

# E2E tests with Playwright
npx playwright test

# E2E with UI
npx playwright test --ui
```

## License

Apache-2.0 - See [LICENSE](../../LICENSE) for details.
