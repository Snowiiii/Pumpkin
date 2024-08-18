use pumpkin_text::{color::NamedColor, TextComponent};
use pumpkin_protocol::CURRENT_MC_PROTOCOL;
use crate::server::CURRENT_MC_VERSION;

use crate::commands::tree::CommandTree;

pub(crate) const NAME: &str = "pumpkin";

const DESCRIPTION: &str = "Displays information about Pumpkin";

pub(crate) fn init_command_tree<'a>() -> CommandTree<'a> {
    CommandTree::new(DESCRIPTION).execute(&|sender, _| {
        let version = env!("CARGO_PKG_VERSION");
        let description = env!("CARGO_PKG_DESCRIPTION");
        sender.send_message(TextComponent::from(
            format!("Pumpkin {version}, {description} (Minecraft {CURRENT_MC_VERSION}, Protocol {CURRENT_MC_PROTOCOL})")).color_named(NamedColor::Green)
        );

        Ok(())
    })
}
