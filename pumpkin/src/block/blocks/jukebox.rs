use crate::block::pumpkin_block::{BlockMetadata, PumpkinBlock};
use crate::entity::player::Player;
use crate::server::Server;
use async_trait::async_trait;
use pumpkin_core::math::position::WorldPosition;
use pumpkin_registry::SYNCED_REGISTRIES;
use pumpkin_world::item::item_registry::Item;

pub struct JukeboxBlock;

impl BlockMetadata for JukeboxBlock {
    const NAMESPACE: &'static str = "minecraft";
    const ID: &'static str = "jukebox";
}

#[async_trait]
impl PumpkinBlock for JukeboxBlock {
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

        let Some(song) = jukebox_playable.song.split(':').nth(1) else {
            return;
        };

        let Some(jukebox_song) = SYNCED_REGISTRIES.jukebox_song.get_index_of(song) else {
            log::error!("Jukebox playable song not registered!");
            return;
        };

        //TODO: Update block state and block nbt

        world.play_record(jukebox_song as i32, location).await;
    }
}
