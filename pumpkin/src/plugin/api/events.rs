use crate::{entity::player::Player, world::World};

use super::PluginContext;

pub trait Hooks: Send + Sync + 'static {
    /// Returns an array of events that the 
    fn registered_events(&self) -> Result<&'static[&'static str], String>;

    /// Called when the plugin is loaded.
    fn on_player_join(&mut self, server: &dyn PluginContext, event: &dyn PlayerConnectionEvent) -> Result<(), String>;

    /// Called when the plugin is unloaded.
    fn on_player_leave(&mut self, server: &dyn PluginContext, event: &dyn PlayerConnectionEvent) -> Result<(), String>;
}

pub trait PlayerConnectionEvent {
    fn get_player(&self) -> &Player;
    fn get_world(&self) -> &World;
}