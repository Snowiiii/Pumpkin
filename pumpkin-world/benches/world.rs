use criterion::{black_box, criterion_group, criterion_main, Criterion};
use pumpkin_core::math::vector2::Vector2;
use pumpkin_world::chunk::ChunkReader;
use pumpkin_world::level::SaveFile;
use std::path::PathBuf;

fn anvil_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("AnvilChunkReader");
    
    // Our method
    let save_file = SaveFile {
        root_folder: PathBuf::from(""),
        region_folder: PathBuf::from("../.etc/regions"),
    };
    let reader = pumpkin_world::chunk::anvil::AnvilChunkReader::new();
    group.bench_function("Custom", |b| b.iter(|| {
        black_box(reader.read_chunk(&save_file, Vector2::new(0, 0)).unwrap());
    }));
    
    // Memory-mapped
    let loaded_mapped_file = pumpkin_world::chunk::unsafe_anvil::load_anvil_file(PathBuf::from("../.etc/regions/r.0.0.mca")).unwrap();
    group.bench_function("MMap", |b| b.iter(|| {
        black_box(loaded_mapped_file.get_chunk(0, 0).unwrap());
    }));
    
    // FastAnvil
    let mut fast_region = fastanvil::Region::from_stream(std::fs::File::open("../.etc/regions/r.0.0.mca").unwrap()).unwrap();
    group.bench_function("FastAnvil", |b| b.iter(|| {
        black_box(fast_region.read_chunk(0, 0).unwrap());
    }));
    
    group.finish();
}

criterion_group!(benches, anvil_benchmark);
criterion_main!(benches);