use async_trait::async_trait;
use itertools::Itertools;
use pumpkin_core::text::TextComponent;

use crate::{
    command::{
        args::ConsumedArgs, tree::CommandTree, CommandError, CommandExecutor, CommandSender,
    },
    PLUGIN_MANAGER,
};

const NAMES: [&str; 1] = ["plugins"];

const DESCRIPTION: &str = "List all available plugins.";

struct ListExecutor;

#[async_trait]
impl CommandExecutor for ListExecutor {
    async fn execute<'a>(
        &self,
        sender: &mut CommandSender<'a>,
        _server: &crate::server::Server,
        _args: &ConsumedArgs<'a>,
    ) -> Result<(), CommandError> {
        let plugin_manager = PLUGIN_MANAGER.lock().await;
        let plugins = plugin_manager.list_plugins();

        let message = if plugins.is_empty() {
            "There are no loaded plugins."
        } else {
            &format!(
                "There are {} plugins loaded: {}",
                plugins.len(),
                plugins.iter().map(|(plugin, _)| plugin.name).join(", ")
            )
        };

        sender.send_message(TextComponent::text(message)).await;

        Ok(())
    }
}

pub fn init_command_tree<'a>() -> CommandTree<'a> {
    CommandTree::new(NAMES, DESCRIPTION).execute(&ListExecutor)
}
