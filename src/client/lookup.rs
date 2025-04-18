use std::net::{IpAddr, SocketAddr};
use crate::SharedConnections;

pub async fn match_ip_port(
    ip: &str,
    connections: &SharedConnections
) -> Option<SocketAddr> {
    let target_ip: IpAddr = match ip.parse() {
        Ok(ip) => ip,
        Err(_) => return None,
    };

    let map = connections.read().await;

    map.keys()
        .filter_map(|addr_str| addr_str.parse::<SocketAddr>().ok())
        .find(|sock| sock.ip() == target_ip)
}