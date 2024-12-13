use crate::block::block_manager::BlockActionResult;
use crate::entity::player::Player;
use async_trait::async_trait;
use pumpkin_core::math::position::WorldPosition;
use pumpkin_inventory::Furnace;
use pumpkin_inventory::{OpenContainer, WindowType};
use pumpkin_macros::pumpkin_block;
use pumpkin_world::block::block_registry::get_block;
use pumpkin_world::item::item_registry::Item;

use crate::{block::pumpkin_block::PumpkinBlock, server::Server};

#[pumpkin_block("minecraft:furnace")]
pub struct FurnaceBlock;

#[async_trait]
impl PumpkinBlock for FurnaceBlock {
    async fn on_use<'a>(&self, player: &Player, _location: WorldPosition, server: &Server) {
        self.open_furnace_screen(player, _location, server).await;
    }

    async fn on_use_with_item<'a>(
        &self,
        player: &Player,
        _location: WorldPosition,
        _item: &Item,
        server: &Server,
    ) -> BlockActionResult {
        self.open_furnace_screen(player, _location, server).await;
        BlockActionResult::Consume
    }
}

impl FurnaceBlock {
    pub async fn open_furnace_screen(
        &self,
        player: &Player,
        location: WorldPosition,
        server: &Server,
    ) {
        //TODO: Adjust /craft command to real crafting table
        let entity_id = player.entity_id();
        player.open_container.store(Some(3));
        {
            let mut open_containers = server.open_containers.write().await;
            if let Some(ender_chest) = open_containers.get_mut(&3) {
                ender_chest.add_player(entity_id);
            } else {
                let open_container = OpenContainer::new_empty_container::<Furnace>(
                    entity_id,
                    Some(location),
                    get_block("minecraft:furnace").cloned(),
                );
                open_containers.insert(3, open_container);
            }
        }
        player.open_container(server, WindowType::Furnace).await;
    }
}
