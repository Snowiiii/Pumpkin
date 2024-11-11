use async_trait::async_trait;
use pumpkin_core::text::color::NamedColor;
use pumpkin_core::text::TextComponent;

use crate::command::args::arg_players::PlayersArgumentConsumer;
use crate::command::args::{Arg, ConsumedArgs};
use crate::command::tree::CommandTree;
use crate::command::tree_builder::argument;
use crate::command::InvalidTreeError;
use crate::command::{CommandExecutor, CommandSender};
use InvalidTreeError::InvalidConsumptionError;

const NAMES: [&str; 1] = ["kick"];
const DESCRIPTION: &str = "Kicks the target player from the server.";

const ARG_TARGET: &str = "target";

struct KickExecutor;

#[async_trait]
impl CommandExecutor for KickExecutor {
    async fn execute<'a>(
        &self,
        sender: &mut CommandSender<'a>,
        _server: &crate::server::Server,
        args: &ConsumedArgs<'a>,
    ) -> Result<(), InvalidTreeError> {
        let Some(Arg::Players(targets)) = args.get(&ARG_TARGET) else {
            return Err(InvalidConsumptionError(Some(ARG_TARGET.into())));
        };

        let target_count = targets.len();

        for target in targets {
            target
                .kick(TextComponent::text("Kicked by an operator"))
                .await;
        }

        let msg = if target_count == 1 {
            TextComponent::text("Player has been kicked.")
        } else {
            TextComponent::text_string(format!("{target_count} players have been kicked."))
        };

        sender.send_message(msg.color_named(NamedColor::Blue)).await;

        Ok(())
    }
}

pub fn init_command_tree<'a>() -> CommandTree<'a> {
    CommandTree::new(NAMES, DESCRIPTION)
        .with_child(argument(ARG_TARGET, &PlayersArgumentConsumer).execute(&KickExecutor))
}
