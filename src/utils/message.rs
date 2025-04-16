use miniserde::{Serialize, Deserialize};
use miniserde::json::Object;

use json::JsonValue;

#[derive(Debug, Serialize, Deserialize)]
pub struct Message {
    pub source: String,
    pub destination: String,
    pub content: Object
}

#[derive(Debug)]
pub struct JSONMessage {
    pub source: String,
    pub destination: String,
    pub content: JsonValue,
}