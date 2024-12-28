use crate::block::block_manager::BlockActionResult;
use crate::block::container::ContainerBlock;
use crate::block::pumpkin_block::PumpkinBlock;
use crate::entity::player::Player;
use crate::server::Server;
use async_trait::async_trait;
use pumpkin_core::math::position::WorldPosition;
use pumpkin_inventory::CraftingTable;
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
        self.open(block, player, _location, server).await;
    }

    async fn on_use_with_item<'a>(
        &self,
        block: &Block,
        player: &Player,
        _location: WorldPosition,
        _item: &Item,
        server: &Server,
    ) -> BlockActionResult {
        self.open(block, player, _location, server).await;
        BlockActionResult::Consume
    }

    async fn on_broken<'a>(
        &self,
        block: &Block,
        player: &Player,
        location: WorldPosition,
        server: &Server,
    ) {
        self.on_broken(block, player, location, server).await;
    }

    async fn on_close<'a>(&self, player: &Player, location: WorldPosition, server: &Server) {
        self.close(location, server, player).await;

        // TODO: items should be re-added to player inventory or dropped dependending on if they are in movement.
        // TODO: unique containers should be implemented as a separate stack internally (optimizes large player servers for example)
        // TODO: ephemeral containers (crafting tables) might need to be separate data structure than stored (ender chest)
    }
}

impl ContainerBlock<CraftingTable> for CraftingTableBlock {
    const UNIQUE: bool = true;
}
