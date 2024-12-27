use async_trait::async_trait;
use pumpkin_core::text::{color::NamedColor, hover::HoverEvent, TextComponent};

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

        let message_text = if plugins.is_empty() {
            "There are no loaded plugins."
        } else if plugins.len() == 1 {
            "There is 1 plugin loaded:\n"
        } else {
            &format!("There are {} plugins loaded:\n", plugins.len(),)
        };
        let mut message = TextComponent::text(message_text);

        for (i, (metadata, loaded)) in plugins.clone().into_iter().enumerate() {
            let fmt = if i == plugins.len() - 1 {
                metadata.name.to_string()
            } else {
                format!("{}, ", metadata.name)
            };
            let hover_text = format!(
                "Version: {}\nAuthors: {}\nDescription: {}",
                metadata.version, metadata.authors, metadata.description
            );
            let component = if *loaded {
                TextComponent::text_string(fmt)
                    .color_named(NamedColor::Green)
                    .hover_event(HoverEvent::ShowText(hover_text.into()))
            } else {
                TextComponent::text_string(fmt)
                    .color_named(NamedColor::Red)
                    .hover_event(HoverEvent::ShowText(hover_text.into()))
            };
            message = message.add_child(component);
        }

        sender.send_message(message).await;

        Ok(())
    }
}

pub fn init_command_tree() -> CommandTree {
    CommandTree::new(NAMES, DESCRIPTION).execute(ListExecutor)
}
