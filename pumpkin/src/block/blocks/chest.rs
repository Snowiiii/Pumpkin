use async_trait::async_trait;
use pumpkin_core::math::position::WorldPosition;
use pumpkin_inventory::{Chest, OpenContainer, WindowType};
use pumpkin_macros::{pumpkin_block, sound};
use pumpkin_world::{block::block_registry::Block, item::item_registry::Item};

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
        _location: WorldPosition,
        server: &Server,
    ) {
        self.open_chest_block(block, player, _location, server)
            .await;
        player
            .world()
            .play_block_sound(sound!("block.chest.open"), _location)
            .await;
    }

    async fn on_use_with_item<'a>(
        &self,
        block: &Block,
        player: &Player,
        _location: WorldPosition,
        _item: &Item,
        server: &Server,
    ) -> BlockActionResult {
        self.open_chest_block(block, player, _location, server)
            .await;
        BlockActionResult::Consume
    }

    async fn on_broken<'a>(
        &self,
        block: &Block,
        player: &Player,
        location: WorldPosition,
        server: &Server,
    ) {
        super::standard_on_broken_with_container(block, player, location, server).await;
    }

    async fn on_close<'a>(
        &self,
        _block: &Block,
        player: &Player,
        _location: WorldPosition,
        _server: &Server,
        _container: &OpenContainer,
    ) {
        player
            .world()
            .play_block_sound(sound!("block.chest.close"), _location)
            .await;
        // TODO: send entity updates close
    }
}

impl ChestBlock {
    pub async fn open_chest_block(
        &self,
        block: &Block,
        player: &Player,
        location: WorldPosition,
        server: &Server,
    ) {
        // TODO: shouldn't Chest and window type be constrained together to avoid errors?
        super::standard_open_container::<Chest>(
            block,
            player,
            location,
            server,
            WindowType::Generic9x3,
        )
        .await;
        // TODO: send entity updates open
    }
}
