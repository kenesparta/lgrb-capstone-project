# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

LGRB (Let's Get Rusty) Capstone Project - A multi-crate Rust IoT system connecting a BBC micro:bit v2 to a real-time web
dashboard via Bluetooth Low Energy.

**Architecture:**

```
micro:bit v2 (lgrcp-embed) → BLE → ble-listener → HTTP POST → ws-server → WebSocket → Browser
```

## Build Commands

### WebSocket Server (ws-server)

```bash
cargo run -p ws-server              # Debug, listens on 0.0.0.0:3000
cargo run -p ws-server --release    # Release
```

### BLE Listener (ble-listener)

```bash
cargo run -p ble-listener           # Requires Bluetooth adapter + device advertising "LGR-BLE"
cargo run -p ble-listener --release
```

### Embedded Firmware (lgrcp-embed)

```bash
# Install toolchain first
cd lgrcp-embed && make prepare

# Build
cargo build --release --target thumbv7em-none-eabihf -p lgrcp-embed

# Flash to micro:bit v2
cargo flash --chip nRF52833_xxAA --release --target thumbv7em-none-eabihf -p lgrcp-embed
```

### Cross-Compilation (Raspberry Pi aarch64)

```bash
rustup target add aarch64-unknown-linux-gnu
make build-ws-server      # Build ws-server for aarch64
make build-ble-listener   # Build ble-listener for aarch64
make run                  # Start both services in background
make stop                 # Stop services
make logs-ws              # Tail ws-server logs
make logs-ble             # Tail ble-listener logs
```

### Test API Without Hardware

```bash
# Terminal 1: Start server
cargo run -p ws-server

# Terminal 2: Send mock event
curl -X POST http://localhost:3000/api/button \
  -H 'Content-Type: application/json' \
  -d '{"button":"A","state":"pressed","timestamp":1728011234}'

# Open http://localhost:3000/ to see dashboard
```

## Crate Architecture

| Crate            | Purpose                              | Runtime | Key Dependencies                             |
|------------------|--------------------------------------|---------|----------------------------------------------|
| **lgrcp-embed**  | Firmware for micro:bit v2 (nRF52833) | Embassy | embassy-*, trouble-host, microbit-bsp, defmt |
| **ble-listener** | BLE→HTTP bridge on host              | Tokio   | btleplug, reqwest, serde                     |
| **ws-server**    | WebSocket server + dashboard         | Tokio   | axum, tower-http, serde                      |

## Key Technical Details

### Event Data Schema (JSON)

```json
{
  "button": "A|B|ANY",
  "state": "PRESSED|RELEASED",
  "timestamp": 1728011234
}
```

### BLE Button Notification Mapping

- `1` → Button A pressed
- `2` → Button B pressed
- `0` → Button released

### GATT Services (lgrcp-embed)

- Battery Service: `0000180F-0000-1000-8000-00805F9B34FB`
- Button Service: `EF680800-9B35-4933-9B10-52FFA9740042` (custom)

### Hardcoded Configuration

- Device name: `"LGR-BLE"` (lgrcp-embed & ble-listener)
- Server address: `0.0.0.0:3000` (ws-server)
- API endpoint: `http://0.0.0.0:3000/api/button` (ble-listener)

## API Endpoints (ws-server)

| Route         | Method | Description                                      |
|---------------|--------|--------------------------------------------------|
| `/`           | GET    | Dashboard HTML                                   |
| `/ws`         | GET    | WebSocket (broadcast-only)                       |
| `/api/button` | POST   | Ingest ButtonEvent, broadcasts to all WS clients |

## Debugging

### Embedded (lgrcp-embed)

```bash
# Uses defmt-rtt logging via probe-rs
probe-rs gdb --chip nRF52833_xxAA target/thumbv7em-none-eabihf/release/lgrcp-embed
```

### Host Components

Both ble-listener and ws-server use `println!` for logging. When started via Makefile, logs go to `logs/` directory.
