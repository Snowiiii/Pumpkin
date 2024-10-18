use crate::commands::tree::CommandTree;
use crate::commands::tree::RawArgs;
use crate::commands::tree_builder::argument;
use crate::commands::CommandSender;
use pumpkin_core::text::{color::NamedColor, TextComponent};

const NAMES: [&str; 1] = ["say"];
const DESCRIPTION: &str = "Sends a message to all players.";

const ARG_CONTENT: &str = "content";

pub fn consume_arg_content(_src: &CommandSender, args: &mut RawArgs) -> Option<String> {
    args.pop().map(|v| v.to_string())
}

pub fn init_command_tree<'a>() -> CommandTree<'a> {
    CommandTree::new(NAMES, DESCRIPTION).with_child(
        argument(ARG_CONTENT, consume_arg_content).execute(&|sender, server, args| {
            if let Some(content) = args.get("content") {
                let message = &format!("[Console]: {content}");
                let message = TextComponent::text(message).color_named(NamedColor::Blue);

                server.broadcast_message(message.clone());
                sender.send_message(message);
            } else {
                sender.send_message(
                    TextComponent::text("Please provide a message: say [content]")
                        .color_named(NamedColor::Red),
                );
            }

            Ok(())
        }),
    )
}
