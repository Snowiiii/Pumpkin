use async_trait::async_trait;
use pumpkin_core::math::position::WorldPosition;
use pumpkin_inventory::{Chest, OpenContainer, WindowType};
use pumpkin_macros::{pumpkin_block, sound};
use pumpkin_protocol::{client::play::CBlockAction, codec::var_int::VarInt};
use pumpkin_world::{
    block::block_registry::{get_block, Block},
    item::item_registry::Item,
};

use crate::{
    block::{block_manager::BlockActionResult, pumpkin_block::PumpkinBlock},
    entity::player::Player,
    server::Server,
};

#[derive(PartialEq)]
pub enum ChestState {
    IsOpened,
    IsClosed,
}

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
        location: WorldPosition,
        server: &Server,
        container: &mut OpenContainer,
    ) {
        container.remove_player(player.entity_id());

        self.play_chest_action(container, player, location, server, ChestState::IsClosed)
            .await;
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

        if let Some(container_id) = server.get_container_id(location, block.clone()).await {
            let open_containers = server.open_containers.read().await;
            if let Some(container) = open_containers.get(&u64::from(container_id)) {
                self.play_chest_action(container, player, location, server, ChestState::IsOpened)
                    .await;
            }
        }
    }

    pub async fn play_chest_action(
        &self,
        container: &OpenContainer,
        player: &Player,
        location: WorldPosition,
        server: &Server,
        state: ChestState,
    ) {
        let num_players = container.get_number_of_players() as u8;
        if state == ChestState::IsClosed && num_players == 0 {
            player
                .world()
                .play_block_sound(sound!("block.chest.close"), location)
                .await;
        } else if state == ChestState::IsOpened && num_players == 1 {
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
                    num_players,
                    VarInt(e.id.into()),
                ))
                .await;
        }
    }
}
