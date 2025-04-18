use clap::Parser;

#[derive(Parser, Debug)]
#[command(author, version, about)]
struct Args {
    #[arg(long)]
    debug: bool,
}

const PREFIX: &str = "[DEBUG]";

pub fn is_enabled() -> bool {
    let args = Args::parse();
    args.debug
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