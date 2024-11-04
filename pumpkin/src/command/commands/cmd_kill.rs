use async_trait::async_trait;
use pumpkin_core::text::color::NamedColor;
use pumpkin_core::text::TextComponent;

use crate::command::args::arg_entities::EntitiesArgumentConsumer;
use crate::command::args::{Arg, ConsumedArgs};
use crate::command::tree::CommandTree;
use crate::command::tree_builder::{argument, require};
use crate::command::{CommandExecutor, CommandSender, InvalidTreeError};
use InvalidTreeError::InvalidConsumptionError;

const NAMES: [&str; 1] = ["kill"];
const DESCRIPTION: &str = "Kills all target entities.";

const ARG_TARGET: &str = "target";

struct KillExecutor;

#[async_trait]
impl CommandExecutor for KillExecutor {
    async fn execute<'a>(
        &self,
        sender: &mut CommandSender<'a>,
        _server: &crate::server::Server,
        args: &ConsumedArgs<'a>,
    ) -> Result<(), InvalidTreeError> {
        let Some(Arg::Entities(targets)) = args.get(&ARG_TARGET) else {
            return Err(InvalidConsumptionError(Some(ARG_TARGET.into())));
        };

        let target_count = targets.len();

        for target in targets {
            target.living_entity.kill().await;
        }

        let msg = if target_count == 1 {
            TextComponent::text("Enitity has been killed.")
        } else {
            TextComponent::text_string(format!("{target_count} entities have been killed."))
        };

        sender.send_message(msg.color_named(NamedColor::Blue)).await;

        Ok(())
    }
}

struct KillSelfExecutor;

#[async_trait]
impl CommandExecutor for KillSelfExecutor {
    async fn execute<'a>(
        &self,
        sender: &mut CommandSender<'a>,
        _server: &crate::server::Server,
        _args: &ConsumedArgs<'a>,
    ) -> Result<(), InvalidTreeError> {
        let target = sender
            .as_player()
            .ok_or(InvalidTreeError::InvalidRequirementError)?;

        target.living_entity.kill().await;

        Ok(())
    }
}

#[allow(clippy::redundant_closure_for_method_calls)] // causes lifetime issues
pub fn init_command_tree<'a>() -> CommandTree<'a> {
    CommandTree::new(NAMES, DESCRIPTION)
        .with_child(argument(ARG_TARGET, &EntitiesArgumentConsumer).execute(&KillExecutor))
        .with_child(require(&|sender| sender.is_player()).execute(&KillSelfExecutor))
}
