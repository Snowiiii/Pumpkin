use async_trait::async_trait;

use super::types::player::PlayerEvent;
use super::context::Context;

#[derive(Eq, PartialEq, Ord, PartialOrd)]
pub enum EventPriority {
    Highest,
    High,
    Normal,
    Low,
    Lowest,
}

pub struct EventDescriptor {
    pub name: &'static str,
    pub priority: EventPriority,
    pub blocking: bool,
}

#[async_trait]
pub trait Hooks: Send + Sync + 'static {
    /// Returns an array of events that the
    fn registered_events(&self) -> Result<&'static [EventDescriptor], String> {
        Ok(&[])
    }

    /// Called when a player joins the server.
    async fn on_player_join(
        &mut self,
        _server: &Context,
        _player: &PlayerEvent,
    ) -> Result<bool, String> {
        Ok(false)
    }

    /// Called when a player leaves the server.
    async fn on_player_leave(
        &mut self,
        _server: &Context,
        _player: &PlayerEvent,
    ) -> Result<bool, String> {
        Ok(false)
    }
}
