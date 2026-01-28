use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::config::WEB_SERVER_URL;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ButtonEvent {
    pub button: String,
    pub state: String,
    pub timestamp: u64,
}

pub async fn send_button_event(client: &Client, button: &str, state: &str) {
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
