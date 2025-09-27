use btleplug::api::{
    bleuuid::BleUuid, Central, CentralEvent, Manager as _, Peripheral as _, ScanFilter,
    WriteType,
};
use btleplug::platform::{Adapter, Manager, Peripheral};
use futures::stream::StreamExt;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::time;
use uuid::Uuid;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ButtonEvent {
    pub button: String,
    pub state: String,
    pub timestamp: u64,
}

// UUIDs matching your micro:bit server
const BATTERY_SERVICE_UUID: &str = "0000180F-0000-1000-8000-00805F9B34FB";
const BATTERY_LEVEL_UUID: &str = "00002A19-0000-1000-8000-00805F9B34FB";
const DEVICE_NAME: &str = "LGR-BLE";
const WEB_SERVER_URL: &str = "http://127.0.0.1:3000/api/button";

async fn find_device(adapter: &Adapter) -> Option<Peripheral> {
    println!("🔍 Scanning for {} device...", DEVICE_NAME);

    // Start scanning
    adapter
        .start_scan(ScanFilter::default())
        .await
        .expect("Can't scan BLE adapter for connected devices");

    time::sleep(Duration::from_secs(10)).await;

    // Get discovered devices
    let peripherals = adapter.peripherals().await.unwrap();

    if peripherals.is_empty() {
        println!("❌ No BLE devices found");
        return None;
    }

    println!("Found {} BLE devices:", peripherals.len());

    // Find our target device
    for peripheral in peripherals {
        let properties = peripheral.properties().await.unwrap();
        let name = properties
            .and_then(|p| p.local_name)
            .unwrap_or_else(|| "Unknown".to_string());

        println!("  - {} ({})", name, peripheral.address());

        if name == DEVICE_NAME {
            println!("✅ Found device: {} ({})", name, peripheral.address());
            return Some(peripheral);
        }
    }

    println!("❌ {} device not found!", DEVICE_NAME);
    println!("Make sure your micro:bit is running and advertising");
    None
}

async fn send_button_event(client: &Client, button: &str, state: &str) {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64;

    let event = ButtonEvent {
        button: button.to_string(),
        state: state.to_string(),
        timestamp,
    };

    match client.post(WEB_SERVER_URL).json(&event).send().await {
        Ok(response) => {
            if response.status().is_success() {
                println!("📤 Sent {} {} to web server", button, state);
            } else {
                println!("❌ Failed to send event: HTTP {}", response.status());
            }
        }
        Err(e) => {
            println!("❌ Network error sending event: {}", e);
        }
    }
}

fn handle_button_notification(data: &[u8], client: &Client) {
    if !data.is_empty() {
        let value = data[0];
        let rt = tokio::runtime::Handle::current();

        match value {
            1 => {
                println!("🔴 Button A (LEFT) PRESSED");
                let client = client.clone();
                rt.spawn(async move {
                    send_button_event(&client, "A", "PRESSED").await;
                });
            }
            2 => {
                println!("🔵 Button B (RIGHT) PRESSED");
                let client = client.clone();
                rt.spawn(async move {
                    send_button_event(&client, "B", "PRESSED").await;
                });
            }
            0 => {
                println!("⚪ Button RELEASED");
                let client = client.clone();
                rt.spawn(async move {
                    send_button_event(&client, "ANY", "RELEASED").await;
                });
            }
            _ => println!("Unknown button value: {}", value),
        }
    } else {
        println!("Received empty notification data");
    }
}

async fn connect_and_listen(peripheral: &Peripheral, client: &Client) -> Result<(), Box<dyn Error>> {
    println!("🔗 Connecting to device...");

    // Connect to device
    peripheral.connect().await?;
    println!("🔗 Connected: {}", peripheral.is_connected().await?);

    // Discover services
    peripheral.discover_services().await?;
    let services = peripheral.services();

    println!("\n📋 Available services ({}):", services.len());

    let mut button_char_found = false;

    // Iterate through services and characteristics
    for service in &services {
        println!("  🔹 Service {}", service.uuid);

        for characteristic in &service.characteristics {
            let props: Vec<String> = characteristic.properties.iter()
                .map(|p| format!("{:?}", p))
                .collect();
            println!("    └─ Characteristic {} ({})",
                     characteristic.uuid,
                     props.join(", "));

            // Subscribe to notifications for characteristics that support it
            if characteristic.properties.contains(btleplug::api::CharPropFlags::NOTIFY) {
                println!("    📡 Attempting to subscribe to notifications on {}",
                         characteristic.uuid);

                match peripheral.subscribe(&characteristic).await {
                    Ok(_) => {
                        println!("    ✅ Successfully subscribed to {}", characteristic.uuid);
                        button_char_found = true;
                    }
                    Err(e) => {
                        println!("    ❌ Failed to subscribe to {}: {}",
                                 characteristic.uuid, e);
                    }
                }
            }
        }
    }

    if !button_char_found {
        return Err("❌ No notify characteristics found!".into());
    }

    // Try to read battery level
    let battery_service_uuid = Uuid::parse_str(BATTERY_SERVICE_UUID)?;
    let battery_char_uuid = Uuid::parse_str(BATTERY_LEVEL_UUID)?;

    for service in &services {
        if service.uuid == battery_service_uuid {
            for characteristic in &service.characteristics {
                if characteristic.uuid == battery_char_uuid
                    && characteristic.properties.contains(btleplug::api::CharPropFlags::READ) {
                    match peripheral.read(&characteristic).await {
                        Ok(data) => {
                            if !data.is_empty() {
                                println!("🔋 Battery Level: {}%", data[0]);
                            }
                        }
                        Err(e) => {
                            println!("Could not read battery level: {}", e);
                        }
                    }
                    break;
                }
            }
            break;
        }
    }

    println!("\n🎮 Ready! Press buttons A or B on your micro:bit...");
    println!("📡 Events will be sent to web browser at http://127.0.0.1:3000");
    println!("Press Ctrl+C to stop\n");

    // Listen for notifications
    let mut notification_stream = peripheral.notifications().await?;

    loop {
        tokio::select! {
            Some(data) = notification_stream.next() => {
                handle_button_notification(&data.value, client);
            }
            _ = tokio::signal::ctrl_c() => {
                println!("\n🛑 Stopping...");
                break;
            }
        }
    }

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    println!("🚀 Starting BLE Button Tester with WebSocket");
    println!("{}", "=" .repeat(50));

    // Create HTTP client for sending events to web server
    let client = Client::new();

    // Get the first Bluetooth adapter
    let manager = Manager::new().await?;
    let adapter_list = manager.adapters().await?;

    if adapter_list.is_empty() {
        return Err("No Bluetooth adapters found".into());
    }

    let adapter = match adapter_list.into_iter().next() {
        Some(adapter) => adapter,
        None => return Err("Failed to get first Bluetooth adapter".into()),
    };
    println!("Using adapter: {}", adapter.adapter_info().await?);

    // Find and connect to device
    if let Some(peripheral) = find_device(&adapter).await {
        match connect_and_listen(&peripheral, &client).await {
            Ok(_) => {}
            Err(e) => {
                println!("❌ Connection error: {}", e);
            }
        }

        // Disconnect
        if peripheral.is_connected().await? {
            peripheral.disconnect().await?;
        }
    }

    println!("👋 Goodbye!");
    Ok(())
}