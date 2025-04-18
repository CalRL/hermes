use json::JsonValue;

#[derive(Debug)]
pub struct JSONMessage {
    pub source: String,
    pub destination: String,
    pub content: JsonValue,
}