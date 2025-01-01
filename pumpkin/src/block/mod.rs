use block_properties_manager::BlockPropertiesManager;
use blocks::chest::ChestBlock;
use blocks::furnace::FurnaceBlock;

use crate::block::block_manager::BlockManager;
use crate::block::blocks::crafting_table::CraftingTableBlock;
use crate::block::blocks::jukebox::JukeboxBlock;
use std::sync::Arc;

pub mod block_manager;
pub mod block_properties_manager;
mod blocks;
mod properties;
pub mod pumpkin_block;

#[must_use]
pub fn default_block_manager() -> Arc<BlockManager> {
    let mut manager = BlockManager::default();

    manager.register(JukeboxBlock);
    manager.register(CraftingTableBlock);
    manager.register(FurnaceBlock);
    manager.register(ChestBlock);

    Arc::new(manager)
}

#[must_use]
pub fn default_block_properties_manager() -> Arc<BlockPropertiesManager> {
    let mut manager = BlockPropertiesManager::default();

    manager.build_properties_registry();

    Arc::new(manager)
}
