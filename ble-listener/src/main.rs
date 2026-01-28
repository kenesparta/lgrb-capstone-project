mod bluetooth;
mod config;
mod event;

use btleplug::api::{Central, Manager as _, Peripheral as _};
use btleplug::platform::Manager;
use reqwest::Client;
use std::error::Error;

use crate::bluetooth::{connect_and_listen, find_device};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    println!("🚀 Starting BLE Button Tester with WebSocket");
    println!("{}", "=".repeat(50));

    let client = Client::new();

    let manager = Manager::new()
        .await
        .map_err(|e| format!("Failed to create Bluetooth manager: {}", e))?;

    let adapters = manager
        .adapters()
        .await
        .map_err(|e| format!("Failed to get Bluetooth adapters: {}", e))?;

    let adapter = adapters
        .into_iter()
        .next()
        .ok_or("No Bluetooth adapters found")?;

    let adapter_info = adapter
        .adapter_info()
        .await
        .map_err(|e| format!("Failed to get adapter info: {}", e))?;

    println!("Using adapter: {}", adapter_info);

    match find_device(&adapter).await {
        Ok(peripheral) => {
            if let Err(e) = connect_and_listen(&peripheral, &client).await {
                println!("❌ Connection error: {}", e);
            }

            if peripheral.is_connected().await.unwrap_or(false) {
                if let Err(e) = peripheral.disconnect().await {
                    println!("❌ Failed to disconnect: {}", e);
                }
            }
        }
        Err(e) => {
            println!("❌ Device discovery failed: {}", e);
        }
    }

    println!("👋 Goodbye!");
    Ok(())
}
