use async_trait::async_trait;
use pumpkin_core::text::{color::NamedColor, TextComponent};
use pumpkin_protocol::CURRENT_MC_PROTOCOL;

use crate::{commands::tree::CommandTree, server::CURRENT_MC_VERSION};

use super::CommandExecutor;

const NAMES: [&str; 1] = ["pumpkin"];

const DESCRIPTION: &str = "Display information about Pumpkin.";

struct PumpkinExecutor {}

#[async_trait]
impl CommandExecutor for PumpkinExecutor {
    async fn execute(
        &self,
        sender: &mut super::CommandSender,
        _server: &crate::server::Server,
        _args: &super::tree::ConsumedArgs,
    ) -> Result<(), super::dispatcher::InvalidTreeError> {
        let version = env!("CARGO_PKG_VERSION");
        let description = env!("CARGO_PKG_DESCRIPTION");

        sender.send_message(TextComponent::text(
             &format!("Pumpkin {version}, {description} (Minecraft {CURRENT_MC_VERSION}, Protocol {CURRENT_MC_PROTOCOL})")
         ).color_named(NamedColor::Green)).await;

        Ok(())
    }
}

pub fn init_command_tree<'a>() -> CommandTree<'a> {
    CommandTree::new(NAMES, DESCRIPTION).execute(&PumpkinExecutor {})
}
