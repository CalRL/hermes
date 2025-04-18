use std::net::IpAddr;

pub mod message;
pub mod debug_mode;

pub fn is_valid_ip(ip: &str) -> bool {
    ip.parse::<IpAddr>().is_ok()
}