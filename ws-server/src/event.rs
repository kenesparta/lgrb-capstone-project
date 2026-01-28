use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ButtonEvent {
    pub button: String,
    pub state: String,
    pub timestamp: u64,
}
