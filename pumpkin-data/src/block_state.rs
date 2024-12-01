/// an array of all block states, indexed by their id
#[cfg(not(clippy))]
include!(concat!(env!("OUT_DIR"), "/block_states.rs"));

#[cfg(clippy)]
pub static BLOCK_STATES: [pumpkin_core::registries::blocks::State; 0] = [];
