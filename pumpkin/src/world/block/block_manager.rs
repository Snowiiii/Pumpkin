use crate::entity::player::Player;
use crate::server::Server;
use crate::world::block::pumpkin_block::PumpkinBlock;
use async_trait::async_trait;
use pumpkin_core::math::position::WorldPosition;
use pumpkin_world::block::block_registry::Block;
use pumpkin_world::item::item_registry::Item;
use std::collections::HashMap;
use std::sync::Arc;

#[derive(Default)]
pub struct BlockManager {
    interactable: HashMap<String, Arc<dyn InteractiveBlock>>,
    with_block_update_consumer: HashMap<String, Arc<dyn BlockUpdateConsumer>>,
}

impl BlockManager {
    pub fn register<T: PumpkinBlock>(&mut self, block: T) {
        T::register(block, self);
    }

    pub fn register_block_interactable<T: PumpkinBlock + 'static + InteractiveBlock>(
        &mut self,
        block: Arc<T>,
    ) {
        let key = format!("{}:{}", T::NAMESPACE, T::ID);
        self.interactable.insert(key, block);
    }

    pub fn register_with_block_update_consumer<T: PumpkinBlock + 'static + BlockUpdateConsumer>(
        &mut self,
        block: Arc<T>,
    ) {
        let key = format!("{}:{}", T::NAMESPACE, T::ID);
        self.with_block_update_consumer.insert(key, block);
    }

    pub async fn on_use(
        &self,
        block: &Block,
        player: &Player,
        location: WorldPosition,
        server: &Server,
    ) {
        let interactable = self.interactable.get(block.name.as_str());
        if let Some(interactable) = interactable {
            interactable.on_use(player, location, server).await;
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
        let interactable = self.interactable.get(block.name.as_str());
        if let Some(interactable) = interactable {
            interactable
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
        let with_block_update_consumer = self.with_block_update_consumer.get(block.name.as_str());
        if let Some(interactable) = with_block_update_consumer {
            interactable.on_placed(player, location, server).await;
        }
    }
}

#[async_trait]
pub trait InteractiveBlock: Sync + Send {
    async fn on_use<'a>(&self, player: &Player, location: WorldPosition, server: &Server);
    async fn on_use_with_item<'a>(
        &self,
        player: &Player,
        location: WorldPosition,
        item: &Item,
        server: &Server,
    );
}

#[async_trait]
pub trait BlockUpdateConsumer: Sync + Send {
    async fn on_placed<'a>(&self, player: &Player, location: WorldPosition, server: &Server);
}
