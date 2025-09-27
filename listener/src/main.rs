
use btleplug::api::{
    bleuuid::BleUuid, Central, CentralEvent, Manager as _, Peripheral as _, ScanFilter,
    WriteType,
};
use btleplug::platform::{Adapter, Manager, Peripheral};
use futures::stream::StreamExt;
use std::error::Error;
use std::time::Duration;
use tokio::time;
use uuid::Uuid;

// UUIDs matching your micro:bit server
// const BUTTON_SERVICE_UUID: &str = "EF680800-9B35-4933-9B10-52FFA9740042";
// const BUTTON_CHAR_UUID: &str = "EF680801-9B35-4933-9B10-52FFA9740042";
const BATTERY_SERVICE_UUID: &str = "0000180F-0000-1000-8000-00805F9B34FB";
const BATTERY_LEVEL_UUID: &str = "00002A19-0000-1000-8000-00805F9B34FB";

// const DEVICE_NAME: &str = "LGR-BLE";
const DEVICE_NAME: &str = "LGR-BLE";

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

fn handle_button_notification(data: &[u8]) {
    if !data.is_empty() {
        let value = data[0];
        match value {
            1 => println!("🔴 Button A (LEFT) PRESSED"),
            2 => println!("🔵 Button B (RIGHT) PRESSED"),
            0 => println!("⚪ Button RELEASED"),
            _ => println!("Unknown button value: {}", value),
        }
    } else {
        println!("Received empty notification data");
    }
}

async fn connect_and_listen(peripheral: &Peripheral) -> Result<(), Box<dyn Error>> {
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
    println!("Press Ctrl+C to stop\n");

    // Listen for notifications
    let mut notification_stream = peripheral.notifications().await?;

    loop {
        tokio::select! {
            Some(data) = notification_stream.next() => {
                handle_button_notification(&data.value);
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
    println!("🚀 Starting BLE Button Tester");
    println!("{}", "=" .repeat(40));

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
        match connect_and_listen(&peripheral).await {
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