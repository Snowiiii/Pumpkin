use async_trait::async_trait;
use pumpkin_core::text::{color::NamedColor, TextComponent};

use crate::command::{
    args::{
        arg_message::MsgArgConsumer, arg_players::PlayersArgumentConsumer, ConsumedArgs, FindArg,
    },
    tree::CommandTree,
    tree_builder::argument,
    CommandExecutor, CommandSender, InvalidTreeError,
};

const NAMES: [&str; 3] = ["msg", "tell", "w"];

const DESCRIPTION: &str = "Sends a private message to one or more players.";

const ARG_TARGET: &str = "target";
const ARG_MESSAGE: &str = "message";

struct MsgExecutor;

#[async_trait]
impl CommandExecutor for MsgExecutor {
    async fn execute<'a>(
        &self,
        sender: &mut CommandSender<'a>,
        _server: &crate::server::Server,
        args: &ConsumedArgs<'a>,
    ) -> Result<(), InvalidTreeError> {
        let sender_name = match sender {
            CommandSender::Console => "Console",
            CommandSender::Rcon(_) => "Rcon",
            CommandSender::Player(player) => &player.gameprofile.name.clone(),
        };

        let targets = PlayersArgumentConsumer::find_arg(args, ARG_TARGET)?;
        let msg = MsgArgConsumer::find_arg(args, ARG_MESSAGE)?;

        let sender_message = targets
            .iter()
            .map(|target| format!("You whisper to {}: {}", target.gameprofile.name, msg))
            .collect::<Vec<String>>()
            .join("\n");
        let sender_text = TextComponent::text(&sender_message)
            .color_named(NamedColor::Gray)
            .italic();

        let recipient_message = format!("{sender_name} whispers to you: {msg}");
        let recipient_text = TextComponent::text(&recipient_message)
            .color_named(NamedColor::Gray)
            .italic();

        sender.send_message(sender_text).await;

        for target_player in targets {
            target_player.send_system_message(&recipient_text).await;
        }

        Ok(())
    }
}

pub fn init_command_tree<'a>() -> CommandTree<'a> {
    CommandTree::new(NAMES, DESCRIPTION).with_child(
        argument(ARG_TARGET, &PlayersArgumentConsumer)
            .with_child(argument(ARG_MESSAGE, &MsgArgConsumer).execute(&MsgExecutor)),
    )
}
