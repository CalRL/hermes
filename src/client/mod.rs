use std::net::{IpAddr, SocketAddr, ToSocketAddrs};
use std::sync::Arc;
use std::time::SystemTime;
use json::{array, JsonValue};
use reqwest::{Client, Method, Url};
use time::format_description::well_known;
use time::OffsetDateTime;
use crate::utils::{config, debug_mode};
use crate::SharedConnections;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader, WriteHalf};
use tokio::net::TcpStream;
use tokio::sync::{Mutex};
use tokio::time::Instant;
use log::debug;
use crate::client::forwarding::forward_to_peer;
use crate::utils::message::JSONMessage;

mod forwarding;
mod lookup;
pub(crate) mod api;

pub async fn handle_connection(
    stream: TcpStream,
    addr: String,
    connections: SharedConnections,
) {
    debug_mode::log(&format!("[{}] Initializing connection handler", addr));

    // Init
    let (reader, mut writer) = tokio::io::split(stream);
    let writer = Arc::new(Mutex::new(writer));

    let addr_for_writer = addr.clone();
    let addr_for_reader = addr.clone();

    // Store sender
    {
        debug_mode::log(&format!("[{}] Registering sender in map", addr));
        let mut map = connections.write().await;
        map.insert(addr.clone(), Arc::clone(&writer));
    }

    // Reader task
    let connections_reader = Arc::clone(&connections);
    let read_task = tokio::spawn(async move {

        debug_mode::log(&format!("[{}] Reader task started", addr_for_reader));

        let mut reader = BufReader::new(reader).lines();
        while let Ok(Some(line)) = reader.next_line().await {
            debug_mode::log("-------------------------------------------------------------");
            debug_mode::log(&format!("[{}] Reader: > {}", addr_for_reader, line));

            match json::parse(&line) {
                Ok(mut parsed) => {
                    debug_mode::log(&format!("{}", parsed));

                    let parts: Vec<&str> = addr_for_reader.split(":").collect();
                    let source: JsonValue = JsonValue::String(parts[0].to_string());
                    debug_mode::log(source.as_str().unwrap());
                    parsed["source"] = source.clone();

                    let formatted: &str = &format!("New source: {}", source.clone());
                    debug_mode::log(formatted);

                    debug_mode::log(&format!(
                        "JSON Dump: {}",
                        parsed.clone().dump()
                    ));

                    if let Some(dest_ip) = parsed["destination"].as_str() {
                        if let Some((resolved_ip, stream_mutex)) =
                            resolve_ip(dest_ip, Arc::clone(&connections_reader)).await
                        {

                            let result = forward_to_peer(stream_mutex, &parsed.dump()).await;
                            debug_mode::log(&format!("[FORWARD to {}] {}", resolved_ip, result));
                            //reply_to_sender(&addr_for_reader, &result, connections_reader.clone()).await;
                            log_to_api(parsed.clone()).await.expect("TODO: panic message");
                            if let Some(logger) = api::get_logger() {
                                let _ = api::send_to_logger(parsed);
                            }

                        } else {
                            debug_mode::log(&format!("[WARN] No connection for {}", dest_ip));
                            reply_with_error(&addr_for_reader, &format!("Destination {} not connected", dest_ip), connections_reader.clone()).await;
                            let mut json = parsed;
                            json["content"] = JsonValue::from("fail, no dest");
                            log_to_api(json.clone()).await.expect("TODO: panic message");
                            if let Some(logger) = api::get_logger() {
                                let _ = api::send_to_logger(json);
                            }
                        }
                    }
                }
                Err(e) => {
                    debug_mode::log(&format!("[WARN] Failed to parse JSON: {}", e));
                }
            }
        }
        debug_mode::log(&format!("[{}] Reader task ended", addr_for_reader));
    });

    let _ = read_task.await;

    // Remove from map after disconnect
    {
        debug_mode::log(&format!("[{}] Cleaning up connection", addr));
        let mut map = connections.write().await;
        map.remove(&addr);
    }

    println!("Disconnected: {}", addr);
}

