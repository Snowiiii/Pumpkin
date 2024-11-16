use std::sync::Arc;

use async_trait::async_trait;
use pumpkin_core::text::color::NamedColor;
use pumpkin_core::text::TextComponent;
use pumpkin_inventory::Container;

use crate::command::args::arg_entities::EntitiesArgumentConsumer;
use crate::command::args::{Arg, ConsumedArgs};
use crate::command::tree::CommandTree;
use crate::command::tree_builder::{argument, require};
use crate::command::{CommandError, CommandExecutor, CommandSender};
use crate::entity::player::Player;
use CommandError::InvalidConsumption;

const NAMES: [&str; 1] = ["clear"];
const DESCRIPTION: &str = "Clear yours or targets inventory.";

const ARG_TARGET: &str = "target";

async fn clear_player(target: &Player) -> usize {
    let mut inventory = target.inventory.lock().await;

    let slots = inventory.all_slots();
    let items_count = slots
        .iter()
        .filter_map(|slot| slot.as_ref().map(|slot| slot.item_count as usize))
        .sum();
    for slot in slots {
        *slot = None;
    }
    drop(inventory);
    target.set_container_content(None).await;
    items_count
}

fn clear_command_text_output(item_count: usize, targets: &[Arc<Player>]) -> TextComponent {
    match targets {
        [target] if item_count == 0 => TextComponent::text_string(format!(
            "No items were found on player {}",
            target.gameprofile.name
        ))
        .color_named(NamedColor::Red),
        [target] => TextComponent::text_string(format!(
            "Removed {} item(s) on player {}",
            item_count, target.gameprofile.name
        )),
        targets if item_count == 0 => {
            TextComponent::text_string(format!("No items were found on {} players", targets.len()))
                .color_named(NamedColor::Red)
        }
        targets => TextComponent::text_string(format!(
            "Removed {item_count} item(s) from {} players",
            targets.len()
        )),
    }
}

struct ClearExecutor;

#[async_trait]
impl CommandExecutor for ClearExecutor {
    async fn execute<'a>(
        &self,
        sender: &mut CommandSender<'a>,
        _server: &crate::server::Server,
        args: &ConsumedArgs<'a>,
    ) -> Result<(), CommandError> {
        let Some(Arg::Entities(targets)) = args.get(&ARG_TARGET) else {
            return Err(InvalidConsumption(Some(ARG_TARGET.into())));
        };

        let mut item_count = 0;
        for target in targets {
            item_count += clear_player(target).await;
        }

        let msg = clear_command_text_output(item_count, targets);

        sender.send_message(msg).await;

        Ok(())
    }
}

struct ClearSelfExecutor;

#[async_trait]
impl CommandExecutor for ClearSelfExecutor {
    async fn execute<'a>(
        &self,
        sender: &mut CommandSender<'a>,
        _server: &crate::server::Server,
        _args: &ConsumedArgs<'a>,
    ) -> Result<(), CommandError> {
        let target = sender.as_player().ok_or(CommandError::InvalidRequirement)?;

        let item_count = clear_player(&target).await;

        let hold_target = [target];
        let msg = clear_command_text_output(item_count, &hold_target);

        sender.send_message(msg).await;

        Ok(())
    }
}

#[allow(clippy::redundant_closure_for_method_calls)] // causes lifetime issues
pub fn init_command_tree<'a>() -> CommandTree<'a> {
    CommandTree::new(NAMES, DESCRIPTION)
        .with_child(argument(ARG_TARGET, &EntitiesArgumentConsumer).execute(&ClearExecutor))
        .with_child(require(&|sender| sender.is_player()).execute(&ClearSelfExecutor))
}
