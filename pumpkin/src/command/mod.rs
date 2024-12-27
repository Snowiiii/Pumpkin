use std::fmt;
use std::sync::Arc;

use crate::command::commands::cmd_seed;
use crate::command::commands::{cmd_bossbar, cmd_transfer};
use crate::command::dispatcher::CommandDispatcher;
use crate::entity::player::{PermissionLvl, Player};
use crate::server::Server;
use crate::world::World;
use args::ConsumedArgs;
use async_trait::async_trait;
use commands::{
    cmd_clear, cmd_fill, cmd_gamemode, cmd_give, cmd_help, cmd_kick, cmd_kill, cmd_list,
    cmd_plugin, cmd_plugins, cmd_pumpkin, cmd_say, cmd_setblock, cmd_stop, cmd_teleport, cmd_time,
    cmd_worldborder,
};
use dispatcher::CommandError;
use pumpkin_core::math::vector3::Vector3;
use pumpkin_core::text::TextComponent;

pub mod args;
pub mod client_cmd_suggestions;
mod commands;
pub mod dispatcher;
pub mod tree;
pub mod tree_builder;
mod tree_format;

pub enum CommandSender<'a> {
    Rcon(&'a tokio::sync::Mutex<Vec<String>>),
    Console,
    Player(Arc<Player>),
}

impl fmt::Display for CommandSender<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                CommandSender::Console => "Server",
                CommandSender::Rcon(_) => "Rcon",
                CommandSender::Player(p) => &p.gameprofile.name,
            }
        )
    }
}

impl<'a> CommandSender<'a> {
    pub async fn send_message(&self, text: TextComponent<'a>) {
        match self {
            CommandSender::Console => log::info!("{}", text.to_pretty_console()),
            CommandSender::Player(c) => c.send_system_message(&text).await,
            CommandSender::Rcon(s) => s.lock().await.push(text.to_pretty_console()),
        }
    }

    #[must_use]
    pub const fn is_player(&self) -> bool {
        matches!(self, CommandSender::Player(_))
    }

    #[must_use]
    pub const fn is_console(&self) -> bool {
        matches!(self, CommandSender::Console)
    }
    #[must_use]
    pub fn as_player(&self) -> Option<Arc<Player>> {
        match self {
            CommandSender::Player(player) => Some(player.clone()),
            _ => None,
        }
    }

    /// prefer using `has_permission_lvl(lvl)`
    #[must_use]
    pub fn permission_lvl(&self) -> PermissionLvl {
        match self {
            CommandSender::Console | CommandSender::Rcon(_) => PermissionLvl::Four,
            CommandSender::Player(p) => p.permission_lvl(),
        }
    }

    #[must_use]
    pub fn has_permission_lvl(&self, lvl: PermissionLvl) -> bool {
        match self {
            CommandSender::Console | CommandSender::Rcon(_) => true,
            CommandSender::Player(p) => (p.permission_lvl() as i8) >= (lvl as i8),
        }
    }

    #[must_use]
    pub fn position(&self) -> Option<Vector3<f64>> {
        match self {
            CommandSender::Console | CommandSender::Rcon(..) => None,
            CommandSender::Player(p) => Some(p.living_entity.entity.pos.load()),
        }
    }

    #[must_use]
    pub fn world(&self) -> Option<&World> {
        match self {
            // TODO: maybe return first world when console
            CommandSender::Console | CommandSender::Rcon(..) => None,
            CommandSender::Player(p) => Some(&p.living_entity.entity.world),
        }
    }
}

#[must_use]
pub fn default_dispatcher<'a>() -> CommandDispatcher<'a> {
    let mut dispatcher = CommandDispatcher::default();

    dispatcher.register(cmd_pumpkin::init_command_tree());
    dispatcher.register(cmd_bossbar::init_command_tree());
    dispatcher.register(cmd_say::init_command_tree());
    dispatcher.register(cmd_gamemode::init_command_tree());
    dispatcher.register(cmd_stop::init_command_tree());
    dispatcher.register(cmd_help::init_command_tree());
    dispatcher.register(cmd_kill::init_command_tree());
    dispatcher.register(cmd_kick::init_command_tree());
    dispatcher.register(cmd_plugin::init_command_tree());
    dispatcher.register(cmd_plugins::init_command_tree());
    dispatcher.register(cmd_worldborder::init_command_tree());
    dispatcher.register(cmd_teleport::init_command_tree());
    dispatcher.register(cmd_time::init_command_tree());
    dispatcher.register(cmd_give::init_command_tree());
    dispatcher.register(cmd_list::init_command_tree());
    dispatcher.register(cmd_clear::init_command_tree());
    dispatcher.register(cmd_setblock::init_command_tree());
    dispatcher.register(cmd_seed::init_command_tree());
    dispatcher.register(cmd_transfer::init_command_tree());
    dispatcher.register(cmd_fill::init_command_tree());

    dispatcher
}

#[async_trait]
pub trait CommandExecutor: Sync {
    async fn execute<'a>(
        &self,
        sender: &mut CommandSender<'a>,
        server: &Server,
        args: &ConsumedArgs<'a>,
    ) -> Result<(), CommandError>;
}