pub async fn resolve_ip(
    ip: &str,
    connections: SharedConnections,
) -> Option<(String, Arc<Mutex<WriteHalf<TcpStream>>>)> {
    let map = connections.read().await;

    if let Some(stream) = map.get(ip) {
        return Some((ip.to_string(), Arc::clone(stream)));
    }

    if let Ok(target_ip) = ip.parse::<IpAddr>() {
        return map.iter().find_map(|(key, stream)| {
            key.parse::<SocketAddr>().ok().and_then(|addr| {
                if addr.ip() == target_ip {
                    Some((key.clone(), Arc::clone(stream)))
                } else {
                    None
                }
            })
        });
    }

    None
}

pub async fn send_keep_alives(connections: &SharedConnections) {
    debug_mode::log("Sending keep-alives...");
    let mut to_remove = Vec::new();
    let map = connections.read().await;

    for (addr, tx) in map.iter() {
        let msg = r#"{ "type": "keepalive" }\n"#.to_string();
        debug_mode::log(&format!("[{}] Sending keepalive", addr));

        let mut stream = tx.lock().await;
        if let Err(e) = stream.write_all(msg.as_bytes()).await {
            eprintln!("Failed to send keepalive to {}: {}", addr, e);
            to_remove.push(addr.clone());
        }
    }
    drop(map);

    if !to_remove.is_empty() {
        debug_mode::log("Cleaning up disconnected clients...");
        let mut map = connections.write().await;
        for addr in to_remove {
            map.remove(&addr);
            println!("Removed inactive client: {}", addr);
        }
    }

    debug_mode::log("Keep-alive round complete.");
}

pub async fn log_to_api(mut msg: JsonValue) -> Result<(), reqwest::Error> {
    let now = OffsetDateTime::now_utc();
    let timestamp = now.format(&well_known::Rfc3339).unwrap();
    msg["timestamp"] = timestamp.into();

    debug_mode::log(&msg.dump());

    let client = Client::new();
    let response = client
        .post(config::api_url())
        .header(reqwest::header::CONTENT_TYPE, "application/json")
        .body(msg.dump()) // JSON body
        .send()
        .await?;

    debug_mode::log(&format!("[API] Logged successfully: {}", response.status()));
    Ok(())
}

async fn reply_to_sender(source_ip: &str, message: &str, shared_connections: SharedConnections) {
    if let Some((_, sender_mutex)) = resolve_ip(source_ip.clone(), shared_connections.clone()).await {
        let mut sender = sender_mutex.lock().await;
        if let Err(e) = sender.write_all(format!("{}\n", message).as_bytes()).await {
            debug_mode::log(&format!("[ERROR] Failed to respond to {}: {}", source_ip, e));
        } else {
            debug_mode::log(&format!("[{}] Sent response to original sender", source_ip));
        }
    } else {
        debug_mode::log(&format!("[WARN] Could not find source IP {} to send response", source_ip));
        debug_mode::log("-------------------------------------------------------------");
        let map = shared_connections.read().await;
        let keys: Vec<_> = map.keys().cloned().collect();
        debug_mode::log(&format!("Active Connections: {:?}", keys));
        debug_mode::log("-------------------------------------------------------------");
    }
}

pub async fn reply_with_error(
    addr: &str,
    message: &str,
    shared: SharedConnections
) {
    let map = shared.read().await;
    if let Some(sender_mutex) = map.get(addr) {
        let sender_mutex = Arc::clone(sender_mutex);
        let addr = addr.to_string();
        let msg = "Message sent to peer\n".to_string();
        debug_mode::log("msg created");

        tokio::spawn(async move {
            let mut stream = sender_mutex.lock().await;
            if let Err(e) = stream.write_all(msg.as_bytes()).await {
                debug_mode::log(&format!("[ERROR] Failed to notify {}: {}", addr, e));
            }
            debug_mode::log("Sent!")
        });
    }
}
