# ble-listener

A small Rust utility that connects to a Bluetooth Low Energy (BLE) device (e.g., BBC micro:bit) named "LGR-BLE", subscribes to its notify characteristics, and forwards button events to the local web server via HTTP. It also attempts to read the device battery level if available.

By default, events are POSTed as JSON to:
- http://0.0.0.0:3000/api/button

This pairs with the ws-server package, which broadcasts the events to web clients and serves a dashboard.

## What it does
- Scans for nearby BLE devices and selects the one whose advertised name is `LGR-BLE`.
- Connects and discovers services/characteristics.
- Subscribes to notify characteristics and translates notification bytes into button events:
  - 1 → Button A pressed
  - 2 → Button B pressed
  - 0 → Button released
- Sends each event to the web server as:
  ```json
  { "button": "A|B|ANY", "state": "PRESSED|RELEASED", "timestamp": 1728000000000 }
  ```
- Tries to read the standard Battery Service (0x180F) and print the battery level.

## Tech stack
- Rust (Tokio async)
- btleplug (cross-platform BLE)
- reqwest (HTTP client)
- serde (serialization)

## Prerequisites
- Rust and Cargo installed
- A working Bluetooth adapter
- A BLE device advertising as `LGR-BLE` (e.g., micro:bit with custom firmware)
- The companion web server running (recommended): `ws-server` listening on port 3000

## Build and run
From repository root using workspace package selection:
- Debug: `cargo run -p ble-listener`
- Release: `cargo run -p ble-listener --release`

Or from the ble-listener directory:
- Debug: `cargo run`
- Release: `cargo run --release`

With the provided Makefile (from repo root):
- Build both services for aarch64: `make build`
- Start both services in background: `make run`
- Tail logs: `make logs-all` (or check `logs/ble-listener.log`)
- Stop both: `make stop`

Note: The Makefile defaults to the `aarch64-unknown-linux-gnu` target. Adjust if your environment differs.

## Configuration
Configuration is currently done by editing constants in `src/main.rs`:
- Device name:
  ```rust
  const DEVICE_NAME: &str = "LGR-BLE";
  ```
- Web server endpoint (HTTP POST target):
  ```rust
  const WEB_SERVER_URL: &str = "http://0.0.0.0:3000/api/button";
  ```
- Battery service/characteristic UUIDs (if your device differs):
  ```rust
  const BATTERY_SERVICE_UUID: &str = "0000180F-0000-1000-8000-00805F9B34FB";
  const BATTERY_LEVEL_UUID: &str = "00002A19-0000-1000-8000-00805F9B34FB";
  ```

If you need runtime configurability (env vars/CLI flags), consider refactoring to read these from the environment or arguments.

## Example output
```
🚀 Starting BLE Button Tester with WebSocket
==================================================
Using adapter: hci0
🔍 Scanning for LGR-BLE a device...
Found 3 BLE devices:
  - LGR-BLE (AA:BB:CC:DD:EE:FF)
✅ Found device: LGR-BLE (AA:BB:CC:DD:EE:FF)
🔗 Connecting to device...
🔗 Connected: true
📋 Available services (3):
  🔹 Service 0000180F-0000-1000-8000-00805F9B34FB
    └─ Characteristic 00002A19-0000-1000-8000-00805F9B34FB (READ)
🔋 Battery Level: 92%
🎮 Ready! Press buttons A or B on your micro:bit...
📡 Events will be sent to the web browser at http://127.0.0.1:3000
🔴 Button A (LEFT) PRESSED
📤 Sent A PRESSED to web server
⚪ Button RELEASED
📤 Sent ANY RELEASED to web server
```

## OS-specific notes
- Linux:
  - You may need appropriate permissions to access BLE without sudo. Consider adding your user to the `bluetooth` group or configuring udev rules.
  - On some systems, running with `sudo` is the quickest way to test.
- macOS:
  - Grant Bluetooth permissions to your terminal/IDE.
  - Pairing might be required depending on your device/firmware.
- Windows:
  - Ensure Bluetooth is enabled and supported by btleplug on your Windows version.

## Troubleshooting
- No devices found:
  - Ensure the micro:bit (or other BLE device) is powered and advertising as `LGR-BLE`.
  - Increase scan time if necessary (hardcoded 10s delay after `start_scan`).
  - Verify your adapter with other BLE tools (e.g., `bluetoothctl` on Linux).
- Connected but no events:
  - The code subscribes to any characteristic with NOTIFY. Ensure your firmware emits notifications where `data[0]` aligns with 0/1/2 mapping.
  - Check that the server at `WEB_SERVER_URL` is reachable (e.g., `curl http://0.0.0.0:3000/`).
- HTTP errors (4xx/5xx):
  - Confirm `ws-server` is running and listening on `0.0.0.0:3000`.
  - Validate JSON schema expected by `/api/button` (see ws-server README).
- Permission errors:
  - See OS-specific notes above for Bluetooth permissions.

## Relation to ws-server
- ws-server exposes:
  - POST `/api/button` for ingesting events (used by this app)
  - GET `/ws` for broadcasting events to WebSocket clients
  - GET `/` serves a simple dashboard to visualize button events

Open http://localhost:3000/ in your browser to see events arriving in real time after running both `ws-server` and `ble-listener`.

## Development notes
- Main entry points:
  - `find_device(adapter)` → scan/select `LGR-BLE`
  - `connect_and_listen(peripheral, client)` → subscribe to NOTIFY, read battery, process notifications
  - `handle_button_notification(data, client)` → map bytes to A/B/RELEASED and POST
- Event struct (`ButtonEvent`):
  ```rust
  #[derive(Clone, Debug, Serialize, Deserialize)]
  pub struct ButtonEvent {
      pub button: String,
      pub state: String,
      pub timestamp: u64,
  }
  ```

## License
This project is part of the lgrb-capstone-project. See repository-level licensing, if provided.
