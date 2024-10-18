use crate::commands::arg_player::{consume_arg_player, parse_arg_player};
use crate::commands::tree::CommandTree;
use crate::commands::tree_builder::argument;
use pumpkin_core::text::{color::NamedColor, TextComponent};

const NAMES: [&str; 1] = ["kill"];
const DESCRIPTION: &str = "Kills a target player.";

const ARG_TARGET: &str = "target";

pub fn init_command_tree<'a>() -> CommandTree<'a> {
    CommandTree::new(NAMES, DESCRIPTION).with_child(
        argument(ARG_TARGET, consume_arg_player).execute(&|sender, server, args| {
            // TODO parse entities not only players
            let target = parse_arg_player(sender, server, ARG_TARGET, args)?;
            target.living_entity.kill();

            sender.send_message(
                TextComponent::text("Player has been killed.").color_named(NamedColor::Blue),
            );

            Ok(())
        }),
    )
}
