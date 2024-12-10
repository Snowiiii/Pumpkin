use async_trait::async_trait;
use pumpkin_core::math::position::WorldPosition;
use pumpkin_inventory::{Chest, OpenContainer, WindowType};
use pumpkin_macros::{pumpkin_block, sound};
use pumpkin_world::{block::block_registry::get_block, item::item_registry::Item};

use crate::{
    block::{block_manager::BlockActionResult, pumpkin_block::PumpkinBlock},
    entity::player::Player,
    server::Server,
};

#[pumpkin_block("minecraft:chest")]
pub struct ChestBlock;

#[async_trait]
impl PumpkinBlock for ChestBlock {
    async fn on_use<'a>(&self, player: &Player, _location: WorldPosition, server: &Server) {
        self.open_chest_block(player, _location, server).await;
        player
            .world()
            .play_block_sound(sound!("block.chest.open"), _location)
            .await;
    }

    async fn on_use_with_item<'a>(
        &self,
        player: &Player,
        _location: WorldPosition,
        _item: &Item,
        server: &Server,
    ) -> BlockActionResult {
        self.open_chest_block(player, _location, server).await;
        BlockActionResult::Consume
    }

    async fn on_close<'a>(&self, player: &Player, _location: WorldPosition, _server: &Server) {
        player
            .world()
            .play_block_sound(sound!("block.chest.close"), _location)
            .await;
    }
}

impl ChestBlock {
    pub async fn open_chest_block(
        &self,
        player: &Player,
        location: WorldPosition,
        server: &Server,
    ) {
        let entity_id = player.entity_id();
        // TODO: This should be a unique identifier for the each block container
        player.open_container.store(Some(2));
        {
            let mut open_containers = server.open_containers.write().await;
            if let Some(chest) = open_containers.get_mut(&2) {
                chest.add_player(entity_id);
            } else {
                let open_container = OpenContainer::new_empty_container::<Chest>(
                    entity_id,
                    Some(location),
                    get_block("minecraft:chest").cloned(),
                );
                open_containers.insert(2, open_container);
            }
        }
        player.open_container(server, WindowType::Generic9x3).await;
    }
}
