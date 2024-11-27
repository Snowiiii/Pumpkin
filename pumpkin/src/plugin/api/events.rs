use crate::entity::player::Player;

use super::PluginContext;

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

pub trait Hooks: Send + Sync + 'static {
    /// Returns an array of events that the
    fn registered_events(&self) -> Result<&'static [EventDescriptor], String> {
        Ok(&[])
    }

    /// Called when the plugin is loaded.
    fn on_player_join(
        &mut self,
        _server: &dyn PluginContext,
        _event: &Player,
    ) -> Result<bool, String> {
        Ok(false)
    }

    /// Called when the plugin is unloaded.
    fn on_player_leave(
        &mut self,
        _server: &dyn PluginContext,
        _event: &Player,
    ) -> Result<bool, String> {
        Ok(false)
    }
}
