use pumpkin::plugin::*;
use pumpkin_api_macros::{plugin_impl, plugin_method, plugin_event};

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
fn on_player_join(&mut self, server: &dyn PluginContext, event: &mut PlayerConnection) -> Result<(), String> {
    server.get_logger().info(format!("Player {} joined the game", event.get_player().gameprofile.name).as_str());
    Ok(())
}

#[plugin_event]
fn on_player_leave(&mut self, server: &dyn PluginContext, event: &mut PlayerConnection) -> Result<(), String> {
    server.get_logger().info(format!("Player {} left the game", event.get_player().gameprofile.name).as_str());
    Ok(())
}

#[plugin_impl]
pub struct MyPlugin {}
