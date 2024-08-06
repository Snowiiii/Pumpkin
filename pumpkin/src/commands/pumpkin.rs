use pumpkin_protocol::{text::TextComponent, CURRENT_MC_PROTOCOL};

use crate::server::CURRENT_MC_VERSION;

use super::Command;

pub struct PumpkinCommand {}

impl PumpkinCommand {
    pub fn new() -> Self {
        Self {}
    }
}

impl<'a> Command<'a> for PumpkinCommand {
    const NAME: &'a str = "pumpkin";

    const DESCRIPTION: &'a str = "Displays information about Pumpkin";

    fn on_execute(sender: &mut super::CommandSender<'a>, _command: String) {
        let version = env!("CARGO_PKG_VERSION");
        let description = env!("CARGO_PKG_DESCRIPTION");
        sender.send_message(TextComponent::from(format!("Pumpkin {version}, {description} (Minecraft {CURRENT_MC_VERSION}, Protocol {CURRENT_MC_PROTOCOL})")))
    }
}
