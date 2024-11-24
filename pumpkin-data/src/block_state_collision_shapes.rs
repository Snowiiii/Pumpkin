#[cfg(not(clippy))]
include!(concat!(env!("OUT_DIR"), "/state_collision_shape_data.rs"));

#[cfg(clippy)]
pub static BLOCK_STATE_COLLISION_SHAPES: [&[u16]; 0] = [];
