use criterion::{criterion_group, criterion_main, Criterion};
use pumpkin_world::bench_create_and_populate_noise;

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("overworld convert + noise", |b| {
        b.iter(bench_create_and_populate_noise)
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
