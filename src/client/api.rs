use std::sync::Mutex;
use std::time::Instant;
use once_cell::sync::Lazy;
use tokio::sync::mpsc::{self, Receiver, Sender};
use json::JsonValue;
use crate::utils::debug_mode;


static LOGGER_SENDER: Lazy<Mutex<Option<Sender<JsonValue>>>> = Lazy::new(|| Mutex::new(None));

pub fn start_logger() -> Sender<JsonValue> {
    let (tx, rx) = mpsc::channel(100000000);
    set_logger(tx.clone());
    tokio::spawn(api_logger(rx));
    tx
}

pub fn send_to_logger(msg: JsonValue) {
    let guard = LOGGER_SENDER.lock().unwrap();
    if let Some(sender) = guard.as_ref() {
        match sender.try_send(msg) {
            Ok(_) => {}
            Err(tokio::sync::mpsc::error::TrySendError::Full(_)) => {
                debug_mode::log("[ERROR] API logger queue is full. Dropping message.");
            }
            Err(tokio::sync::mpsc::error::TrySendError::Closed(_)) => {
                debug_mode::log("[ERROR] API logger is closed.");
            }
        }
    } else {
        debug_mode::log("[ERROR] API logger is not initialized.");
    }
}

pub fn set_logger(sender: Sender<JsonValue>) {
    *LOGGER_SENDER.lock().unwrap() = Some(sender);
}

pub fn get_logger() -> Option<Sender<JsonValue>> {
    LOGGER_SENDER.lock().unwrap().as_ref().cloned()
}

pub async fn api_logger(mut rx: Receiver<JsonValue>) {
    let mut start_time: Option<Instant> = None;
    let mut message_count = 0;

    while let Some(mut msg) = rx.recv().await {
        // Start timer on first message
        if start_time.is_none() {
            start_time = Some(Instant::now());
        }

        message_count += 1;

        // Add timestamp
        let now = time::OffsetDateTime::now_utc();
        msg["timestamp"] = now.format(&time::format_description::well_known::Rfc3339).unwrap().into();

        debug_mode::log(&format!("[API QUEUE] Logging: {}", msg));

        if let Err(e) = send_to_api(msg).await {
            debug_mode::log(&format!("[API ERROR] {}", e));
        }
    }

    if let Some(start) = start_time {
        let duration = start.elapsed();
        eprintln!(
            "[API] Processed {} messages in {:.2?} ({:.2} msg/sec)",
            message_count,
            duration,
            message_count as f64 / duration.as_secs_f64()
        );
    } else {
        debug_mode::log("[API] No messages were processed.");
    }
}


async fn send_to_api(msg: JsonValue) -> Result<(), reqwest::Error> {
    let client = reqwest::Client::new();
    client
        .post(crate::utils::config::api_url())
        .header("Content-Type", "application/json")
        .body(msg.dump())
        .send()
        .await?
        .error_for_status()?; // force Err on 400/500

    Ok(())
}