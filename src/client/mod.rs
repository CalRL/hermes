use std::net::{IpAddr, SocketAddr};
use std::sync::Arc;
use crate::utils::debug_mode;
use crate::SharedConnections;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader, WriteHalf};
use tokio::net::TcpStream;
use tokio::sync::{Mutex};
use crate::client::forwarding::forward_to_peer;
use crate::utils::message::JSONMessage;

mod forwarding;
mod lookup;
mod spring;

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
            debug_mode::log(&format!("[{}] > {}", addr_for_reader, line));

            match json::parse(&line) {
                Ok(parsed) => {
                    if let Some(dest_ip) = parsed["destination"].as_str() {
                        if let Some((resolved_ip, stream_mutex)) =
                            resolve_ip(dest_ip, Arc::clone(&connections_reader)).await
                        {
                            let result = forward_to_peer(stream_mutex, &line).await;
                            debug_mode::log(&format!("[FORWARD to {}] {}", resolved_ip, result));
                        } else {
                            debug_mode::log(&format!("[WARN] No connection for {}", dest_ip));
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

pub async fn resolve_ip(ip: &str, connections: SharedConnections) -> Option<(String, Arc<Mutex<WriteHalf<TcpStream>>>)> {

    let target_ip: IpAddr = ip.parse().ok()?;

    let map = connections.read().await;

    map.iter()
        .find_map(|(key, stream)| {
            key.parse::<SocketAddr>()
                .ok()
                .filter(|addr| addr.ip() == target_ip)
                .map(|_| (key.clone(), Arc::clone(stream)))
        })
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

pub async fn log_to_api(message: JSONMessage) {

}