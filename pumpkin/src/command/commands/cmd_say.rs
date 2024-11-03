use async_trait::async_trait;
use pumpkin_core::text::TextComponent;
use pumpkin_protocol::client::play::CSystemChatMessage;

use crate::command::{
    args::{arg_message::MsgArgConsumer, Arg, ConsumedArgs},
    tree::CommandTree,
    tree_builder::{argument, require},
    CommandExecutor, CommandSender, InvalidTreeError,
};
use InvalidTreeError::InvalidConsumptionError;

const NAMES: [&str; 1] = ["say"];

const DESCRIPTION: &str = "Broadcast a message to all Players.";

const ARG_MESSAGE: &str = "message";

struct SayExecutor;

#[async_trait]
impl CommandExecutor for SayExecutor {
    async fn execute<'a>(
        &self,
        sender: &mut CommandSender<'a>,
        server: &crate::server::Server,
        args: &ConsumedArgs<'a>,
    ) -> Result<(), InvalidTreeError> {
        let sender = match sender {
            CommandSender::Console => "Console",
            CommandSender::Rcon(_) => "Rcon",
            CommandSender::Player(player) => &player.gameprofile.name,
        };

        let Some(Arg::Msg(msg)) = args.get(ARG_MESSAGE) else {
            return Err(InvalidConsumptionError(Some(ARG_MESSAGE.into())));
        };

        server
            .broadcast_packet_all(&CSystemChatMessage::new(
                &TextComponent::text(&format!("[{sender}] {msg}")),
                false,
            ))
            .await;
        Ok(())
    }
}

pub fn init_command_tree<'a>() -> CommandTree<'a> {
    CommandTree::new(NAMES, DESCRIPTION).with_child(
        require(&|sender| sender.permission_lvl() >= 2)
            .with_child(argument(ARG_MESSAGE, &MsgArgConsumer).execute(&SayExecutor)),
    )
}
