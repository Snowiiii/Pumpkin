use pumpkin_core::text::{color::NamedColor, TextComponent};
use crate::commands::tree::CommandTree;
use crate::commands::arg_player::{consume_arg_player, parse_arg_player};
use crate::commands::tree::RawArgs;
use crate::commands::CommandSender;
use crate::commands::tree_builder::argument;

const NAMES: [&str; 1] = ["kill"];
const DESCRIPTION: &str = "Kills a target player.";

const ARG_TARGET: &str = "target";

pub fn consume_arg_target(_src: &CommandSender, args: &mut RawArgs) -> Option<String> {
    consume_arg_player(_src, args)
}

pub(crate) fn init_command_tree<'a>() -> CommandTree<'a> {
    CommandTree::new(NAMES, DESCRIPTION)
        .with_child(
            argument(ARG_TARGET, consume_arg_target)
                .execute(&|sender, server, args| {
                    let target = parse_arg_player(sender, server, ARG_TARGET, args)?;
                    target.entity.kill();
                    target.send_system_message(TextComponent::text(
                        "You have been killed."
                    ).color_named(NamedColor::Red));

                    sender.send_message(TextComponent::text(
                        "Player has been killed."
                    ).color_named(NamedColor::Blue));

                    Ok(())
                })
        )
}
