use async_trait::async_trait;
use pumpkin_core::text::TextComponent;

use crate::command::args::arg_command::CommandTreeArgumentConsumer;
use crate::command::args::{Arg, ConsumedArgs};
use crate::command::dispatcher::CommandError;
use crate::command::dispatcher::CommandError::InvalidConsumption;
use crate::command::tree::{Command, CommandTree};
use crate::command::tree_builder::argument;
use crate::command::{CommandExecutor, CommandSender};
use crate::server::Server;

const NAMES: [&str; 3] = ["help", "h", "?"];

const DESCRIPTION: &str = "Print a help message.";

const ARG_COMMAND: &str = "command";

struct CommandHelpExecutor;

#[async_trait]
impl CommandExecutor for CommandHelpExecutor {
    async fn execute<'a>(
        &self,
        sender: &mut CommandSender<'a>,
        _server: &Server,
        args: &ConsumedArgs<'a>,
    ) -> Result<(), CommandError> {
        let Some(Arg::CommandTree(tree)) = args.get(&ARG_COMMAND) else {
            return Err(InvalidConsumption(Some(ARG_COMMAND.into())));
        };

        sender
            .send_message(TextComponent::text(&format!(
                "{} - {} Usage: {}",
                tree.names.join("/"),
                tree.description,
                tree
            )))
            .await;

        Ok(())
    }
}

struct BaseHelpExecutor;

#[async_trait]
impl CommandExecutor for BaseHelpExecutor {
    async fn execute<'a>(
        &self,
        sender: &mut CommandSender<'a>,
        server: &Server,
        _args: &ConsumedArgs<'a>,
    ) -> Result<(), CommandError> {
        let mut keys: Vec<&str> = server.command_dispatcher.commands.keys().copied().collect();
        keys.sort_unstable();

        for key in keys {
            let Command::Tree(tree) = &server.command_dispatcher.commands[key] else {
                continue;
            };

            sender
                .send_message(TextComponent::text(&format!(
                    "{} - {} Usage: {}",
                    tree.names.join("/"),
                    tree.description,
                    tree
                )))
                .await;
        }

        Ok(())
    }
}

pub fn init_command_tree<'a>() -> CommandTree<'a> {
    CommandTree::new(NAMES, DESCRIPTION)
        .with_child(
            argument(ARG_COMMAND, &CommandTreeArgumentConsumer).execute(&CommandHelpExecutor),
        )
        .execute(&BaseHelpExecutor)
}
