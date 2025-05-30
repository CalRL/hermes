mod utils;
mod client;

use std::collections::HashMap;
use std::io::Write;
use std::sync::Arc;
use std::time::Duration;
use utils::config;
use tokio::io::WriteHalf;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{Mutex, RwLock};


pub type SharedConnections = Arc<RwLock<HashMap<String, Arc<Mutex<WriteHalf<TcpStream>>>>>>;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    config::init();
    let _ = client::api::start_logger();

    let listener = TcpListener::bind("0.0.0.0:8000").await?;
    println!("Listening on 0.0.0.0:8000");

    let connections: SharedConnections = Arc::new(RwLock::new(HashMap::new()));

    // Keep-alive loop
    {
        let connections = Arc::clone(&connections);
        tokio::spawn(async move {
            loop {
                tokio::time::sleep(Duration::from_secs(300)).await;
                client::send_keep_alives(&connections).await;
            }
        });
    }

    loop {
        let (mut stream, addr) = listener.accept().await?;

        // add lingering to not close the connection
        let twelve_hours: u64 = 43200;
        stream.set_linger(Some(Duration::from_secs(twelve_hours))).expect("set_linger call failed");

        let addr_str = addr.to_string();
        println!("New connection: {}", addr_str);

        let connections = Arc::clone(&connections);
        tokio::spawn(async move {
            client::handle_connection(stream, addr_str, connections).await;
        });
    }
}