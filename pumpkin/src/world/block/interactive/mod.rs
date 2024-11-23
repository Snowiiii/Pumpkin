pub mod jukebox;

use std::collections::HashMap;
use std::sync::Arc;
use async_trait::async_trait;
use pumpkin_core::math::position::WorldPosition;
use pumpkin_world::block::block_registry::Block;
use pumpkin_world::item::item_registry::Item;
use crate::entity::player::Player;
use crate::world::block::interactive::jukebox::JukeboxBlock;

pub fn default_interactive_block_manager() -> Arc<InteractiveBlockManager>{
    let mut manager = InteractiveBlockManager::default();
    
    manager.register("minecraft:jukebox", Box::new(JukeboxBlock));
    
    Arc::new(manager)
}

#[derive(Default)]
pub struct InteractiveBlockManager {
    interactable: HashMap<String, Box<dyn Interactive>>
}

impl InteractiveBlockManager {
    fn register(&mut self, name: &str, interactive: Box<dyn Interactive>) {
        self.interactable.insert(name.to_string(), interactive);
    }
    
    pub async fn on_use(&self, block: &Block, player: &Player, location: WorldPosition) {
        let interactable = self.interactable.get(block.name.as_str());
        if let Some(interactable) = interactable {
            interactable.on_use(player, location).await;
        }
    }

    pub async fn on_use_with_item(&self, block: &Block, player: &Player, location: WorldPosition, item: &Item) {
        let interactable = self.interactable.get(block.name.as_str());
        if let Some(interactable) = interactable {
            interactable.on_use_with_item(player, location, item).await;
        }
    }
}

#[async_trait]
pub trait Interactive: Send + Sync {
    async fn on_use<'a>(&self, player: &Player, location: WorldPosition);
    async fn on_use_with_item<'a>(&self, player: &Player, location: WorldPosition, item: &Item);
}