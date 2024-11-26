use crate::{entity::player::Player, world::World};

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

pub trait Event {
    fn handle(&mut self, server: &dyn PluginContext, hooks: &mut dyn Hooks) -> Result<(), String>;
    fn is_cancelled(&self) -> bool;
    fn cancel(&mut self) -> Result<(), String>;
}

pub struct PlayerConnection<'a> {
    pub player: &'a Player,
    pub world: &'a World,
    pub is_cancelled: bool,
    pub is_join: bool,
}

impl PlayerConnectionEvent for PlayerConnection<'_> {
    fn get_player(&self) -> &Player {
        self.player
    }

    fn get_world(&self) -> &World {
        self.world
    }
}

impl Event for PlayerConnection<'_> {
    fn handle(&mut self, server: &dyn PluginContext, hooks: &mut dyn Hooks) -> Result<(), String> {
        if self.is_cancelled {
            return Ok(());
        }

        if self.is_join {
            hooks.on_player_join(server, self)
        } else {
            hooks.on_player_leave(server, self)
        }
    }

    fn is_cancelled(&self) -> bool {
        self.is_cancelled
    }

    fn cancel(&mut self) -> Result<(), String> {
        self.is_cancelled = true;
        Ok(())
    }
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
        _event: &mut PlayerConnection,
    ) -> Result<(), String> {
        Ok(())
    }

    /// Called when the plugin is unloaded.
    fn on_player_leave(
        &mut self,
        _server: &dyn PluginContext,
        _event: &mut PlayerConnection,
    ) -> Result<(), String> {
        Ok(())
    }
}

pub trait PlayerConnectionEvent {
    fn get_player(&self) -> &Player;
    fn get_world(&self) -> &World;
}
