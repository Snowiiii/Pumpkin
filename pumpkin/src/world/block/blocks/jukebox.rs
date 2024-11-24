use crate::entity::player::Player;
use crate::server::Server;
use crate::world::block::block_manager::{BlockManager, BlockUpdateConsumer, InteractiveBlock};
use crate::world::block::pumpkin_block::{BlockMetadata, PumpkinBlock};
use async_trait::async_trait;
use pumpkin_core::math::position::WorldPosition;
use pumpkin_registry::SYNCED_REGISTRIES;
use pumpkin_world::item::item_registry::Item;
use std::sync::Arc;

pub struct JukeboxBlock;

impl BlockMetadata for JukeboxBlock {
    const NAMESPACE: &'static str = "minecraft";
    const ID: &'static str = "jukebox";
}

impl PumpkinBlock for JukeboxBlock {
    fn register(self, block_manager: &mut BlockManager) {
        let block = Arc::new(self);
        block_manager.register_block_interactable(block.clone());
        block_manager.register_with_block_update_consumer(block);
    }
}

#[async_trait]
impl InteractiveBlock for JukeboxBlock {
    async fn on_use<'a>(&self, player: &Player, location: WorldPosition, _server: &Server) {
        // For now just stop the music at this position
        let world = &player.living_entity.entity.world;

        world.stop_record(location).await;
    }

    async fn on_use_with_item<'a>(
        &self,
        player: &Player,
        location: WorldPosition,
        item: &Item,
        _server: &Server,
    ) {
        let world = &player.living_entity.entity.world;

        let Some(jukebox_playable) = &item.components.jukebox_playable else {
            return;
        };

        let Some(jukebox_song) = SYNCED_REGISTRIES
            .jukebox_song
            .get_index_of(jukebox_playable.song.as_str())
        else {
            log::error!("Jukebox playable song not registered!");
            return;
        };

        //TODO: Update block state and block nbt

        world.play_record(jukebox_song as i32, location).await;
    }
}

#[async_trait]
impl BlockUpdateConsumer for JukeboxBlock {
    async fn on_placed<'a>(&self, _player: &Player, _location: WorldPosition, _server: &Server) {
        todo!()
    }
}
