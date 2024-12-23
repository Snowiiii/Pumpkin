use async_trait::async_trait;
use pumpkin_core::text::{color::NamedColor, hover::HoverEvent, TextComponent};

use crate::{
    command::{
        args::{arg_simple::SimpleArgConsumer, Arg, ConsumedArgs},
        tree::CommandTree,
        tree_builder::{argument, literal, require},
        CommandError, CommandExecutor, CommandSender,
    },
    entity::player::PermissionLvl,
    PLUGIN_MANAGER,
};

use crate::command::CommandError::InvalidConsumption;

const NAMES: [&str; 1] = ["plugin"];

const DESCRIPTION: &str = "Manage plugins.";

const PLUGIN_NAME: &str = "plugin_name";

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

struct LoadExecutor;

#[async_trait]
impl CommandExecutor for LoadExecutor {
    async fn execute<'a>(
        &self,
        sender: &mut CommandSender<'a>,
        _server: &crate::server::Server,
        args: &ConsumedArgs<'a>,
    ) -> Result<(), CommandError> {
        let Some(Arg::Simple(plugin_name)) = args.get(PLUGIN_NAME) else {
            return Err(InvalidConsumption(Some(PLUGIN_NAME.into())));
        };
        let mut plugin_manager = PLUGIN_MANAGER.lock().await;

        if plugin_manager.is_plugin_loaded(plugin_name) {
            sender
                .send_message(
                    TextComponent::text_string(format!("Plugin {} is already loaded", plugin_name))
                        .color_named(NamedColor::Red),
                )
                .await;
            return Ok(());
        }

        let result = plugin_manager.load_plugin(plugin_name).await;

        match result {
            Ok(_) => {
                sender
                    .send_message(
                        TextComponent::text_string(format!(
                            "Plugin {} loaded successfully",
                            plugin_name
                        ))
                        .color_named(NamedColor::Green),
                    )
                    .await;
            }
            Err(e) => {
                sender
                    .send_message(
                        TextComponent::text_string(format!(
                            "Failed to load plugin {}: {}",
                            plugin_name, e
                        ))
                        .color_named(NamedColor::Red),
                    )
                    .await;
            }
        }

        Ok(())
    }
}

struct UnloadExecutor;

#[async_trait]
impl CommandExecutor for UnloadExecutor {
    async fn execute<'a>(
        &self,
        sender: &mut CommandSender<'a>,
        _server: &crate::server::Server,
        args: &ConsumedArgs<'a>,
    ) -> Result<(), CommandError> {
        let Some(Arg::Simple(plugin_name)) = args.get(PLUGIN_NAME) else {
            return Err(InvalidConsumption(Some(PLUGIN_NAME.into())));
        };
        let mut plugin_manager = PLUGIN_MANAGER.lock().await;

        if !plugin_manager.is_plugin_loaded(plugin_name) {
            sender
                .send_message(
                    TextComponent::text_string(format!("Plugin {} is not loaded", plugin_name))
                        .color_named(NamedColor::Red),
                )
                .await;
            return Ok(());
        }

        let result = plugin_manager.unload_plugin(plugin_name).await;

        match result {
            Ok(_) => {
                sender
                    .send_message(
                        TextComponent::text_string(format!(
                            "Plugin {} unloaded successfully",
                            plugin_name
                        ))
                        .color_named(NamedColor::Green),
                    )
                    .await;
            }
            Err(e) => {
                sender
                    .send_message(
                        TextComponent::text_string(format!(
                            "Failed to unload plugin {}: {}",
                            plugin_name, e
                        ))
                        .color_named(NamedColor::Red),
                    )
                    .await;
            }
        }

        Ok(())
    }
}

pub fn init_command_tree<'a>() -> CommandTree<'a> {
    CommandTree::new(NAMES, DESCRIPTION).with_child(
        require(&|sender| sender.has_permission_lvl(PermissionLvl::Three))
            .with_child(
                literal("load")
                    .with_child(argument(PLUGIN_NAME, &SimpleArgConsumer).execute(&LoadExecutor)),
            )
            .with_child(
                literal("unload")
                    .with_child(argument(PLUGIN_NAME, &SimpleArgConsumer).execute(&UnloadExecutor)),
            )
            .with_child(literal("list").execute(&ListExecutor)),
    )
}
