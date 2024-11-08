use async_trait::async_trait;
use pumpkin_core::text::color::{Color, NamedColor};
use pumpkin_core::text::TextComponent;
use pumpkin_world::item::item_registry;

use crate::command::args::arg_bounded_num::BoundedNumArgumentConsumer;
use crate::command::args::arg_item::ItemArgumentConsumer;
use crate::command::args::arg_players::PlayersArgumentConsumer;
use crate::command::args::{ConsumedArgs, FindArg, FindArgDefaultName};
use crate::command::tree::CommandTree;
use crate::command::tree_builder::{argument, argument_default_name, require};
use crate::command::{CommandExecutor, CommandSender, InvalidTreeError};

const NAMES: [&str; 1] = ["give"];

const DESCRIPTION: &str = "Give items to player(s).";

const ARG_ITEM: &str = "item";

static ITEM_COUNT_CONSUMER: BoundedNumArgumentConsumer<i32> =
    BoundedNumArgumentConsumer::new().name("count").max(6400);

struct GiveExecutor;

#[async_trait]
impl CommandExecutor for GiveExecutor {
    async fn execute<'a>(
        &self,
        sender: &mut CommandSender<'a>,
        _server: &crate::server::Server,
        args: &ConsumedArgs<'a>,
    ) -> Result<(), InvalidTreeError> {
        let targets = PlayersArgumentConsumer.find_arg_default_name(args)?;

        let item_name = ItemArgumentConsumer::find_arg(args, ARG_ITEM)?;

        let Some(item) = item_registry::get_item(item_name) else {
            sender
                .send_message(
                    TextComponent::text_string(format!("Item {item_name} does not exist."))
                        .color(Color::Named(NamedColor::Red)),
                )
                .await;
            return Ok(());
        };

        let item_count = match ITEM_COUNT_CONSUMER.find_arg_default_name(args) {
            Err(_) => 1,
            Ok(Ok(count)) => count,
            Ok(Err(())) => {
                sender
                    .send_message(
                        TextComponent::text("Item count is too large or too small.")
                            .color(Color::Named(NamedColor::Red)),
                    )
                    .await;
                return Ok(());
            }
        };

        for target in targets {
            target.give_items(item, item_count).await;
        }

        sender
            .send_message(TextComponent::text_string(match targets {
                [target] => format!(
                    "Gave {item_count} {item_name} to {}",
                    target.gameprofile.name
                ),
                _ => format!("Gave {item_count} {item_name} to {} players", targets.len()),
            }))
            .await;

        Ok(())
    }
}

pub fn init_command_tree<'a>() -> CommandTree<'a> {
    CommandTree::new(NAMES, DESCRIPTION).with_child(
        require(&|sender| sender.permission_lvl() >= 2).with_child(
            argument_default_name(&PlayersArgumentConsumer).with_child(
                argument(ARG_ITEM, &ItemArgumentConsumer)
                    .execute(&GiveExecutor)
                    .with_child(argument_default_name(&ITEM_COUNT_CONSUMER).execute(&GiveExecutor)),
            ),
        ),
    )
}
