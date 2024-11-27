use pumpkin::plugin::*;
use pumpkin_api_macros::{plugin_impl, plugin_method, plugin_event};
use pumpkin::entity::player::Player;

#[plugin_method]
fn on_load(&mut self, server: &dyn PluginContext) -> Result<(), String> {
    server.get_logger().info("Plugin loaded!");
    Ok(())
}

#[plugin_method]
fn on_unload(&mut self, server: &dyn PluginContext) -> Result<(), String> {
    server.get_logger().info("Plugin unloaded!");
    Ok(())
}

#[plugin_event(blocking = true, priority = Highest)]
fn on_player_join(&mut self, server: &dyn PluginContext, player: &Player) -> Result<bool, String> {
    server.get_logger().info(format!("Player {} joined the game", player.gameprofile.name).as_str());
    // Returning true will block any other plugins from receiving this event
    Ok(true)
}

#[plugin_event]
fn on_player_leave(&mut self, server: &dyn PluginContext, player: &Player) -> Result<bool, String> {
    server.get_logger().info(format!("Player {} left the game", player.gameprofile.name).as_str());
    Ok(false)
}

#[plugin_impl]
pub struct MyPlugin {}
