use criterion::{black_box, criterion_group, criterion_main, Criterion};

pub fn get_block_by_id(c: &mut Criterion) {
    c.bench_function("get block by id using hash: last", |b| {
        b.iter(|| {
            pumpkin_world::block::block_registry::get_block(black_box(
                "minecraft:pale_hanging_moss",
            ))
        })
    });
    c.bench_function("get block by id using hash: first", |b| {
        b.iter(|| pumpkin_world::block::block_registry::get_block(black_box("minecraft:air")))
    });
}

criterion_group!(benches, get_block_by_id);
criterion_main!(benches);
