use crate::utils::config;
const PREFIX: &str = "[DEBUG]";


pub fn is_enabled() -> fn() -> bool {
    let bool = config::is_debug;
    bool
}

pub fn log(message: &str) {
    if is_enabled() {
        println!("{PREFIX} {}", message);
    }
}

pub fn warn(message: &str) {
    if is_enabled() {
        eprintln!("{PREFIX} {}", message);
    }
}