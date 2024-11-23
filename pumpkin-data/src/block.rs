// clippy should skip these
#[cfg(not(clippy))]
pumpkin_macros::include_blocks! {}

#[cfg(clippy)]
pub static BLOCKS: [pumpkin_core::registries::blocks::Block; 0] = [];
#[cfg(clippy)]
pub static BLOCK_STATES: [pumpkin_core::registries::blocks::State; 0] = [];
