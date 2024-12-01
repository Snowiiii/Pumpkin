use std::collections::HashMap;
use std::sync::LazyLock;

use pumpkin_core::registries::blocks::{Block, BlockEntityKind, Shape, State};
use pumpkin_data::block::BLOCKS;
use pumpkin_data::block_entities::BLOCK_ENTITY_KINDS;
use pumpkin_data::block_shapes::BLOCK_SHAPES;
use pumpkin_data::block_state::BLOCK_STATES;
use pumpkin_data::block_state_collision_shapes::BLOCK_STATE_COLLISION_SHAPES;

/// maps numerical block ids to the corrsponding block names to efficiently find blocks by their name at runtime
static BLOCK_NAME_MAP: LazyLock<HashMap<&str, usize>> = LazyLock::new(|| {
    let mut map = HashMap::new();
    for (i, block) in BLOCKS.iter().enumerate() {
        map.insert(block.name, i);
    }
    map
});

/// get a block by hashing its string identifier (incl. namespace, so e.g. "minecraft:dirt") at runtime
///
/// if identifier is known at compile time consider using [`pumpkin_macros::block_id`] and [`self::get_block_by_id`] instead
pub fn get_block(registry_id: &str) -> Option<&'static Block> {
    let idx = BLOCK_NAME_MAP.get(registry_id)?;
    BLOCKS.get(*idx)
}

pub fn get_block_by_id(id: u16) -> Option<&'static Block> {
    BLOCKS.get(id as usize)
}

pub fn get_state_by_state_id(id: u16) -> Option<&'static State> {
    BLOCK_STATES.get(id as usize)
}

pub fn get_block_by_state_id(id: u16) -> Option<&'static Block> {
    let state = get_state_by_state_id(id)?;
    get_block_by_id(state.block_id)
}

pub fn get_block_and_state_by_state_id(id: u16) -> Option<(&'static Block, &'static State)> {
    let state = get_state_by_state_id(id)?;
    Some((get_block_by_id(state.block_id)?, state))
}

pub fn get_block_entity_kind_by_id(block_entity_kind_id: u32) -> Option<&'static BlockEntityKind> {
    BLOCK_ENTITY_KINDS.get(block_entity_kind_id as usize)
}

pub fn get_shape_by_id(collision_shape_id: u16) -> Option<&'static Shape> {
    BLOCK_SHAPES.get(collision_shape_id as usize)
}

pub fn get_shape_ids_by_block_state(block_state_id: u16) -> Option<&'static [u16]> {
    BLOCK_STATE_COLLISION_SHAPES
        .get(block_state_id as usize)
        .copied()
}

pub fn get_shapes_by_block_state(block_state_id: u16) -> Option<Vec<&'static Shape>> {
    dbg!(block_state_id);
    let ids = get_shape_ids_by_block_state(block_state_id)?;
    let mut vec = Vec::with_capacity(ids.len());
    for id in ids {
        dbg!(id);
        vec.push(get_shape_by_id(*id)?);
    }
    Some(vec)
}

#[cfg(test)]
mod tests {
    use pumpkin_macros::block_entity_id;
    use pumpkin_macros::block_id;
    use pumpkin_macros::block_state_id;

    use super::get_block;
    use super::get_block_by_id;

    #[test]
    fn test_get_block() {
        assert!(get_block("dirt").is_none());
        assert!(get_block("minecraft:dont_mind_me_i_dont_exist").is_none());

        assert!(get_block("minecraft:dirt").is_some());
        assert!(get_block("minecraft:purpur_block").is_some());
    }

    #[test]
    fn test_get_block_by_id() {
        assert!(get_block_by_id(block_id!("air")).unwrap().name == "minecraft:air");
        assert!(get_block_by_id(block_id!("minecraft:dirt")).unwrap().name == "minecraft:dirt");
        assert!(get_block_by_id(block_id!("oak_log")).unwrap().name == "minecraft:oak_log");
    }

    #[test]
    fn test_blocks() {
        let dirt_shapes =
            super::get_shape_ids_by_block_state(block_state_id!("minecraft:dirt")).unwrap();
        assert!(dirt_shapes.len() == 1);
        assert!(super::get_shape_by_id(dirt_shapes[0]).is_some());
        assert!(super::get_shapes_by_block_state(block_state_id!("green_bed")).is_some());

        assert!(super::get_block_entity_kind_by_id(block_entity_id!("chest")).is_some());
        assert!(
            super::get_block_entity_kind_by_id(block_entity_id!("minecraft:jukebox")).is_some()
        );

        assert!(super::get_block_and_state_by_state_id(block_state_id!("birch_log")).is_some());
        assert!(super::get_block_by_state_id(block_state_id!("purpur_block")).is_some());
        assert!(super::get_state_by_state_id(block_state_id!("minecraft:diamond_block")).is_some());

        assert!(super::get_shape_by_id(u16::MAX).is_none());

        assert!(super::get_block_entity_kind_by_id(u32::MAX).is_none());
        assert!(super::get_block_and_state_by_state_id(u16::MAX).is_none());
        assert!(super::get_block_by_state_id(u16::MAX).is_none());
        assert!(super::get_state_by_state_id(u16::MAX).is_none());
    }
}
