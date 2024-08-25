use pumpkin_core::text::TextComponent;
use std::collections::HashMap;
use std::sync::OnceLock;

use crate::commands::dispatcher::CommandDispatcher;
use crate::entity::player::Player;
use crate::server::Server;
mod arg_player;
mod cmd_gamemode;
mod cmd_help;
mod cmd_pumpkin;
mod cmd_stop;
mod dispatcher;
mod tree;
mod tree_builder;
mod tree_format;

pub enum CommandSender<'a> {
    Rcon(&'a mut Vec<String>),
    Console,
    Player(&'a mut Player),
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

    pub fn is_player(&self) -> bool {
        match self {
            CommandSender::Console => false,
            CommandSender::Player(_) => true,
            CommandSender::Rcon(_) => false,
        }
    }

    pub fn is_console(&self) -> bool {
        match self {
            CommandSender::Console => true,
            CommandSender::Player(_) => false,
            CommandSender::Rcon(_) => true,
        }
    }
    pub fn as_mut_player(&mut self) -> Option<&mut Player> {
        match self {
            CommandSender::Player(player) => Some(player),
            CommandSender::Console => None,
            CommandSender::Rcon(_) => None,
        }
    }

    /// todo: implement
    pub fn permission_lvl(&self) -> i32 {
        match self {
            CommandSender::Rcon(_) => 4,
            CommandSender::Console => 4,
            CommandSender::Player(_) => 4,
        }
    }
}

/// todo: reconsider using constant
///
/// Central point from which commands are dispatched. Should always be initialized using
/// [dispatcher_init].
static DISPATCHER: OnceLock<CommandDispatcher> = OnceLock::new();

/// create [CommandDispatcher] instance for [DISPATCHER]
fn dispatcher_init<'a>() -> CommandDispatcher<'a> {
    let mut dispatcher = CommandDispatcher {
        commands: HashMap::new(),
    };

    dispatcher.register(cmd_pumpkin::init_command_tree());
    dispatcher.register(cmd_gamemode::init_command_tree());
    dispatcher.register(cmd_stop::init_command_tree());
    dispatcher.register(cmd_help::init_command_tree());

    dispatcher
}

pub fn handle_command(sender: &mut CommandSender, server: &mut Server, cmd: &str) {
    let dispatcher = DISPATCHER.get_or_init(dispatcher_init);

    if let Err(err) = dispatcher.dispatch(sender, server, cmd) {
        sender.send_message(
            TextComponent::text(&err).color_named(pumpkin_core::text::color::NamedColor::Red),
        )
    }
}
