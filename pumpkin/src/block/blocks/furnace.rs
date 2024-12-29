use crate::block::block_manager::BlockActionResult;
use crate::entity::player::Player;
use async_trait::async_trait;
use pumpkin_core::math::position::WorldPosition;
use pumpkin_inventory::Furnace;
use pumpkin_macros::pumpkin_block;
use pumpkin_world::block::block_registry::Block;
use pumpkin_world::item::item_registry::Item;

use crate::block::container::ContainerBlock;
use crate::{block::pumpkin_block::PumpkinBlock, server::Server};

#[pumpkin_block("minecraft:furnace")]
pub struct FurnaceBlock;

#[async_trait]
impl PumpkinBlock for FurnaceBlock {
    async fn on_use<'a>(
        &self,
        block: &Block,
        player: &Player,
        _location: WorldPosition,
        server: &Server,
    ) {
        self.open(block, player, _location, server).await;
    }

    async fn on_use_with_item<'a>(
        &self,
        block: &Block,
        player: &Player,
        location: WorldPosition,
        _item: &Item,
        server: &Server,
    ) -> BlockActionResult {
        self.open(block, player, location, server).await;
        BlockActionResult::Consume
    }

    async fn on_broken<'a>(
        &self,
        _block: &Block,
        player: &Player,
        location: WorldPosition,
        server: &Server,
    ) {
        self.destroy(location, server, player).await;
    }
}

impl ContainerBlock<Furnace> for FurnaceBlock {
    const UNIQUE: bool = false;
}
