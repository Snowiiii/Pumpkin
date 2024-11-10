use pumpkin_core::text::TextComponent;
use pumpkin_protocol::client::play::CommandSuggestion;

use crate::command::dispatcher::InvalidTreeError::{
    InvalidConsumptionError, InvalidRequirementError,
};
use crate::command::tree::{Command, CommandTree, NodeType, RawArgs};
use crate::command::CommandSender;
use crate::server::Server;
use std::borrow::Cow;
use std::collections::{BTreeSet, HashMap, HashSet};

use super::args::ConsumedArgs;

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
        server: &'a Server,
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

    pub(crate) async fn find_suggestions(
        &'a self,
        src: &mut CommandSender<'a>,
        server: &'a Server,
        cmd: &'a str,
    ) -> Vec<CommandSuggestion<'a>> {
        let mut parts = cmd.split_whitespace();
        let Some(key) = parts.next() else {
            return Vec::new();
        };
        let raw_args: Vec<&str> = parts.rev().collect();

        let Ok(tree) = self.get_tree(key) else {
            return Vec::new();
        };

        let mut suggestions = HashSet::new();

        // try paths and collect the nodes that fail
        for path in tree.iter_paths() {
            match Self::try_find_suggestions_on_path(src, server, &path, tree, raw_args.clone(), cmd).await {
                Err(InvalidConsumptionError(s)) => {
                    log::error!("Error while parsing command \"{cmd}\": {s:?} was consumed, but couldn't be parsed");
                    return Vec::new();
                }
                Err(InvalidRequirementError) => {
                    log::error!("Error while parsing command \"{cmd}\": a requirement that was expected was not met.");
                    return Vec::new();
                }
                Ok(Some(new_suggestions)) => {
                    dbg!(&new_suggestions);
                    suggestions.extend(new_suggestions);
                    dbg!(&suggestions);
                }
                Ok(None) => {}
            }
        }

        let mut suggestions = Vec::from_iter(suggestions);
        suggestions.sort_by(|a, b| a.suggestion.cmp(&b.suggestion));
        suggestions
    }
    
    /// Execute a command using its corresponding [`CommandTree`].
    pub(crate) async fn dispatch(
        &'a self,
        src: &mut CommandSender<'a>,
        server: &'a Server,
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
                    log::error!("Error while parsing command \"{cmd}\": {s:?} was consumed, but couldn't be parsed");
                    return Err("Internal Error (See logs for details)".into());
                }
                Err(InvalidRequirementError) => {
                    log::error!("Error while parsing command \"{cmd}\": a requirement that was expected was not met.");
                    return Err("Internal Error (See logs for details)".into());
                }
                Ok(is_fitting_path) => {
                    if is_fitting_path {
                        return Ok(());
                    }
                }
            }
        }
        Err(format!("Invalid Syntax. Usage: {tree}"))
    }

    pub(crate) fn get_tree(&'a self, key: &str) -> Result<&'a CommandTree<'a>, String> {
        let command: &Command<'a> = self.commands.get(key).ok_or("Command not found")?;

        match command {
            Command::Tree(tree) => Ok(tree),
            Command::Alias(target) => {
                let Some(Command::Tree(tree)) = &self.commands.get(target) else {
                    log::error!("Error while parsing command alias \"{key}\": pointing to \"{target}\" which is not a valid tree");
                    return Err("Internal Error (See logs for details)".into());
                };
                Ok(tree)
            }
        }
    }

    async fn try_is_fitting_path(
        src: &mut CommandSender<'a>,
        server: &'a Server,
        path: &[usize],
        tree: &CommandTree<'a>,
        mut raw_args: RawArgs<'a>,
    ) -> Result<bool, InvalidTreeError> {
        let mut parsed_args: ConsumedArgs = HashMap::new();

        for node in path.iter().map(|&i| &tree.nodes[i]) {
            match node.node_type {
                NodeType::ExecuteLeaf { executor } => {
                    return if raw_args.is_empty() {
                        executor.execute(src, server, &parsed_args).await?;
                        Ok(true)
                    } else {
                        Ok(false)
                    };
                }
                NodeType::Literal { string, .. } => {
                    if raw_args.pop() != Some(string) {
                        return Ok(false);
                    }
                }
                NodeType::Argument { consumer, name, .. } => {
                    match consumer.consume(src, server, &mut raw_args).await {
                        Some(consumed) => {
                            parsed_args.insert(name, consumed);
                        }
                        None => return Ok(false),
                    }
                }
                NodeType::Require { predicate, .. } => {
                    if !predicate(src) {
                        return Ok(false);
                    }
                }
            }
        }

        Ok(false)
    }

    async fn try_find_suggestions_on_path(
        src: &mut CommandSender<'a>,
        server: &'a Server,
        path: &[usize],
        tree: &CommandTree<'a>,
        mut raw_args: RawArgs<'a>,
        input: &'a str,
    ) -> Result<Option<Vec<CommandSuggestion<'a>>>, InvalidTreeError> {
        let mut parsed_args: ConsumedArgs = HashMap::new();

        for node in path.iter().map(|&i| &tree.nodes[i]) {
            match node.node_type {
                NodeType::ExecuteLeaf { .. } => {
                    return Ok(None);
                }
                NodeType::Literal { string, .. } => {
                    if raw_args.pop() != Some(string) {
                        return Ok(None);
                    }
                }
                NodeType::Argument { consumer, name } => {
                    match consumer.consume(src, server, &mut raw_args).await {
                        Some(consumed) => {
                            parsed_args.insert(name, consumed);
                        }
                        None => {
                            return if raw_args.is_empty() {
                                let suggestions = consumer.suggest(src, server, input).await?;
                                Ok(suggestions)
                            } else {
                                Ok(None)
                            };
                        }, 
                    }
                }
                NodeType::Require { predicate, .. } => {
                    if !predicate(src) {
                        return Ok(None);
                    }
                }
            }
        }

        Ok(None)
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
