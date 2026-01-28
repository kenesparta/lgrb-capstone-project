use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        State,
    },
    http::StatusCode,
    response::{Html, IntoResponse},
};
use axum::extract::ws::Utf8Bytes;
use futures_util::{sink::SinkExt, stream::StreamExt};

use crate::event::ButtonEvent;
use crate::state::AppState;

pub async fn websocket_handler(
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
            match serde_json::to_string(&event) {
                Ok(msg) => {
                    if sender.send(Message::Text(Utf8Bytes::from(msg))).await.is_err() {
                        break;
                    }
                }
                Err(e) => {
                    eprintln!("Failed to serialize event: {}", e);
                }
            }
        }
    });

    let recv_task = tokio::spawn(async move {
        while let Some(result) = receiver.next().await {
            match result {
                Ok(msg) => match msg {
                    Message::Close(_) => break,
                    _ => {}
                },
                Err(e) => {
                    eprintln!("WebSocket receive error: {}", e);
                    break;
                }
            }
        }
    });

    tokio::select! {
        _ = send_task => {},
        _ = recv_task => {},
    }
}

pub async fn serve_html() -> impl IntoResponse {
    Html(include_str!("../index.html"))
}

pub async fn button_event(
    State(state): State<AppState>,
    axum::extract::Json(event): axum::extract::Json<ButtonEvent>,
) -> impl IntoResponse {
    println!("Received button event: {:?}", event);

    match state.button_tx.send(event) {
        Ok(receiver_count) => {
            println!("Event broadcasted to {} receivers", receiver_count);
        }
        Err(_) => {
            println!("No active WebSocket connections to broadcast to");
        }
    }

    (StatusCode::OK, "Event received")
}
