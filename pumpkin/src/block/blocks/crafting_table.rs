use crate::block::pumpkin_block::PumpkinBlock;
use crate::entity::player::Player;
use crate::server::Server;
use async_trait::async_trait;
use pumpkin_core::math::position::WorldPosition;
use pumpkin_inventory::{CraftingTable, OpenContainer, WindowType};
use pumpkin_macros::pumpkin_block;
use pumpkin_world::item::item_registry::Item;

#[pumpkin_block("minecraft:crafting_table")]
pub struct CraftingTableBlock;

#[async_trait]
impl PumpkinBlock for CraftingTableBlock {
    async fn on_use<'a>(&self, player: &Player, _location: WorldPosition, server: &Server) {
        self.open_crafting_screen(player, server).await;
    }

    async fn on_use_with_item<'a>(
        &self,
        player: &Player,
        _location: WorldPosition,
        _item: &Item,
        server: &Server,
    ) {
        self.open_crafting_screen(player, server).await;
    }
}

impl CraftingTableBlock {
    pub async fn open_crafting_screen(&self, player: &Player, server: &Server) {
        //TODO: Adjust /craft command to real crafting table
        let entity_id = player.entity_id();
        player.open_container.store(Some(1));
        {
            let mut open_containers = server.open_containers.write().await;
            if let Some(ender_chest) = open_containers.get_mut(&1) {
                ender_chest.add_player(entity_id);
            } else {
                let open_container = OpenContainer::new_empty_container::<CraftingTable>(entity_id);
                open_containers.insert(1, open_container);
            }
        }
        player
            .open_container(server, WindowType::CraftingTable)
            .await;
    }
}
