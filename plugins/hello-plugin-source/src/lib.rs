use async_trait::async_trait;
use pumpkin::command::args::arg_block::BlockArgumentConsumer;
use pumpkin::command::args::arg_position_block::BlockPosArgumentConsumer;
use pumpkin::command::args::ConsumedArgs;
use pumpkin::command::args::FindArg;
use pumpkin::command::dispatcher::CommandError;
use pumpkin::command::tree::CommandTree;
use pumpkin::command::tree_builder::argument;
use pumpkin::command::tree_builder::literal;
use pumpkin::command::tree_builder::require;
use pumpkin::command::CommandExecutor;
use pumpkin::command::CommandSender;
use pumpkin::plugin::api::types::player::PlayerEvent;
use pumpkin::plugin::*;
use pumpkin::server::Server;
use pumpkin_api_macros::{plugin_event, plugin_impl, plugin_method, with_runtime};
use pumpkin_core::text::color::NamedColor;
use pumpkin_core::text::TextComponent;
use pumpkin_core::PermissionLvl;
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

const NAMES: [&str; 1] = ["setblock2"];

const DESCRIPTION: &str = "Place a block.";

const ARG_BLOCK: &str = "block";
const ARG_BLOCK_POS: &str = "position";

#[derive(Clone, Copy)]
enum Mode {
    /// with particles + item drops
    Destroy,

    /// only replaces air
    Keep,

    /// default; without particles
    Replace,
}

struct SetblockExecutor(Mode);

// IMPORTANT: If using something that requires a tokio runtime, the #[with_runtime] attribute must be used.
// EVEN MORE IMPORTANT: The #[with_runtime] attribute must be used **BRFORE** the #[async_trait] attribute.
#[with_runtime(global)]
#[async_trait]
impl CommandExecutor for SetblockExecutor {
    async fn execute<'a>(
        &self,
        sender: &mut CommandSender<'a>,
        _server: &Server,
        args: &ConsumedArgs<'a>,
    ) -> Result<(), CommandError> {
        let block = BlockArgumentConsumer::find_arg(args, ARG_BLOCK)?;
        let block_state_id = block.default_state_id;
        let pos = BlockPosArgumentConsumer::find_arg(args, ARG_BLOCK_POS)?;
        let mode = self.0;
        // TODO: allow console to use the command (seed sender.world)
        let world = sender.world().ok_or(CommandError::InvalidRequirement)?;

        let success = match mode {
            Mode::Destroy => {
                world.break_block(pos, None).await;
                world.set_block_state(pos, block_state_id).await;
                true
            }
            Mode::Replace => {
                world.set_block_state(pos, block_state_id).await;
                true
            }
            Mode::Keep => match world.get_block_state(pos).await {
                Ok(old_state) if old_state.air => {
                    world.set_block_state(pos, block_state_id).await;
                    true
                }
                Ok(_) => false,
                Err(e) => return Err(CommandError::OtherPumpkin(e.into())),
            },
        };

        sender
            .send_message(if success {
                TextComponent::text(format!("Placed block {} at {pos}", block.name,))
            } else {
                TextComponent::text(format!("Kept block at {pos}")).color_named(NamedColor::Red)
            })
            .await;

        Ok(())
    }
}

pub fn init_command_tree() -> CommandTree {
    CommandTree::new(NAMES, DESCRIPTION).with_child(
        require(|sender| sender.has_permission_lvl(PermissionLvl::Two) && sender.world().is_some())
            .with_child(
                argument(ARG_BLOCK_POS, BlockPosArgumentConsumer).with_child(
                    argument(ARG_BLOCK, BlockArgumentConsumer)
                        .with_child(literal("replace").execute(SetblockExecutor(Mode::Replace)))
                        .with_child(literal("destroy").execute(SetblockExecutor(Mode::Destroy)))
                        .with_child(literal("keep").execute(SetblockExecutor(Mode::Keep)))
                        .execute(SetblockExecutor(Mode::Replace)),
                ),
            ),
    )
}

#[plugin_method]
async fn on_load(&mut self, server: &Context) -> Result<(), String> {
    env_logger::init();
    server
        .register_command(init_command_tree(), PermissionLvl::Two)
        .await;
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
async fn on_unload(&mut self, server: &Context) -> Result<(), String> {
    let data_folder = server.get_data_folder();
    let cfg = toml::to_string(&self.config).unwrap();
    fs::write(format!("{}/data.toml", data_folder), cfg).unwrap();

    server.get_logger().info("Plugin unloaded!");
    Ok(())
}

#[plugin_event(blocking = true, priority = Highest)]
async fn on_player_join(&mut self, server: &Context, player: &PlayerEvent) -> Result<bool, String> {
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
            TextComponent::text(format!(
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

impl Default for MyPlugin {
    fn default() -> Self {
        Self::new()
    }
}
