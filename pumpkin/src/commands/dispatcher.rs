use pumpkin_core::text::TextComponent;

use crate::commands::dispatcher::InvalidTreeError::{
    InvalidConsumptionError, InvalidRequirementError,
};
use crate::commands::tree::{Command, CommandTree, ConsumedArgs, NodeType, RawArgs};
use crate::commands::CommandSender;
use crate::server::Server;
use std::collections::HashMap;
use std::sync::Arc;

#[derive(Debug)]
pub(crate) enum InvalidTreeError {
    /// This error means that there was an error while parsing a previously consumed argument.
    /// That only happens when consumption is wrongly implemented, as it should ensure parsing may
    /// never fail.
    InvalidConsumptionError(Option<String>),

    /// Return this if a condition that a [Node::Require] should ensure is met is not met.
    InvalidRequirementError,
}

#[derive(Default)]
pub struct CommandDispatcher<'a> {
    pub(crate) commands: HashMap<&'a str, Command<'a>>,
}

/// Stores registered [CommandTree]s and dispatches commands to them.
impl<'a> CommandDispatcher<'a> {
    pub fn handle_command(&self, sender: &mut CommandSender, server: &Arc<Server>, cmd: &str) {
        if let Err(err) = self.dispatch(sender, server, cmd) {
            sender.send_message(
                TextComponent::text(&err).color_named(pumpkin_core::text::color::NamedColor::Red),
            )
        }
    }

    /// Execute a command using its corresponding [CommandTree].
    pub(crate) fn dispatch(
        &'a self,
        src: &mut CommandSender,
        server: &Arc<Server>,
        cmd: &str,
    ) -> Result<(), String> {
        let mut parts = cmd.split_ascii_whitespace();
        let key = parts.next().ok_or("Empty Command")?;
        let raw_args: Vec<&str> = parts.rev().collect();

        let tree = self.get_tree(key)?;

        // try paths until fitting path is found
        for path in tree.iter_paths() {
            match Self::try_is_fitting_path(src, server, path, tree, raw_args.clone()) {
                Err(InvalidConsumptionError(s)) => {
                    println!("Error while parsing command \"{cmd}\": {s:?} was consumed, but couldn't be parsed");
                    return Err("Internal Error (See logs for details)".into());
                }
                Err(InvalidRequirementError) => {
                    println!("Error while parsing command \"{cmd}\": a requirement that was expected was not met.");
                    return Err("Internal Error (See logs for details)".into());
                }
                Ok(is_fitting_path) => {
                    if is_fitting_path {
                        return Ok(());
                    }
                }
            }
        }

        Err(format!("Invalid Syntax. Usage: {}", tree))
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

    fn try_is_fitting_path(
        src: &mut CommandSender,
        server: &Arc<Server>,
        path: Vec<usize>,
        tree: &CommandTree,
        mut raw_args: RawArgs,
    ) -> Result<bool, InvalidTreeError> {
        let mut parsed_args: ConsumedArgs = HashMap::new();

        for node in path.iter().map(|&i| &tree.nodes[i]) {
            match node.node_type {
                NodeType::ExecuteLeaf { run } => {
                    return if raw_args.is_empty() {
                        run(src, server, &parsed_args)?;
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
                NodeType::Argument {
                    consumer: consume,
                    name,
                    ..
                } => {
                    if let Some(consumed) = consume(src, &mut raw_args) {
                        parsed_args.insert(name, consumed);
                    } else {
                        return Ok(false);
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
