use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        State,
    },
    http::StatusCode,
    response::{Html, IntoResponse},
    routing::get,
    Router,
};
use futures_util::{sink::SinkExt, stream::StreamExt};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};
use tokio::sync::broadcast;
use tower_http::services::ServeDir;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ButtonEvent {
    pub button: String,
    pub state: String,
    pub timestamp: u64,
}

type Clients = Arc<Mutex<HashMap<String, broadcast::Sender<ButtonEvent>>>>;

#[derive(Clone)]
struct AppState {
    clients: Clients,
    button_tx: broadcast::Sender<ButtonEvent>,
}

async fn websocket_handler(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_socket(socket, state))
}

async fn handle_socket(socket: WebSocket, state: AppState) {
    let (mut sender, mut receiver) = socket.split();
    let mut button_rx = state.button_tx.subscribe();

    let send_task = tokio::spawn(async move {
        while let Ok(event) = button_rx.recv().await {
            if let Ok(msg) = serde_json::to_string(&event) {
                if sender.send(Message::Text(msg)).await.is_err() {
                    break;
                }
            }
        }
    });

    let recv_task = tokio::spawn(async move {
        while let Some(msg) = receiver.next().await {
            if let Ok(msg) = msg {
                match msg {
                    Message::Text(_) => {
                        // Handle text messages if needed
                    }
                    Message::Close(_) => break,
                    _ => {}
                }
            } else {
                break;
            }
        }
    });

    tokio::select! {
        _ = send_task => {},
        _ = recv_task => {},
    }
}

async fn serve_html() -> impl IntoResponse {
    Html(include_str!("../index.html"))
}

// API endpoint to receive button events from the listener service
async fn button_event(
    State(state): State<AppState>,
    axum::extract::Json(event): axum::extract::Json<ButtonEvent>,
) -> impl IntoResponse {
    println!("Received button event: {:?}", event);

    // Send the event, but don't fail if there are no receivers
    match state.button_tx.send(event) {
        Ok(receiver_count) => {
            println!("Event broadcasted to {} receivers", receiver_count);
        }
        Err(_) => {
            // This happens when there are no active receivers (WebSocket connections)
            println!("No active WebSocket connections to broadcast to");
        }
    }

    (StatusCode::OK, "Event received")
}


#[tokio::main]
async fn main() {
    let (button_tx, _) = broadcast::channel(100);
    let clients = Arc::new(Mutex::new(HashMap::new()));

    let app_state = AppState {
        clients: clients.clone(),
        button_tx: button_tx.clone(),
    };

    let app = Router::new()
        .route("/", get(serve_html))
        .route("/ws", get(websocket_handler))
        .route("/api/button", axum::routing::post(button_event))
        .nest_service("/pkg", ServeDir::new("pkg"))
        .with_state(app_state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000")
        .await
        .unwrap();

    println!("🚀 Web server running on http://0.0.0.0:3000");

    axum::serve(listener, app).await.unwrap();
}