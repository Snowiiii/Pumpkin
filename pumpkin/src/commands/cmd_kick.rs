use crate::commands::arg_player::{consume_arg_player, parse_arg_player};
use crate::commands::tree::CommandTree;
use crate::commands::tree::RawArgs;
use crate::commands::tree_builder::argument;
use crate::commands::CommandSender;
use pumpkin_core::text::{color::NamedColor, TextComponent};

const NAMES: [&str; 1] = ["kick"];
const DESCRIPTION: &str = "Kicks the target player from the server.";

const ARG_TARGET: &str = "target";

pub fn consume_arg_target(_src: &CommandSender, args: &mut RawArgs) -> Option<String> {
    consume_arg_player(_src, args)
}

pub fn init_command_tree<'a>() -> CommandTree<'a> {
    CommandTree::new(NAMES, DESCRIPTION).with_child(
        argument(ARG_TARGET, consume_arg_target).execute(&|sender, server, args| {
            let target = parse_arg_player(sender, server, ARG_TARGET, args)?;
            target.kick(TextComponent::text("Kicked by an operator"));

            sender.send_message(
                TextComponent::text("Player has been kicked.").color_named(NamedColor::Blue),
            );

            Ok(())
        }),
    )
}
