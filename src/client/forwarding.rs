use std::sync::Arc;
use std::thread;
use tokio::io::{AsyncWriteExt, WriteHalf};
use tokio::net::TcpStream;
use tokio::sync::Mutex;
use crate::utils::debug_mode;

pub async fn forward_to_peer(
    stream_mutex: Arc<Mutex<WriteHalf<TcpStream>>>,
    raw_msg: &str,
) -> String {
    debug_mode::log("🔐 Locking destination stream to send...");
    let mut stream = stream_mutex.lock().await;

    debug_mode::log(&format!("✅ Lock acquired, writing: {}", raw_msg));
    match stream.write_all(format!("{raw_msg}\n").as_bytes()).await {
        Ok(_) => { "Message sent to peer".to_string() }
        Err(e) => format!("Failed to write: {}", e),
    }

    //todo: log to api
}