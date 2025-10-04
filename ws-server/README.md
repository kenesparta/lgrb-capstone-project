# ws-server

A lightweight Axum-based WebSocket server that broadcasts button events from devices (e.g., micro:bit) to connected web clients in real time. It also serves a simple dashboard (index.html) to visualize events and provides an HTTP API to inject events.

## Features
- WebSocket endpoint that pushes ButtonEvent messages to all connected clients
- HTTP endpoint to publish button events (JSON)
- Serves a built-in dashboard at /
- Static file serving for /pkg (if present)
- Tokio broadcast channel fan-out for efficient multi-client delivery

## Tech stack
- Rust (Edition 2021)
- Axum 0.8 (Web framework + WebSocket)
- Tokio 1.x (Async runtime)
- tower-http (Static files)
- Serde/serde_json (Serialization)

## Run locally
Prerequisites: Rust and Cargo installed.

- From repo root using workspace package selection:
  - Debug: `cargo run -p ws-server`
  - Release: `cargo run -p ws-server --release`

- Or from the ws-server directory:
  - Debug: `cargo run`
  - Release: `cargo run --release`

The server listens on 0.0.0.0:3000.

Open the dashboard:
- http://localhost:3000/

## Using the Makefile (cross-compile + background services)
The repository provides a Makefile that builds for aarch64-unknown-linux-gnu and can run ws-server and ble-listener in the background with logs.

Common targets (run from repo root):
- Build only ws-server: `make build-ws-server`
- Start both services in background: `make run`
  - Logs: `make logs-ws` or check `logs/ws-server.log`
- Stop both services: `make stop`
- Tail all logs: `make logs-all`

Note: `make run` starts the cross-compiled binaries under `target/aarch64-unknown-linux-gnu/release/`. Ensure your environment supports that target or adjust the Makefile/commands to your platform.

## HTTP API
- POST /api/button
  - Content-Type: application/json
  - Body schema (ButtonEvent):
    ```json
    {
      "button": "A | B | <other>",
      "state": "pressed | released | <other>",
      "timestamp": 1699999999
    }
    ```
  - Response: `200 OK` with body `"Event received"`

Example cURL:
```bash
curl -X POST http://localhost:3000/api/button \
  -H 'Content-Type: application/json' \
  -d '{"button":"A","state":"pressed","timestamp":1728011234}'
```

## WebSocket API
- GET /ws (WebSocket upgrade)
- Outgoing message format (JSON-encoded ButtonEvent):
  ```json
  {
    "button": "A",
    "state": "pressed",
    "timestamp": 1728011234
  }
  ```
- Incoming messages from clients are currently ignored (except handling Close frames). The server is broadcast-only.

Quick JS example:
```html
<script>
const ws = new WebSocket("ws://localhost:3000/ws");
ws.onopen = () => console.log("connected");
ws.onmessage = (e) => {
  try { console.log("event:", JSON.parse(e.data)); }
  catch (_) { console.log("raw:", e.data); }
};
ws.onclose = () => console.log("disconnected");
</script>
```

## Routes overview
- GET `/` → Serves the included dashboard (index.html)
- GET `/ws` → WebSocket endpoint broadcasting ButtonEvent
- POST `/api/button` → Publish a ButtonEvent to all WS clients
- Static `/pkg/*` → Served from local `pkg/` directory if present

## Configuration
- Address and port are currently hardcoded to `0.0.0.0:3000` in `src/main.rs`.
- If you need configurability (env vars/CLI), consider adding it around the `TcpListener::bind` call.

## Development notes
- ButtonEvent type:
  - Fields: `button: String`, `state: String`, `timestamp: u64`
  - Broadcast is implemented via `tokio::sync::broadcast` with a channel size of 100.
- The server ignores text frames from clients; only Close is handled to end the connection.
- The dashboard uses a WebSocket client to subscribe to events and provides basic visualizations.

## Troubleshooting
- Nothing appears on the dashboard:
  - Ensure you’re posting events to `/api/button` with valid JSON.
  - Check the browser console for WebSocket connection status/errors.
  - Verify the server logs. If started via Makefile, tail `logs/ws-server.log`.
- Can’t bind to port 3000:
  - Another process may be using it; stop it or change the port in `src/main.rs`.
- Cross-compilation issues with Makefile:
  - You may need the aarch64 target toolchain: `rustup target add aarch64-unknown-linux-gnu`
  - Install a suitable linker for your OS, or adapt the Makefile to your native target.
