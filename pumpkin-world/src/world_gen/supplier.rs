use enum_dispatch::enum_dispatch;

use crate::biome::Biome;

#[derive(Clone)]
#[enum_dispatch]
pub enum BiomeSupplier {
    Static(StaticBiomeSupplier),
}

#[enum_dispatch(BiomeSupplier)]
pub trait BiomeSupplierImpl {
    fn biome(&self, x: i32, y: i32, z: i32, noise: &MultiNoiseSampler) -> Biome;
}

#[derive(Clone)]
pub struct StaticBiomeSupplier {}
impl BiomeSupplierImpl for StaticBiomeSupplier {
    fn biome(&self, _x: i32, _y: i32, _z: i32, _noise: &MultiNoiseSampler) -> Biome {
        Biome::Plains
    }
}

pub struct MultiNoiseSampler {
    
}
