use async_trait::async_trait;
use pumpkin_core::text::{color::NamedColor, TextComponent};
use pumpkin_protocol::CURRENT_MC_PROTOCOL;

use crate::{
    command::{
        args::ConsumedArgs, tree::CommandTree, CommandError, CommandExecutor, CommandSender,
    },
    server::CURRENT_MC_VERSION,
    GIT_VERSION,
};

const NAMES: [&str; 2] = ["pumpkin", "version"];

const DESCRIPTION: &str = "Display information about Pumpkin.";

struct PumpkinExecutor;

const CARGO_PKG_VERSION: &str = env!("CARGO_PKG_VERSION");
const CARGO_PKG_DESCRIPTION: &str = env!("CARGO_PKG_DESCRIPTION");

#[async_trait]
impl CommandExecutor for PumpkinExecutor {
    async fn execute<'a>(
        &self,
        sender: &mut CommandSender<'a>,
        _server: &crate::server::Server,
        _args: &ConsumedArgs<'a>,
    ) -> Result<(), CommandError> {
        sender
            .send_message(
                TextComponent::text(&format!(
                    "Pumpkin {} ({}), {} (Minecraft {}, Protocol {})",
                    CARGO_PKG_VERSION,
                    GIT_VERSION,
                    CARGO_PKG_DESCRIPTION,
                    CURRENT_MC_VERSION,
                    CURRENT_MC_PROTOCOL
                ))
                .color_named(NamedColor::Green),
            )
            .await;
        Ok(())
    }
}

pub fn init_command_tree<'a>() -> CommandTree<'a> {
    CommandTree::new(NAMES, DESCRIPTION).execute(&PumpkinExecutor)
}
