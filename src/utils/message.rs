use json::JsonValue;
use tokio::time::Instant;

#[derive(Debug)]
pub struct JSONMessage {
    pub source: String,
    pub destination: String,
    pub content: JsonValue,
    pub timestamp: Instant,
    pub machine_type: String,
}