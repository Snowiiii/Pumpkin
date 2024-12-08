use crate::block::block_manager::BlockActionResult;
use crate::entity::player::Player;
use crate::server::Server;
use async_trait::async_trait;
use pumpkin_core::math::position::WorldPosition;
use pumpkin_world::item::item_registry::Item;

pub trait BlockMetadata {
    const NAMESPACE: &'static str;
    const ID: &'static str;
    fn name(&self) -> String {
        format!("{}:{}", Self::NAMESPACE, Self::ID)
    }
}

#[async_trait]
pub trait PumpkinBlock: Send + Sync {
    async fn on_use<'a>(&self, _player: &Player, _location: WorldPosition, _server: &Server) {}
    async fn on_use_with_item<'a>(
        &self,
        _player: &Player,
        _location: WorldPosition,
        _item: &Item,
        _server: &Server,
    ) -> BlockActionResult {
        BlockActionResult::Continue
    }

    async fn on_placed<'a>(&self, _player: &Player, _location: WorldPosition, _server: &Server) {}

    async fn on_broken<'a>(&self, _player: &Player, _location: WorldPosition, _server: &Server) {}

    async fn on_close<'a>(&self, _player: &Player, _location: WorldPosition, _server: &Server) {}
}
