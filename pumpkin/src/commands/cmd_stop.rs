use pumpkin_core::text::color::NamedColor;
use pumpkin_core::text::TextComponent;

use crate::commands::tree::CommandTree;
use crate::commands::tree_builder::require;

const NAMES: [&str; 1] = ["stop"];

const DESCRIPTION: &str = "Stop the server.";

pub(crate) fn init_command_tree<'a>() -> CommandTree<'a> {
    CommandTree::new(NAMES, DESCRIPTION).with_child(
        require(&|sender| sender.permission_lvl() >= 4).execute(&|sender, _args| {
            sender
                .send_message(TextComponent::text("Stopping Server").color_named(NamedColor::Red));
            std::process::exit(0)
        }),
    )
}
