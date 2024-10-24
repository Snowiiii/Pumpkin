use pumpkin_core::text::TextComponent;

use crate::commands::dispatcher::InvalidTreeError::{
    InvalidConsumptionError, InvalidRequirementError,
};
use crate::commands::tree::{Command, CommandTree, ConsumedArgs, NodeType, RawArgs};
use crate::commands::CommandSender;
use crate::server::Server;
use std::collections::HashMap;

#[derive(Debug)]
pub(crate) enum InvalidTreeError {
    /// This error means that there was an error while parsing a previously consumed argument.
    /// That only happens when consumption is wrongly implemented, as it should ensure parsing may
    /// never fail.
    InvalidConsumptionError(Option<String>),

    /// Return this if a condition that a [`Node::Require`] should ensure is met is not met.
    InvalidRequirementError,
}

#[derive(Default)]
pub struct CommandDispatcher<'a> {
    pub(crate) commands: HashMap<&'a str, Command<'a>>,
}

/// Stores registered [`CommandTree`]s and dispatches commands to them.
impl<'a> CommandDispatcher<'a> {
    pub async fn handle_command(
        &'a self,
        sender: &mut CommandSender<'a>,
        server: &Server,
        cmd: &'a str,
    ) {
        if let Err(err) = self.dispatch(sender, server, cmd).await {
            sender
                .send_message(
                    TextComponent::text_string(err)
                        .color_named(pumpkin_core::text::color::NamedColor::Red),
                )
                .await;
        }
    }

    /// Execute a command using its corresponding [`CommandTree`].
    pub(crate) async fn dispatch(
        &'a self,
        src: &mut CommandSender<'a>,
        server: &Server,
        cmd: &'a str,
    ) -> Result<(), String> {
        // Other languages dont use the ascii whitespace
        let mut parts = cmd.split_whitespace();
        let key = parts.next().ok_or("Empty Command")?;
        let raw_args: Vec<&str> = parts.rev().collect();

        let tree = self.get_tree(key)?;

        // try paths until fitting path is found
        for path in tree.iter_paths() {
            match Self::try_is_fitting_path(src, server, &path, tree, raw_args.clone()).await {
                Err(InvalidConsumptionError(s)) => {
                    println!("Error while parsing command \"{cmd}\": {s:?} was consumed, but couldn't be parsed");
                    return Err("Internal Error (See logs for details)".into());
                }
                Err(InvalidRequirementError) => {
                    println!("Error while parsing command \"{cmd}\": a requirement that was expected was not met.");
                    return Err("Internal Error (See logs for details)".into());
                }
                Ok(is_fitting_path) => match is_fitting_path {
                    Ok(()) => return Ok(()),
                    Err(error) => {
                        // Custom error message or not ?
                        if let Some(error) = error {
                            return Err(error);
                        }
                    }
                },
            }
        }
        Err(format!("Invalid Syntax. Usage: {tree}"))
    }

    pub(crate) fn get_tree(&'a self, key: &str) -> Result<&'a CommandTree<'a>, String> {
        let command = self.commands.get(key).ok_or("Command not found")?;

        match command {
            Command::Tree(tree) => Ok(tree),
            Command::Alias(target) => {
                let Some(Command::Tree(tree)) = &self.commands.get(target) else {
                    println!("Error while parsing command alias \"{key}\": pointing to \"{target}\" which is not a valid tree");
                    return Err("Internal Error (See logs for details)".into());
                };
                Ok(tree)
            }
        }
    }

    async fn try_is_fitting_path(
        src: &mut CommandSender<'a>,
        server: &Server,
        path: &[usize],
        tree: &CommandTree<'a>,
        mut raw_args: RawArgs<'a>,
    ) -> Result<Result<(), Option<String>>, InvalidTreeError> {
        let mut parsed_args: ConsumedArgs = HashMap::new();

        for node in path.iter().map(|&i| &tree.nodes[i]) {
            match node.node_type {
                NodeType::ExecuteLeaf { executor } => {
                    return if raw_args.is_empty() {
                        executor.execute(src, server, &parsed_args).await?;
                        Ok(Ok(()))
                    } else {
                        Ok(Err(None))
                    };
                }
                NodeType::Literal { string, .. } => {
                    if raw_args.pop() != Some(string) {
                        return Ok(Err(None));
                    }
                }
                NodeType::Argument {
                    consumer: consume,
                    name,
                    ..
                } => match consume(src, server, &mut raw_args) {
                    Ok(consumed) => {
                        parsed_args.insert(name, consumed);
                    }
                    Err(err) => {
                        return Ok(Err(err));
                    }
                },
                NodeType::Require { predicate, .. } => {
                    if !predicate(src) {
                        return Ok(Err(None));
                    }
                }
            }
        }

        Ok(Err(None))
    }

    /// Register a command with the dispatcher.
    pub(crate) fn register(&mut self, tree: CommandTree<'a>) {
        let mut names = tree.names.iter();

        let primary_name = names.next().expect("at least one name must be provided");

        for &name in names {
            self.commands.insert(name, Command::Alias(primary_name));
        }

        self.commands.insert(primary_name, Command::Tree(tree));
    }
}
