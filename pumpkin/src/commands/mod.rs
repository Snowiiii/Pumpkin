use pumpkin::PumpkinCommand;
use pumpkin_protocol::text::TextComponent;

use crate::{client::Client, server::Server};

mod pumpkin;

/// I think it would be great to split this up into a seperate crate, But idk how i should do that, Because we have to rely on Client and Server
pub trait Command<'a> {
    // Name of the Plugin, Use lower case
    const NAME: &'a str;
    const DESCRIPTION: &'a str;

    fn on_execute(sender: &mut CommandSender<'a>, command: String, server: &mut Server);
}

pub enum CommandSender<'a> {
    Console,
    Player(&'a mut Client),
}

impl<'a> CommandSender<'a> {
    pub fn send_message(&mut self, text: TextComponent) {
        match self {
            // todo: add color and stuff to console
            CommandSender::Console => log::info!("{}", text.text),
            CommandSender::Player(c) => c.send_message(text),
        }
    }
}
pub fn handle_command(sender: &mut CommandSender, command: String, server: &mut Server) {
    let command = command.to_lowercase();
    dbg!("handling command");
    dbg!(&command);
    if command.starts_with(PumpkinCommand::NAME) {
        PumpkinCommand::on_execute(sender, command, server)
    }
}
