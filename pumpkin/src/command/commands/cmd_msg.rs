use async_trait::async_trait;
use pumpkin_core::text::{color::NamedColor, TextComponent};

use crate::command::{
    args::{arg_message::MsgArgConsumer, arg_players::PlayersArgumentConsumer, Arg, ConsumedArgs},
    tree::CommandTree,
    tree_builder::{argument, require},
    CommandExecutor, CommandSender, InvalidTreeError,
};
use InvalidTreeError::InvalidConsumptionError;

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

        let Some(Arg::Players(targets)) = args.get(&ARG_TARGET) else {
            return Err(InvalidConsumptionError(Some(ARG_TARGET.into())));
        };

        let Some(Arg::Msg(msg)) = args.get(ARG_MESSAGE) else {
            return Err(InvalidConsumptionError(Some(ARG_MESSAGE.into())));
        };

        for target_player in targets {
            let recipient_message = format!("{sender_name} whispers to you: {msg}");
            let sender_message =
                format!("you whisper to {}: {msg}", target_player.gameprofile.name);

            let recipient_text = TextComponent::text(&recipient_message)
                .color_named(NamedColor::Gray)
                .italic();

            let sender_text = TextComponent::text(&sender_message)
                .color_named(NamedColor::Gray)
                .italic();

            target_player.send_system_message(&recipient_text).await;
            sender.send_message(sender_text).await;
        }

        Ok(())
    }
}

pub fn init_command_tree<'a>() -> CommandTree<'a> {
    CommandTree::new(NAMES, DESCRIPTION).with_child(
        require(&|sender| sender.permission_lvl() >= 2).with_child(
            argument(ARG_TARGET, &PlayersArgumentConsumer)
                .with_child(argument(ARG_MESSAGE, &MsgArgConsumer).execute(&MsgExecutor)),
        ),
    )
}
