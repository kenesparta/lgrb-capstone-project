mod config;
mod event;
mod handlers;
mod state;

use axum::{routing::get, Router};
use std::error::Error;
use tower_http::services::ServeDir;

use crate::config::{BROADCAST_CHANNEL_CAPACITY, SERVER_ADDRESS};
use crate::handlers::{button_event, serve_html, websocket_handler};
use crate::state::AppState;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let app_state = AppState::new(BROADCAST_CHANNEL_CAPACITY);

    let app = Router::new()
        .route("/", get(serve_html))
        .route("/ws", get(websocket_handler))
        .route("/api/button", axum::routing::post(button_event))
        .nest_service("/pkg", ServeDir::new("pkg"))
        .with_state(app_state);

    let listener = tokio::net::TcpListener::bind(SERVER_ADDRESS)
        .await
        .map_err(|e| format!("Failed to bind to {}: {}", SERVER_ADDRESS, e))?;

    println!("🚀 Web server running on http://{}", SERVER_ADDRESS);

    axum::serve(listener, app)
        .await
        .map_err(|e| format!("Server error: {}", e))?;

    Ok(())
}
