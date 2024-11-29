use crate::block::pumpkin_block::{BlockMetadata, PumpkinBlock};
use crate::entity::player::Player;
use crate::server::Server;
use pumpkin_core::math::position::WorldPosition;
use pumpkin_world::block::block_registry::Block;
use pumpkin_world::item::item_registry::Item;
use std::collections::HashMap;
use std::sync::Arc;

#[derive(Default)]
pub struct BlockManager {
    blocks: HashMap<String, Arc<dyn PumpkinBlock>>,
}

impl BlockManager {
    pub fn register<T: PumpkinBlock + BlockMetadata + 'static>(&mut self, block: T) {
        self.blocks
            .insert(block.name().to_string(), Arc::new(block));
    }

    pub async fn on_use(
        &self,
        block: &Block,
        player: &Player,
        location: WorldPosition,
        server: &Server,
    ) {
        let pumpkin_block = self.get_pumpkin_block(block);
        if let Some(pumpkin_block) = pumpkin_block {
            pumpkin_block.on_use(player, location, server).await;
        }
    }

    pub async fn on_use_with_item(
        &self,
        block: &Block,
        player: &Player,
        location: WorldPosition,
        item: &Item,
        server: &Server,
    ) {
        let pumpkin_block = self.get_pumpkin_block(block);
        if let Some(pumpkin_block) = pumpkin_block {
            pumpkin_block
                .on_use_with_item(player, location, item, server)
                .await;
        }
    }

    pub async fn on_placed(
        &self,
        block: &Block,
        player: &Player,
        location: WorldPosition,
        server: &Server,
    ) {
        let pumpkin_block = self.get_pumpkin_block(block);
        if let Some(pumpkin_block) = pumpkin_block {
            pumpkin_block.on_placed(player, location, server).await;
        }
    }

    #[must_use]
    pub fn get_pumpkin_block(&self, block: &Block) -> Option<&Arc<dyn PumpkinBlock>> {
        self.blocks
            .get(format!("minecraft:{}", block.name).as_str())
    }
}
