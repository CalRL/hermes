use criterion::{criterion_group, criterion_main, Criterion};
use miniserde::{Deserialize, Serialize};
use miniserde::json as mini_json;
use json as raw_json;

const RAW: &str = r#"
{
    "source": "127.0.0.1",
    "destination": "10.0.0.1",
    "content": {
        "type": "ping",
        "payload": [1, 2, 3, 4]
    }
}
"#;

#[derive(Debug, Serialize, Deserialize)]
struct MiniMessage {
    source: String,
    destination: String,
    content: mini_json::Value,
}

fn miniserde_parse() {
    let _: MiniMessage = mini_json::from_str(RAW).unwrap();
}

fn json_parse() {
    let _ = raw_json::parse(RAW).unwrap();
}

fn bench_json_parsing(c: &mut Criterion) {
    c.bench_function("miniserde parse", |b| b.iter(|| miniserde_parse()));
    c.bench_function("json parse", |b| b.iter(|| json_parse()));
}

criterion_group!(benches, bench_json_parsing);
criterion_main!(benches);