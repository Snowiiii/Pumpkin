use crate::commands::arg_player::parse_arg_player;
use crate::commands::tree::CommandTree;
use crate::commands::tree_builder::argument;
use pumpkin_core::text::{color::NamedColor, TextComponent};

use super::arg_player::consume_arg_player;

const NAMES: [&str; 1] = ["kick"];
const DESCRIPTION: &str = "Kicks the target player from the server.";

const ARG_TARGET: &str = "target";

pub fn init_command_tree<'a>() -> CommandTree<'a> {
    CommandTree::new(NAMES, DESCRIPTION).with_child(
        argument(ARG_TARGET, consume_arg_player).execute(&|sender, server, args| {
            dbg!("aa");
            let target = parse_arg_player(sender, server, ARG_TARGET, args)?;
            target.kick(TextComponent::text("Kicked by an operator"));

            sender.send_message(
                TextComponent::text("Player has been kicked.").color_named(NamedColor::Blue),
            );

            Ok(())
        }),
    )
}
