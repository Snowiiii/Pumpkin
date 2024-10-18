use dispatcher::InvalidTreeError;
use pumpkin_core::text::TextComponent;
use tree::ConsumedArgs;

use crate::commands::dispatcher::CommandDispatcher;
use crate::entity::player::Player;
use crate::server::Server;
mod arg_player;
mod cmd_echest;
mod cmd_gamemode;
mod cmd_help;
mod cmd_kill;
mod cmd_kick;
mod cmd_pumpkin;
mod cmd_stop;
pub mod dispatcher;
mod tree;
mod tree_builder;
mod tree_format;

pub enum CommandSender<'a> {
    Rcon(&'a mut Vec<String>),
    Console,
    Player(&'a Player),
}

impl<'a> CommandSender<'a> {
    pub fn send_message(&mut self, text: TextComponent) {
        match self {
            // TODO: add color and stuff to console
            CommandSender::Console => log::info!("{}", text.to_pretty_console()),
            CommandSender::Player(c) => c.send_system_message(text),
            CommandSender::Rcon(s) => s.push(text.to_pretty_console()),
        }
    }

    pub const fn is_player(&self) -> bool {
        match self {
            CommandSender::Console => false,
            CommandSender::Player(_) => true,
            CommandSender::Rcon(_) => false,
        }
    }

    pub const fn is_console(&self) -> bool {
        match self {
            CommandSender::Console => true,
            CommandSender::Player(_) => false,
            CommandSender::Rcon(_) => true,
        }
    }
    pub fn as_mut_player(&mut self) -> Option<&Player> {
        match self {
            CommandSender::Player(player) => Some(player),
            CommandSender::Console => None,
            CommandSender::Rcon(_) => None,
        }
    }

    /// todo: implement
    pub const fn permission_lvl(&self) -> i32 {
        match self {
            CommandSender::Rcon(_) => 4,
            CommandSender::Console => 4,
            CommandSender::Player(_) => 4,
        }
    }
}

pub fn default_dispatcher<'a>() -> CommandDispatcher<'a> {
    let mut dispatcher = CommandDispatcher::default();

    dispatcher.register(cmd_pumpkin::init_command_tree());
    dispatcher.register(cmd_gamemode::init_command_tree());
    dispatcher.register(cmd_stop::init_command_tree());
    dispatcher.register(cmd_help::init_command_tree());
    dispatcher.register(cmd_echest::init_command_tree());
    dispatcher.register(cmd_kill::init_command_tree());
    dispatcher.register(cmd_kick::init_command_tree());

    dispatcher
}

type RunFunctionType =
    (dyn Fn(&mut CommandSender, &Server, &ConsumedArgs) -> Result<(), InvalidTreeError> + Sync);
