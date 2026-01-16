# synth-ui

Desktop UI for the Synthetic Data Generator built with Tauri 2.0 + SvelteKit.

## Features

- Real-time metrics dashboard with live updates
- Configuration panel for generation parameters
- Stream control (start/stop/pause)
- WebSocket event viewer
- Swiss-design minimalist interface (Inter font, grid-based layout, high contrast)

## Prerequisites

### Linux

Install GTK and WebKit development libraries:

```bash
# Ubuntu/Debian
sudo apt install libgtk-3-dev libwebkit2gtk-4.1-dev libappindicator3-dev librsvg2-dev

# Fedora
sudo dnf install gtk3-devel webkit2gtk4.1-devel libappindicator-gtk3-devel librsvg2-devel

# Arch
sudo pacman -S webkit2gtk-4.1 base-devel curl wget file openssl appmenu-gtk-module gtk3 librsvg libvips
```

### macOS

No additional dependencies required (uses WebKit which is built into macOS).

### Windows

Install the WebView2 runtime (usually already installed on Windows 10/11).

## Building

### Frontend (Svelte)

```bash
cd crates/synth-ui
npm install
npm run build
```

### Backend (Tauri/Rust)

First, add synth-ui to the workspace by uncommenting it in the root `Cargo.toml`:

```toml
[workspace]
members = [
    # ... other crates
    "crates/synth-ui/src-tauri",
]
```

Then build:

```bash
cargo build -p synth-ui --release
```

### Development

Run both frontend and backend in development mode:

```bash
cd crates/synth-ui
npm run tauri dev
```

## Architecture

```
synth-ui/
├── src/                    # Svelte frontend
│   ├── app.css            # Swiss-design CSS system
│   ├── app.html           # HTML template
│   ├── routes/            # SvelteKit pages
│   │   ├── +layout.svelte # App layout with navigation
│   │   ├── +page.svelte   # Dashboard (main page)
│   │   ├── config/        # Configuration page
│   │   └── stream/        # Stream viewer page
│   └── lib/               # Svelte components
│       ├── StatusBar.svelte
│       ├── MetricsPanel.svelte
│       └── ControlPanel.svelte
├── src-tauri/             # Rust backend (Tauri)
│   ├── src/
│   │   ├── lib.rs         # Tauri commands
│   │   └── main.rs        # App entry point
│   └── Cargo.toml
├── package.json           # Node dependencies
├── svelte.config.js       # SvelteKit config
└── vite.config.ts         # Vite config
```

## Server Connection

The UI connects to the synth-server REST API at `http://localhost:3000` by default. Make sure to start the server first:

```bash
cargo run -p synth-server
```

## Pages

### Dashboard (`/`)

- Server connection status
- Real-time metrics (entries generated, anomalies, rate)
- Stream control buttons
- Bulk generation form

### Configuration (`/config`)

- Global settings (industry, date range, seed)
- Chart of Accounts complexity
- Fraud/anomaly injection settings
- Company management

### Stream (`/stream`)

- WebSocket connection to event stream
- Real-time event table
- Auto-scroll and event filtering
