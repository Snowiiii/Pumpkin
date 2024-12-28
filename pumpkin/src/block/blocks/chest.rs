use async_trait::async_trait;
use pumpkin_core::math::position::WorldPosition;
use pumpkin_inventory::Chest;
use pumpkin_macros::{pumpkin_block, sound};
use pumpkin_world::{block::block_registry::Block, item::item_registry::Item};

use crate::block::container::ContainerBlock;
use crate::{
    block::{block_manager::BlockActionResult, pumpkin_block::PumpkinBlock},
    entity::player::Player,
    server::Server,
};

#[pumpkin_block("minecraft:chest")]
pub struct ChestBlock;

#[async_trait]
impl PumpkinBlock for ChestBlock {
    async fn on_use<'a>(
        &self,
        block: &Block,
        player: &Player,
        location: WorldPosition,
        server: &Server,
    ) {
        self.open(block, player, location, server).await;
        player
            .world()
            .play_block_sound(sound!("block.chest.open"), location)
            .await;
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

    async fn on_close<'a>(&self, player: &Player, location: WorldPosition, server: &Server) {
        self.close(location, server, player).await;
        player
            .world()
            .play_block_sound(sound!("block.chest.close"), location)
            .await;
        // TODO: send entity updates close
    }
}
impl ContainerBlock<Chest> for ChestBlock {
    const UNIQUE: bool = false;
}
