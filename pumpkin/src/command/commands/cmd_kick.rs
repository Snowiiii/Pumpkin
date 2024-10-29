use async_trait::async_trait;
use pumpkin_core::text::color::NamedColor;
use pumpkin_core::text::TextComponent;

use crate::command::arg_player::{parse_arg_player, PlayerArgumentConsumer};
use crate::command::tree::CommandTree;
use crate::command::tree_builder::argument;
use crate::command::InvalidTreeError;
use crate::command::{tree::ConsumedArgs, CommandExecutor, CommandSender};

const NAMES: [&str; 1] = ["kick"];
const DESCRIPTION: &str = "Kicks the target player from the server.";

const ARG_TARGET: &str = "target";

struct KickExecutor {}

#[async_trait]
impl CommandExecutor for KickExecutor {
    async fn execute<'a>(
        &self,
        sender: &mut CommandSender<'a>,
        server: &crate::server::Server,
        args: &ConsumedArgs<'a>,
    ) -> Result<(), InvalidTreeError> {
        let target = parse_arg_player(sender, server, ARG_TARGET, args).await?;
        target
            .kick(TextComponent::text("Kicked by an operator"))
            .await;

        sender
            .send_message(
                TextComponent::text("Player has been kicked.").color_named(NamedColor::Blue),
            )
            .await;

        Ok(())
    }
}

pub fn init_command_tree<'a>() -> CommandTree<'a> {
    CommandTree::new(NAMES, DESCRIPTION)
        .with_child(argument(ARG_TARGET, &PlayerArgumentConsumer {}).execute(&KickExecutor {}))
}
