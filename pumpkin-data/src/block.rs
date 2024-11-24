#[cfg(not(clippy))]
include!(concat!(env!("OUT_DIR"), "/block_data.rs"));

#[cfg(clippy)]
pub static BLOCKS: [pumpkin_core::registries::blocks::Block; 0] = [];
