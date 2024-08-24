use level::Level;

pub mod chunk;
pub mod coordinates;
pub mod dimension;
pub const WORLD_HEIGHT: usize = 384;
pub const WORLD_Y_START_AT: i32 = -64;
pub const DIRECT_PALETTE_BITS: u32 = 15;
pub mod block;
pub mod global_registry;
pub mod item;
mod level;
pub mod radial_chunk_iterator;
pub mod vector3;

pub struct World {
    pub level: Level,
    // entities, players...
}

impl World {
    pub fn load(level: Level) -> Self {
        Self { level }
    }
}
