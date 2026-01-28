build-ws-server:
	cargo build --release --target aarch64-unknown-linux-gnu -p ws-server

build-ble-listener:
	cargo build --release --target aarch64-unknown-linux-gnu -p ble-listener

run: build-ws-server build-ble-listener
	mkdir -p logs
	nohup ./target/aarch64-unknown-linux-gnu/release/ws-server > logs/ws-server.log 2>&1 &
	nohup ./target/aarch64-unknown-linux-gnu/release/ble-listener > logs/ble-listener.log 2>&1 &
	@echo "Services started in background"
	@echo "ws-server logs: logs/ws-server.log"
	@echo "ble-listener logs: logs/ble-listener.log"
	@echo "Use 'make stop' to stop services or 'make status' to check running processes"

# Check status of services
status:
	@echo "Service status:"
	@pgrep -f "ws-server" > /dev/null && echo "  ws-server: running (PID: $$(pgrep -f ws-server))" || echo "  ws-server: stopped"
	@pgrep -f "ble-listener" > /dev/null && echo "  ble-listener: running (PID: $$(pgrep -f ble-listener))" || echo "  ble-listener: stopped"

# Stop both services
stop:
	@echo "Stopping services..."
	-pkill -f "ws-server"
	-pkill -f "ble-listener"
	@echo "Services stopped"

# View logs in real-time
logs-ws:
	tail -f logs/ws-server.log

logs-ble:
	tail -f logs/ble-listener.log

logs-all:
	tail -f logs/*.log

# Clean up log files
clean-logs:
	rm -rf logs/
