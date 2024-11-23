use async_trait::async_trait;
use pumpkin_core::math::position::WorldPosition;
use pumpkin_protocol::VarInt;
use pumpkin_registry::SYNCED_REGISTRIES;
use pumpkin_world::item::item_registry::Item;
use crate::entity::player::Player;
use crate::world::block::interactive::Interactive;

pub struct JukeboxBlock;

#[async_trait]
impl Interactive for JukeboxBlock {
    async fn on_use<'a>(&self, player: &Player, location: WorldPosition) {
        // For now just stop the music at this position
        let world = &player.living_entity.entity.world;
        
        world.stop_record(location).await;
    }

    async fn on_use_with_item<'a>(&self, player: &Player, location: WorldPosition, item: &Item) {
        
        let world = &player.living_entity.entity.world;
        
        let Some(jukebox_playable) = &item.components.jukebox_playable else {
            return;
        };

        let Some(jukebox_song) = SYNCED_REGISTRIES.jukebox_song.get_index_of(jukebox_playable.song.as_str()) else {
            log::error!("Jukebox playable song not registered!");
            return;
        };
        
        //TODO: Update block state and block nbt
        
        world.play_record(jukebox_song as i32, location).await;

    }
}