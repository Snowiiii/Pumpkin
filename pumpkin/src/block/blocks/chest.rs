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
        // TODO: drop all items and close screen if different player breaks block
        let entity_id = player.entity_id();
        if let Some(container_id) = server.get_container_id(location, block.clone()).await {
            let mut open_containers = server.open_containers.write().await;
            if let Some(container) = open_containers.get_mut(&u64::from(container_id)) {
                log::info!("Good ct ID: {}", container_id);

                container.on_destroy().await;

                container.remove_player(entity_id);
                player.open_container.store(None);
            }
        }
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
        let entity_id = player.entity_id();
        if let Some(container_id) = server.get_container_id(location, block.clone()).await {
            let mut open_containers = server.open_containers.write().await;
            if let Some(container) = open_containers.get_mut(&u64::from(container_id)) {
                log::info!("Good chest ID: {}", container_id);
                container.add_player(entity_id);
                player.open_container.store(Some(container_id.into()));
            }
            // drop(open_containers);
        } else {
            let mut open_containers = server.open_containers.write().await;

            let new_id = server.new_container_id();
            log::info!("New chest ID: {}", new_id);

            let open_container = OpenContainer::new_empty_container::<Chest>(
                entity_id,
                Some(location),
                Some(block.clone()),
            );
            open_containers.insert(new_id.into(), open_container);
            player.open_container.store(Some(new_id.into()));
            // drop(open_containers);
        }
        player.open_container(server, WindowType::Generic9x3).await;
    }
}
