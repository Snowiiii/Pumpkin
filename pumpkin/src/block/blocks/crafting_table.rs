use crate::block::block_manager::BlockActionResult;
use crate::block::pumpkin_block::PumpkinBlock;
use crate::entity::player::Player;
use crate::server::Server;
use async_trait::async_trait;
use pumpkin_core::math::position::WorldPosition;
use pumpkin_inventory::{CraftingTable, OpenContainer, WindowType};
use pumpkin_macros::pumpkin_block;
use pumpkin_world::{block::block_registry::Block, item::item_registry::Item};

#[pumpkin_block("minecraft:crafting_table")]
pub struct CraftingTableBlock;

#[async_trait]
impl PumpkinBlock for CraftingTableBlock {
    async fn on_use<'a>(
        &self,
        block: &Block,
        player: &Player,
        _location: WorldPosition,
        server: &Server,
    ) {
        self.open_crafting_screen(block, player, _location, server)
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
        self.open_crafting_screen(block, player, _location, server)
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
            if let Some(ender_chest) = open_containers.get_mut(&u64::from(container_id)) {
                log::info!("Good ct ID: {}", container_id);

                ender_chest.on_destroy().await;

                ender_chest.remove_player(entity_id);
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
        container: &OpenContainer,
    ) {
        log::info!("On Close CT");
        let entity_id = player.entity_id();

        for player_id in container.all_player_ids() {
            if entity_id == player_id {
                container.on_destroy().await;
            }
        }

        // TODO: should re-add all items to player or drop?
    }
}

impl CraftingTableBlock {
    pub async fn open_crafting_screen(
        &self,
        block: &Block,
        player: &Player,
        location: WorldPosition,
        server: &Server,
    ) {
        //TODO: Adjust /craft command to real crafting table
        let entity_id = player.entity_id();
        let mut open_containers = server.open_containers.write().await;
        let mut id_to_use = -1;

        for (id, container) in open_containers.iter() {
            if let Some(a_block) = container.get_block() {
                if a_block.id == block.id && container.all_player_ids().is_empty() {
                    id_to_use = *id as i64;
                }
            }
        }

        if id_to_use == -1 {
            let new_id = server.new_container_id();

            log::info!("New ct ID: {}", new_id);

            let open_container = OpenContainer::new_empty_container::<CraftingTable>(
                entity_id,
                Some(location),
                Some(block.clone()),
            );

            open_containers.insert(new_id.into(), open_container);

            player.open_container.store(Some(new_id.into()));
            // drop(open_containers);
        } else {
            log::info!("Using previous ct ID: {}", id_to_use);
            if let Some(ender_chest) = open_containers.get_mut(&(id_to_use as u64)) {
                ender_chest.add_player(entity_id);
                player
                    .open_container
                    .store(Some(id_to_use.try_into().unwrap()));
            }
        }
        drop(open_containers);
        player
            .open_container(server, WindowType::CraftingTable)
            .await;
    }
}
