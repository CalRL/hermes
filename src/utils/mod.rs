use std::net::IpAddr;
use dotenv::dotenv;
use std::env;

pub mod message;
pub mod debug_mode;
pub mod config;

pub fn is_valid_ip(ip: &str) -> bool {
    ip.parse::<IpAddr>().is_ok()
}

pub struct Config {
    pub api_url: String,
    pub debug: bool
}

impl Config {
    pub fn load() -> Self {
        dotenv().ok(); // Load from .env

        let api_url = env::var("API_URL")
            .expect("API_URL must be set in .env");

        let debug = env::var("DEBUG")
            .map(|val| val == "true" || val == "1")
            .unwrap_or(false);

        Self { api_url, debug }
    }
}