use async_trait::async_trait;
use pumpkin_core::text::color::NamedColor;
use pumpkin_core::text::TextComponent;

use crate::command::arg_player::{parse_arg_player, PlayerArgumentConsumer};
use crate::command::tree::CommandTree;
use crate::command::tree_builder::argument;
use crate::command::{tree::ConsumedArgs, CommandExecutor, CommandSender, InvalidTreeError};

const NAMES: [&str; 1] = ["kill"];
const DESCRIPTION: &str = "Kills a target player.";

const ARG_TARGET: &str = "target";

struct KillExecutor {}

#[async_trait]
impl CommandExecutor for KillExecutor {
    async fn execute<'a>(
        &self,
        sender: &mut CommandSender<'a>,
        server: &crate::server::Server,
        args: &ConsumedArgs<'a>,
    ) -> Result<(), InvalidTreeError> {
        // TODO parse entities not only players
        let target = parse_arg_player(sender, server, ARG_TARGET, args).await?;
        target.living_entity.kill().await;

        sender
            .send_message(
                TextComponent::text("Player has been killed.").color_named(NamedColor::Blue),
            )
            .await;

        Ok(())
    }
}

pub fn init_command_tree<'a>() -> CommandTree<'a> {
    CommandTree::new(NAMES, DESCRIPTION)
        .with_child(argument(ARG_TARGET, &PlayerArgumentConsumer {}).execute(&KillExecutor {}))
}
