use std::env;
use std::sync::OnceLock;
use dotenv::dotenv;

#[derive(Debug)]
pub struct Config {
    pub api_url: String,
    pub debug: bool,
}

static CONFIG: OnceLock<Config> = OnceLock::new();

pub fn init() {
    dotenv().ok();

    let api_url = env::var("API_URL")
        .expect("API_URL must be set in .env");

    let debug = env::var("DEBUG")
        .map(|val| val == "true" || val == "1")
        .unwrap_or(false);

    println!("Debug: {}", debug);

    CONFIG.set(Config { api_url, debug })
        .expect("Config already initialized");
}

fn get_config() -> &'static Config {
    CONFIG.get().expect("Config not initialized. Call config::init() first.")
}

pub fn api_url() -> &'static str {
    &get_config().api_url
}

pub fn is_debug() -> bool {
    get_config().debug
}