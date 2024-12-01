/// an array of all block entities, indexed by their id
#[cfg(not(clippy))]
include!(concat!(env!("OUT_DIR"), "/block_entities.rs"));

#[cfg(clippy)]
pub static BLOCK_ENTITY_KINDS: [pumpkin_core::registries::blocks::BlockEntityKind; 0] = [];
