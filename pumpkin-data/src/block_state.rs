#[cfg(not(clippy))]
include!(concat!(env!("OUT_DIR"), "/state_data.rs"));

#[cfg(clippy)]
pub static BLOCK_STATES: [pumpkin_core::registries::blocks::State; 0] = [];
