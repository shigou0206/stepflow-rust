#[derive(thiserror::Error, Debug)]
pub enum EventBusError {
    #[error("broadcast error: {0}")]
    BroadcastError(String),
} 