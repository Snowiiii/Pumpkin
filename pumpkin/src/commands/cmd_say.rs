use async_trait::async_trait;
use pumpkin_core::text::TextComponent;
use pumpkin_protocol::client::play::CSystemChatMessage;
use super::{arg_simple::SimpleArgConsumer, tree::CommandTree, tree_builder::{argument, require}, CommandExecutor, CommandSender};

const NAMES: [&str; 1] = ["say"];

const DESCRIPTION: &str = "Broadcast a message to all Players.";

const ARG_MESSAGE: &str = "message";

struct SayExecutor {}

#[async_trait]
impl CommandExecutor for SayExecutor {
    async fn execute<'a>(
        &self,
        sender: &mut super::CommandSender<'a>,
        server: &crate::server::Server,
        args: &super::tree::ConsumedArgs<'a>,
    ) -> Result<(), super::dispatcher::InvalidTreeError> {
        let sender = match sender {
            CommandSender::Console => "Console",
            CommandSender::Rcon(_) => "Rcon",
            CommandSender::Player(player) => &player.gameprofile.name
        };

        server.broadcast_packet_all(&CSystemChatMessage::new(&TextComponent::text(&("[".to_string() + sender + "] " + args.get("message").unwrap())), false)).await;
        Ok(())
    }
}

pub fn init_command_tree<'a>() -> CommandTree<'a> {
    CommandTree::new(NAMES, DESCRIPTION)
        .with_child(
            require(&|sender| sender.permission_lvl() >= 2).with_child(
                argument(ARG_MESSAGE, &SimpleArgConsumer {})
                .execute(&SayExecutor {})
            )
    )
}