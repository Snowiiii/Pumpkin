pub mod chunk;
pub mod dimension;
pub const WORLD_HEIGHT: usize = 384;
pub const WORLD_Y_START_AT: i32 = -64; // TODO: make sure where it actually starts at (i think its at -64 but not sure)
pub const DIRECT_PALETTE_BITS: u32 = 15;
mod world;
mod block_registry;
