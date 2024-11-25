use pumpkin::plugin::*;
use pumpkin::plugin_metadata;
use pumpkin_api_macros::{plugin_impl, plugin_method, plugin_event};

plugin_metadata!(
    "Plugin name",
    "plugin-id",
    "1.0.0",
    &["Author Name"],
    "Description"
);

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

#[plugin_event]
fn on_player_join(&mut self, server: &dyn PluginContext, event: &dyn PlayerConnectionEvent) -> Result<(), String> {
    server.get_logger().info(format!("Player {} joined the game", event.get_player().gameprofile.name).as_str());
    Ok(())
}

#[plugin_impl]
pub struct MyPlugin {}
