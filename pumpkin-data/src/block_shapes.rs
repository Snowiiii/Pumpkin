#[cfg(not(clippy))]
include!(concat!(env!("OUT_DIR"), "/block_shapes.rs"));

#[cfg(clippy)]
pub static BLOCK_SHAPES: [pumpkin_core::registries::blocks::Shape; 0] = [];
