use pumpkin::entity::player::Player;
use pumpkin::plugin::*;
use pumpkin_api_macros::{plugin_event, plugin_impl, plugin_method};
use pumpkin_core::text::TextComponent;
use pumpkin_core::GameMode;

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
async fn on_player_join(
    &mut self,
    server: &dyn PluginContext,
    player: &Player,
) -> Result<bool, String> {
    server
        .get_logger()
        .info(format!("Player {} joined the game", player.gameprofile.name).as_str());
    /// TODO: Calling any method that involves sending packets to the player will cause the client to crash
    //let _ = player.send_system_message(&TextComponent::text("Hello, world!")).await;
    //player.set_gamemode(GameMode::Creative).await;
    //player.kill().await;
    // Returning true will block any other plugins from receiving this event
    Ok(true)
}

#[plugin_event]
async fn on_player_leave(
    &mut self,
    server: &dyn PluginContext,
    player: &Player,
) -> Result<bool, String> {
    server
        .get_logger()
        .info(format!("Player {} left the game", player.gameprofile.name).as_str());
    Ok(false)
}

#[plugin_impl]
pub struct MyPlugin {}
