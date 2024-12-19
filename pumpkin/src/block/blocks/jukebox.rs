use crate::block::block_manager::BlockActionResult;
use crate::block::pumpkin_block::PumpkinBlock;
use crate::entity::player::Player;
use crate::server::Server;
use async_trait::async_trait;
use pumpkin_core::math::position::WorldPosition;
use pumpkin_macros::pumpkin_block;
use pumpkin_registry::SYNCED_REGISTRIES;
use pumpkin_world::block::block_registry::Block;
use pumpkin_world::item::item_registry::Item;

#[pumpkin_block("minecraft:jukebox")]
pub struct JukeboxBlock;

#[async_trait]
impl PumpkinBlock for JukeboxBlock {
    async fn on_use<'a>(
        &self,
        _block: &Block,
        player: &Player,
        location: WorldPosition,
        _server: &Server,
    ) {
        // For now just stop the music at this position
        let world = &player.living_entity.entity.world;

        world.stop_record(location).await;
    }

    async fn on_use_with_item<'a>(
        &self,
        _block: &Block,
        player: &Player,
        location: WorldPosition,
        item: &Item,
        _server: &Server,
    ) -> BlockActionResult {
        let world = &player.living_entity.entity.world;

        let Some(jukebox_playable) = &item.components.jukebox_playable else {
            return BlockActionResult::Continue;
        };

        let Some(song) = jukebox_playable.song.split(':').nth(1) else {
            return BlockActionResult::Continue;
        };

        let Some(jukebox_song) = SYNCED_REGISTRIES.jukebox_song.get_index_of(song) else {
            log::error!("Jukebox playable song not registered!");
            return BlockActionResult::Continue;
        };

        //TODO: Update block state and block nbt

        world.play_record(jukebox_song as i32, location).await;

        BlockActionResult::Consume
    }

    async fn on_broken<'a>(
        &self,
        _block: &Block,
        player: &Player,
        location: WorldPosition,
        _server: &Server,
    ) {
        // For now just stop the music at this position
        let world = &player.living_entity.entity.world;

        world.stop_record(location).await;
    }
}
