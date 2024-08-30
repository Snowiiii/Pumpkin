use level::Level;

pub mod biome;
pub mod block;
pub mod chunk;
pub mod coordinates;
pub mod dimension;
pub mod global_registry;
pub mod item;
mod level;
pub mod radial_chunk_iterator;
pub mod vector3;
pub mod vector2;
mod world_gen;

pub const WORLD_HEIGHT: usize = 384;
pub const WORLD_LOWEST_Y: i16 = -64;
pub const WORLD_MAX_Y: i16 = WORLD_HEIGHT as i16 - WORLD_LOWEST_Y.abs();
pub const DIRECT_PALETTE_BITS: u32 = 15;

pub struct World {
    pub level: Level,
    // entities, players...
}

impl World {
    pub fn load(level: Level) -> Self {
        Self { level }
    }
}
