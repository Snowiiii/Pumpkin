use crate::{entity::player::Player, world::World};

use super::PluginContext;

pub trait Hooks: Send + Sync + 'static {
    /// Returns an array of events that the 
    fn registered_events(&self) -> Result<&'static[&'static str], String> {
        Ok(&[])
    }

    /// Called when the plugin is loaded.
    fn on_player_join(&mut self, _server: &dyn PluginContext, _event: &dyn PlayerConnectionEvent) -> Result<(), String> {
        Ok(())
    }

    /// Called when the plugin is unloaded.
    fn on_player_leave(&mut self, _server: &dyn PluginContext, _event: &dyn PlayerConnectionEvent) -> Result<(), String> {
        Ok(())
    }
}

pub trait PlayerConnectionEvent {
    fn get_player(&self) -> &Player;
    fn get_world(&self) -> &World;
}