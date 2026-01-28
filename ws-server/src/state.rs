use tokio::sync::broadcast;

use crate::event::ButtonEvent;

#[derive(Clone)]
pub struct AppState {
    pub button_tx: broadcast::Sender<ButtonEvent>,
}

impl AppState {
    pub fn new(capacity: usize) -> Self {
        let (button_tx, _) = broadcast::channel(capacity);
        Self { button_tx }
    }
}
