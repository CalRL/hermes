use std::sync::Arc;
use tokio::io::{AsyncWriteExt, WriteHalf};
use tokio::net::TcpStream;
use tokio::sync::Mutex;
use crate::utils::debug_mode;
use crate::utils::message::JSONMessage;

pub async fn forward_to_peer(
    stream_mutex: Arc<Mutex<WriteHalf<TcpStream>>>,
    raw_msg: &str,
) -> String {
    debug_mode::log("🔐 Locking destination stream to send...");
    let mut stream = stream_mutex.lock().await;

    debug_mode::log("✅ Lock acquired, writing...");
    match stream.write_all(format!("{raw_msg}\n").as_bytes()).await {
        Ok(_) => "Message sent".to_string(),
        Err(e) => format!("Failed to write: {}", e),
    }
}

pub fn post_to_api(message: &JSONMessage) {
    let client = reqwest::blocking::Client::new();
    let res = client
        .post("http://your-api-server.local/messages")
        //.json(message)
        .send();

    if let Err(e) = res {
        eprintln!("API post failed: {}", e);
    }
}