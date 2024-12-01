/// a lookup-array for the collision shape ids of each block state
#[cfg(not(clippy))]
include!(concat!(env!("OUT_DIR"), "/block_state_collision_shapes.rs"));

#[cfg(clippy)]
pub static BLOCK_STATE_COLLISION_SHAPES: [&[u16]; 0] = [];
