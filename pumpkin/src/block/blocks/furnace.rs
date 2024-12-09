use crate::block::block_manager::BlockActionResult;
use crate::entity::player::Player;
use async_trait::async_trait;
use pumpkin_core::math::position::WorldPosition;
use pumpkin_inventory::Furnace;
use pumpkin_inventory::{OpenContainer, WindowType};
use pumpkin_macros::pumpkin_block;
use pumpkin_world::block::block_registry::Block;
use pumpkin_world::item::item_registry::Item;

use crate::{block::pumpkin_block::PumpkinBlock, server::Server};

#[pumpkin_block("minecraft:furnace")]
pub struct FurnaceBlock;

#[async_trait]
impl PumpkinBlock for FurnaceBlock {
    async fn on_use<'a>(
        &self,
        block: &Block,
        player: &Player,
        _location: WorldPosition,
        server: &Server,
    ) {
        self.open_furnace_screen(block, player, _location, server)
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
        self.open_furnace_screen(block, player, _location, server)
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
}

impl FurnaceBlock {
    pub async fn open_furnace_screen(
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
                log::info!("Good furnance ID: {}", container_id);
                container.add_player(entity_id);
                player.open_container.store(Some(container_id.into()));
            }
            // drop(open_containers);
        } else {
            let mut open_containers = server.open_containers.write().await;

            let new_id = server.new_container_id();
            log::info!("New furnace ID: {}", new_id);

            let open_container = OpenContainer::new_empty_container::<Furnace>(
                entity_id,
                Some(location),
                Some(block.clone()),
            );
            open_containers.insert(new_id.into(), open_container);
            player.open_container.store(Some(new_id.into()));
            // drop(open_containers);
        }
        player.open_container(server, WindowType::Furnace).await;
    }
}
