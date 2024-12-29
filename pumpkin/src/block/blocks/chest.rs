use async_trait::async_trait;
use pumpkin_core::math::position::WorldPosition;
use pumpkin_inventory::Chest;
use pumpkin_macros::{pumpkin_block, sound};
use pumpkin_protocol::client::play::CBlockAction;
use pumpkin_protocol::codec::var_int::VarInt;
use pumpkin_world::{
    block::block_registry::{get_block, Block},
    item::item_registry::Item,
};

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
        self.play_chest_action(player, location, server).await;
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
        self.play_chest_action(player, location, server).await;
    }
}

impl ChestBlock {
    pub async fn play_chest_action(
        &self,
        player: &Player,
        location: WorldPosition,
        server: &Server,
    ) {
        let num_players = server
            .open_containers
            .read()
            .await
            .get_by_location(&location)
            .map(|container| container.all_player_ids().len())
            .unwrap_or_default();
        if num_players == 0 {
            player
                .world()
                .play_block_sound(sound!("block.chest.close"), location)
                .await;
        } else if num_players == 1 {
            player
                .world()
                .play_block_sound(sound!("block.chest.open"), location)
                .await;
        }

        if let Some(e) = get_block("minecraft:chest").cloned() {
            server
                .broadcast_packet_all(&CBlockAction::new(
                    &location,
                    1,
                    num_players as u8,
                    VarInt(e.id.into()),
                ))
                .await;
        }
    }
}

impl ContainerBlock<Chest> for ChestBlock {
    const UNIQUE: bool = false;
}
