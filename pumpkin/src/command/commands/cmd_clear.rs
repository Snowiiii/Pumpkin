use std::sync::Arc;

use async_trait::async_trait;
use pumpkin_core::text::color::NamedColor;
use pumpkin_core::text::TextComponent;
use pumpkin_inventory::Container;
use pumpkin_world::item::ItemStack;

use crate::command::args::arg_entities::EntitiesArgumentConsumer;
use crate::command::args::{Arg, ConsumedArgs};
use crate::command::tree::CommandTree;
use crate::command::tree_builder::{argument, require};
use crate::command::{CommandExecutor, CommandSender, InvalidTreeError};
use crate::entity::player::Player;
use InvalidTreeError::InvalidConsumptionError;

const NAMES: [&str; 1] = ["clear"];
const DESCRIPTION: &str = "Clear yours or targets inventory.";

const ARG_TARGET: &str = "target";

async fn clear_player(target: &Arc<Player>) -> usize {
    let mut inventory = target.inventory.lock().await;

    let mut items_count: usize = 0;
    let mut slots = vec![];
    for (slot, item) in inventory.all_slots().iter_mut().enumerate() {
        if let Some(is) = item {
            items_count += is.item_count as usize;
            **item = Option::<ItemStack>::None;
            slots.push(slot);
        }
    }
    // TODO Update whole inventory at once
    for slot in slots {
        target
            .send_inventory_slot_update(&mut inventory, slot)
            .await
            .unwrap();
    }
    items_count
}

struct ClearExecutor;

#[async_trait]
impl CommandExecutor for ClearExecutor {
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

        let mut items_count = 0;
        for target in targets {
            let item_count = clear_player(target).await;
            items_count += item_count;
        }

        let msg = if target_count == 1 {
            let target = targets.first().unwrap();
            if items_count == 0 {
                TextComponent::text_string(format!(
                    "No items were found on player {}",
                    target.gameprofile.name
                ))
                .color_named(NamedColor::Red)
            } else {
                TextComponent::text_string(format!(
                    "Removed {} item(s) on player {}",
                    items_count, target.gameprofile.name
                ))
            }
        } else if items_count == 0 {
            TextComponent::text_string(format!("No items were found on {target_count} players"))
                .color_named(NamedColor::Red)
        } else {
            TextComponent::text_string(format!(
                "Removed {items_count} item(s) from {target_count} players"
            ))
        };

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
    ) -> Result<(), InvalidTreeError> {
        let target = sender
            .as_player()
            .ok_or(InvalidTreeError::InvalidRequirementError)?;

        let items_count = clear_player(&target).await;

        let msg = if items_count == 0 {
            TextComponent::text_string(format!(
                "No items were found on player {}",
                target.gameprofile.name
            ))
            .color_named(NamedColor::Red)
        } else {
            TextComponent::text_string(format!(
                "Removed {} item(s) on player {}",
                items_count, target.gameprofile.name
            ))
        };

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
