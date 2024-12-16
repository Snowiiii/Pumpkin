use pumpkin::plugin::api::types::player::PlayerEvent;
use pumpkin::plugin::*;
use pumpkin_api_macros::{plugin_event, plugin_impl, plugin_method};
use pumpkin_core::text::color::NamedColor;
use pumpkin_core::text::TextComponent;
use serde::{Deserialize, Serialize};
use std::fs;

#[derive(Serialize, Deserialize, Debug)]
struct Config {
    bans: Bans,
}

#[derive(Serialize, Deserialize, Debug)]
struct Bans {
    players: Vec<String>,
}

#[plugin_method]
fn on_load(&mut self, server: &Context) -> Result<(), String> {
    let data_folder = server.get_data_folder();
    if !fs::exists(format!("{}/data.toml", data_folder)).unwrap() {
        let cfg = toml::to_string(&self.config).unwrap();
        fs::write(format!("{}/data.toml", data_folder), cfg).unwrap();
        server
            .get_logger()
            .info(format!("Created config in {} with {:#?}", data_folder, self.config).as_str());
    } else {
        let data = fs::read_to_string(format!("{}/data.toml", data_folder)).unwrap();
        self.config = toml::from_str(data.as_str()).unwrap();
        server
            .get_logger()
            .info(format!("Loaded config from {} with {:#?}", data_folder, self.config).as_str());
    }

    server.get_logger().info("Plugin loaded!");
    Ok(())
}

#[plugin_method]
fn on_unload(&mut self, server: &Context) -> Result<(), String> {
    let data_folder = server.get_data_folder();
    let cfg = toml::to_string(&self.config).unwrap();
    fs::write(format!("{}/data.toml", data_folder), cfg).unwrap();

    server.get_logger().info("Plugin unloaded!");
    Ok(())
}

#[plugin_event(blocking = true, priority = Highest)]
async fn on_player_join(
    &mut self,
    server: &Context,
    player: &PlayerEvent,
) -> Result<bool, String> {
    server.get_logger().info(
        format!(
            "Player {} joined the game. Config is {:#?}",
            player.gameprofile.name, self.config
        )
        .as_str(),
    );

    if self.config.bans.players.contains(&player.gameprofile.name) {
        let _ = player
            .kick(TextComponent::text("You are banned from the server"))
            .await;
        return Ok(true);
    }

    let _ = player
        .send_message(
            TextComponent::text_string(format!(
                "Hello {}, welocme to the server",
                player.gameprofile.name
            ))
            .color_named(NamedColor::Green),
        )
        .await;
    Ok(true)
}

#[plugin_event]
async fn on_player_leave(
    &mut self,
    server: &Context,
    player: &PlayerEvent,
) -> Result<bool, String> {
    server
        .get_logger()
        .info(format!("Player {} left the game", player.gameprofile.name).as_str());
    Ok(false)
}

#[plugin_impl]
pub struct MyPlugin {
    config: Config,
}

impl MyPlugin {
    pub fn new() -> Self {
        MyPlugin {
            config: Config {
                bans: Bans { players: vec![] },
            },
        }
    }
}
