use async_trait::async_trait;
use pumpkin_core::text::{color::NamedColor, TextComponent};
use pumpkin_protocol::CURRENT_MC_PROTOCOL;

use crate::{
    command::{
        tree::CommandTree, tree::ConsumedArgs, CommandExecutor, CommandSender, InvalidTreeError,
    },
    server::CURRENT_MC_VERSION,
};

const NAMES: [&str; 1] = ["pumpkin"];

const DESCRIPTION: &str = "Display information about Pumpkin.";

struct PumpkinExecutor;

#[async_trait]
impl CommandExecutor for PumpkinExecutor {
    async fn execute<'a>(
        &self,
        sender: &mut CommandSender<'a>,
        _server: &crate::server::Server,
        _args: &ConsumedArgs<'a>,
    ) -> Result<(), InvalidTreeError> {
        let version = env!("CARGO_PKG_VERSION");
        let description = env!("CARGO_PKG_DESCRIPTION");

        sender.send_message(TextComponent::text(
             &format!("Pumpkin {version}, {description} (Minecraft {CURRENT_MC_VERSION}, Protocol {CURRENT_MC_PROTOCOL})")
         ).color_named(NamedColor::Green)).await;

        Ok(())
    }
}

pub fn init_command_tree<'a>() -> CommandTree<'a> {
    CommandTree::new(NAMES, DESCRIPTION).execute(&PumpkinExecutor)
}
