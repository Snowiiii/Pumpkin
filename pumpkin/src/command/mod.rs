use std::sync::Arc;

use args::ConsumedArgs;
use async_trait::async_trait;
use commands::{
    cmd_clear, cmd_echest, cmd_gamemode, cmd_give, cmd_help, cmd_kick, cmd_kill, cmd_list,
    cmd_pumpkin, cmd_say, cmd_stop, cmd_teleport, cmd_worldborder,
};
use dispatcher::InvalidTreeError;
use pumpkin_core::math::vector3::Vector3;
use pumpkin_core::text::TextComponent;

use crate::command::dispatcher::CommandDispatcher;
use crate::entity::player::Player;
use crate::server::Server;

pub mod args;
mod commands;
pub mod dispatcher;
mod tree;
mod tree_builder;
mod tree_format;

pub enum CommandSender<'a> {
    Rcon(&'a tokio::sync::Mutex<Vec<String>>),
    Console,
    Player(Arc<Player>),
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

    /// todo: implement
    #[must_use]
    pub const fn permission_lvl(&self) -> i32 {
        4
    }

    #[must_use]
    pub fn position(&self) -> Option<Vector3<f64>> {
        match self {
            CommandSender::Console | CommandSender::Rcon(..) => None,
            CommandSender::Player(p) => Some(p.living_entity.entity.pos.load()),
        }
    }
}

#[must_use]
pub fn default_dispatcher<'a>() -> Arc<CommandDispatcher<'a>> {
    let mut dispatcher = CommandDispatcher::default();

    dispatcher.register(cmd_pumpkin::init_command_tree());
    dispatcher.register(cmd_say::init_command_tree());
    dispatcher.register(cmd_gamemode::init_command_tree());
    dispatcher.register(cmd_stop::init_command_tree());
    dispatcher.register(cmd_help::init_command_tree());
    dispatcher.register(cmd_echest::init_command_tree());
    dispatcher.register(cmd_kill::init_command_tree());
    dispatcher.register(cmd_kick::init_command_tree());
    dispatcher.register(cmd_worldborder::init_command_tree());
    dispatcher.register(cmd_teleport::init_command_tree());
    dispatcher.register(cmd_give::init_command_tree());
    dispatcher.register(cmd_list::init_command_tree());
    dispatcher.register(cmd_clear::init_command_tree());

    Arc::new(dispatcher)
}

#[async_trait]
pub(crate) trait CommandExecutor: Sync {
    async fn execute<'a>(
        &self,
        sender: &mut CommandSender<'a>,
        server: &Server,
        args: &ConsumedArgs<'a>,
    ) -> Result<(), InvalidTreeError>;
}
