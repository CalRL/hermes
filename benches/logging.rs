use std::time::Instant;
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn println_loop() {
    let start = Instant::now();

    for _ in 0..1000 {
        println!("This is a test");
    }

    let elapsed = start.elapsed();
    println!("Wall time: {:.2?}", elapsed);
}

fn empty_loop() {
    let mut i: i64 = 0;
    for _ in 0..1000 {
        i+=1;
    }
    black_box(i);
}

fn benchmark_println_vs_empty(c: &mut Criterion) {
    c.bench_function("println! loop", |b| b.iter(|| println_loop()));
    c.bench_function("empty loop", |b| b.iter(|| empty_loop()));
}

criterion_group!(benches, benchmark_println_vs_empty);
criterion_main!(benches);