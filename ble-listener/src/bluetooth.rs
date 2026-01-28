use btleplug::api::{Central, Peripheral as _, ScanFilter};
use btleplug::platform::{Adapter, Peripheral};
use futures::stream::StreamExt;
use reqwest::Client;
use std::error::Error;
use std::time::Duration;
use tokio::time;
use uuid::Uuid;

use crate::config::{BATTERY_LEVEL_UUID, BATTERY_SERVICE_UUID, DEVICE_NAME};
use crate::event::send_button_event;

pub async fn find_device(adapter: &Adapter) -> Result<Peripheral, Box<dyn Error>> {
    println!("🔍 Scanning for {} device...", DEVICE_NAME);

    adapter
        .start_scan(ScanFilter::default())
        .await
        .map_err(|e| format!("Failed to start BLE scan: {}", e))?;

    time::sleep(Duration::from_secs(10)).await;

    let peripherals = adapter
        .peripherals()
        .await
        .map_err(|e| format!("Failed to get peripherals: {}", e))?;

    if peripherals.is_empty() {
        return Err("No BLE devices found".into());
    }

    println!("Found {} BLE devices:", peripherals.len());

    for peripheral in peripherals {
        let properties = peripheral
            .properties()
            .await
            .map_err(|e| format!("Failed to get device properties: {}", e))?;

        let name = properties
            .and_then(|p| p.local_name)
            .unwrap_or_else(|| "Unknown".to_string());

        println!("  - {} ({})", name, peripheral.address());

        if name == DEVICE_NAME {
            println!("✅ Found device: {} ({})", name, peripheral.address());
            return Ok(peripheral);
        }
    }

    Err(format!(
        "{} device not found. Make sure your micro:bit is running and advertising.",
        DEVICE_NAME
    )
    .into())
}

pub fn handle_button_notification(data: &[u8], client: &Client) {
    if data.is_empty() {
        println!("Received empty notification data");
        return;
    }

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
}

pub async fn connect_and_listen(
    peripheral: &Peripheral,
    client: &Client,
) -> Result<(), Box<dyn Error>> {
    println!("🔗 Connecting to device...");

    peripheral.connect().await?;
    println!("🔗 Connected: {}", peripheral.is_connected().await?);

    peripheral.discover_services().await?;
    let services = peripheral.services();

    println!("\n📋 Available services ({}):", services.len());

    let mut button_char_found = false;

    for service in &services {
        println!("  🔹 Service {}", service.uuid);

        for characteristic in &service.characteristics {
            let props: Vec<String> = characteristic
                .properties
                .iter()
                .map(|p| format!("{:?}", p))
                .collect();
            println!(
                "    └─ Characteristic {} ({})",
                characteristic.uuid,
                props.join(", ")
            );

            if characteristic
                .properties
                .contains(btleplug::api::CharPropFlags::NOTIFY)
            {
                println!(
                    "    📡 Attempting to subscribe to notifications on {}",
                    characteristic.uuid
                );

                match peripheral.subscribe(&characteristic).await {
                    Ok(_) => {
                        println!("    ✅ Successfully subscribed to {}", characteristic.uuid);
                        button_char_found = true;
                    }
                    Err(e) => {
                        println!(
                            "    ❌ Failed to subscribe to {}: {}",
                            characteristic.uuid, e
                        );
                    }
                }
            }
        }
    }

    if !button_char_found {
        return Err("❌ No notify characteristics found!".into());
    }

    read_battery_level(peripheral, &services).await;

    println!("\n🎮 Ready! Press buttons A or B on your micro:bit...");
    println!("📡 Events will be sent to the web browser at http://127.0.0.1:3000");
    println!("Press Ctrl+C to stop\n");

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

async fn read_battery_level(
    peripheral: &Peripheral,
    services: &std::collections::BTreeSet<btleplug::api::Service>,
) {
    let battery_service_uuid = match Uuid::parse_str(BATTERY_SERVICE_UUID) {
        Ok(uuid) => uuid,
        Err(_) => return,
    };

    let battery_char_uuid = match Uuid::parse_str(BATTERY_LEVEL_UUID) {
        Ok(uuid) => uuid,
        Err(_) => return,
    };

    for service in services {
        if service.uuid == battery_service_uuid {
            for characteristic in &service.characteristics {
                if characteristic.uuid == battery_char_uuid
                    && characteristic
                        .properties
                        .contains(btleplug::api::CharPropFlags::READ)
                {
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
                    return;
                }
            }
            return;
        }
    }
}
