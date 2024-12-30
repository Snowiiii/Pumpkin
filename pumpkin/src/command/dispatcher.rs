use pumpkin_core::permission::PermissionLvl;
use pumpkin_core::text::TextComponent;
use pumpkin_protocol::client::play::CommandSuggestion;

use super::args::ConsumedArgs;

use crate::command::dispatcher::CommandError::{
    GeneralCommandIssue, InvalidConsumption, InvalidRequirement, OtherPumpkin, PermissionDenied,
};
use crate::command::tree::{Command, CommandTree, NodeType, RawArgs};
use crate::command::CommandSender;
use crate::error::PumpkinError;
use crate::server::Server;
use pumpkin_core::text::color::{Color, NamedColor};
use std::collections::{HashMap, HashSet};

#[derive(Debug)]
pub(crate) enum CommandError {
    /// This error means that there was an error while parsing a previously consumed argument.
    /// That only happens when consumption is wrongly implemented, as it should ensure parsing may
    /// never fail.
    InvalidConsumption(Option<String>),

    /// Return this if a condition that a [`Node::Require`] should ensure is met is not met.
    InvalidRequirement,

    PermissionDenied,

    OtherPumpkin(Box<dyn PumpkinError>),

    GeneralCommandIssue(String),
}

impl CommandError {
    pub fn into_string_or_pumpkin_error(self, cmd: &str) -> Result<String, Box<dyn PumpkinError>> {
        match self {
            InvalidConsumption(s) => {
                log::error!("Error while parsing command \"{cmd}\": {s:?} was consumed, but couldn't be parsed");
                Ok("Internal Error (See logs for details)".into())
            }
            InvalidRequirement => {
                log::error!("Error while parsing command \"{cmd}\": a requirement that was expected was not met.");
                Ok("Internal Error (See logs for details)".into())
            }
            PermissionDenied => {
                log::warn!("Permission denied for command \"{cmd}\"");
                Ok("I'm sorry, but you do not have permission to perform this command. Please contact the server administrator if you believe this is an error.".into())
            }
            GeneralCommandIssue(s) => Ok(s),
            OtherPumpkin(e) => Err(e),
        }
    }
}

#[derive(Default)]
pub struct CommandDispatcher {
    pub(crate) commands: HashMap<String, Command>,
    pub(crate) permissions: HashMap<String, PermissionLvl>,
}

/// Stores registered [`CommandTree`]s and dispatches commands to them.
impl CommandDispatcher {
    pub async fn handle_command<'a>(
        &'a self,
        sender: &mut CommandSender<'a>,
        server: &'a Server,
        cmd: &'a str,
    ) {
        if let Err(e) = self.dispatch(sender, server, cmd).await {
            match e.into_string_or_pumpkin_error(cmd) {
                Ok(err) => {
                    sender
                        .send_message(
                            TextComponent::text_string(err)
                                .color_named(pumpkin_core::text::color::NamedColor::Red),
                        )
                        .await;
                }
                Err(pumpkin_error) => {
                    pumpkin_error.log();
                    sender.send_message(TextComponent::text("Unknown internal error occurred while running command. Please see server log").color(Color::Named(NamedColor::Red))).await;
                }
            }
        }
    }

    /// server side suggestions (client side suggestions work independently)
    ///
    /// # todo
    /// - make this less ugly
    /// - do not query suggestions for the same consumer multiple times just because they are on different paths through the tree
    pub(crate) async fn find_suggestions<'a>(
        &'a self,
        src: &mut CommandSender<'a>,
        server: &'a Server,
        cmd: &'a str,
    ) -> Vec<CommandSuggestion<'a>> {
        let mut parts = cmd.split_whitespace();
        let Some(key) = parts.next() else {
            return Vec::new();
        };
        let mut raw_args: Vec<&str> = parts.rev().collect();

        let Ok(tree) = self.get_tree(key) else {
            return Vec::new();
        };

        let mut suggestions = HashSet::new();

        // try paths and collect the nodes that fail
        // todo: make this more fine-grained
        for path in tree.iter_paths() {
            match Self::try_find_suggestions_on_path(src, server, &path, tree, &mut raw_args, cmd)
                .await
            {
                Err(InvalidConsumption(s)) => {
                    log::error!("Error while parsing command \"{cmd}\": {s:?} was consumed, but couldn't be parsed");
                    return Vec::new();
                }
                Err(InvalidRequirement) => {
                    log::error!("Error while parsing command \"{cmd}\": a requirement that was expected was not met.");
                    return Vec::new();
                }
                Err(PermissionDenied) => {
                    log::warn!("Permission denied for command \"{cmd}\"");
                    return Vec::new();
                }
                Err(GeneralCommandIssue(issue)) => {
                    log::error!("Error while parsing command \"{cmd}\": {issue}");
                    return Vec::new();
                }
                Err(OtherPumpkin(e)) => {
                    log::error!("Error while parsing command \"{cmd}\": {e}");
                    return Vec::new();
                }
                Ok(Some(new_suggestions)) => {
                    suggestions.extend(new_suggestions);
                }
                Ok(None) => {}
            }
        }

        let mut suggestions = Vec::from_iter(suggestions);
        suggestions.sort_by(|a, b| a.suggestion.cmp(&b.suggestion));
        suggestions
    }

    /// Execute a command using its corresponding [`CommandTree`].
    pub(crate) async fn dispatch<'a>(
        &'a self,
        src: &mut CommandSender<'a>,
        server: &'a Server,
        cmd: &'a str,
    ) -> Result<(), CommandError> {
        // Other languages dont use the ascii whitespace
        let mut parts = cmd.split_whitespace();
        let key = parts
            .next()
            .ok_or(GeneralCommandIssue("Empty Command".to_string()))?;
        let raw_args: Vec<&str> = parts.rev().collect();

        let Some(permission) = self.permissions.get(key) else {
            return Err(GeneralCommandIssue("Command not found".to_string()));
        };

        if !src.has_permission_lvl(*permission) {
            return Err(PermissionDenied);
        }

        let tree = self.get_tree(key)?;

        // try paths until fitting path is found
        for path in tree.iter_paths() {
            if Self::try_is_fitting_path(src, server, &path, tree, &mut raw_args.clone()).await? {
                return Ok(());
            }
        }
        Err(GeneralCommandIssue(format!(
            "Invalid Syntax. Usage: {tree}"
        )))
    }

    pub(crate) fn get_tree(&self, key: &str) -> Result<&CommandTree, CommandError> {
        let command = self
            .commands
            .get(key)
            .ok_or(GeneralCommandIssue("Command not found".to_string()))?;

        match command {
            Command::Tree(tree) => Ok(tree),
            Command::Alias(target) => {
                let Some(Command::Tree(tree)) = self.commands.get(target) else {
                    log::error!("Error while parsing command alias \"{key}\": pointing to \"{target}\" which is not a valid tree");
                    return Err(GeneralCommandIssue(
                        "Internal Error (See logs for details)".into(),
                    ));
                };
                Ok(tree)
            }
        }
    }

    pub(crate) fn get_permission_lvl(&self, key: &str) -> Option<PermissionLvl> {
        self.permissions.get(key).copied()
    }

    async fn try_is_fitting_path<'a>(
        src: &mut CommandSender<'a>,
        server: &'a Server,
        path: &[usize],
        tree: &'a CommandTree,
        raw_args: &mut RawArgs<'a>,
    ) -> Result<bool, CommandError> {
        let mut parsed_args: ConsumedArgs = HashMap::new();

        for node in path.iter().map(|&i| &tree.nodes[i]) {
            match &node.node_type {
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
                    match consumer.consume(src, server, raw_args).await {
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

    async fn try_find_suggestions_on_path<'a>(
        src: &mut CommandSender<'a>,
        server: &'a Server,
        path: &[usize],
        tree: &'a CommandTree,
        raw_args: &mut RawArgs<'a>,
        input: &'a str,
    ) -> Result<Option<Vec<CommandSuggestion<'a>>>, CommandError> {
        let mut parsed_args: ConsumedArgs = HashMap::new();

        for node in path.iter().map(|&i| &tree.nodes[i]) {
            match &node.node_type {
                NodeType::ExecuteLeaf { .. } => {
                    return Ok(None);
                }
                NodeType::Literal { string, .. } => {
                    if raw_args.pop() != Some(string) {
                        return Ok(None);
                    }
                }
                NodeType::Argument { consumer, name } => {
                    match consumer.consume(src, server, raw_args).await {
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
                        }
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
    pub(crate) fn register(&mut self, tree: CommandTree, permission: PermissionLvl) {
        let mut names = tree.names.iter();

        let primary_name = names.next().expect("at least one name must be provided");

        for name in names {
            self.commands
                .insert(name.to_string(), Command::Alias(primary_name.to_string()));
        }

        self.permissions
            .insert(primary_name.to_string(), permission);
        self.commands
            .insert(primary_name.to_string(), Command::Tree(tree));
    }
}

#[cfg(test)]
mod test {
    use crate::command::{default_dispatcher, tree::CommandTree};
    use pumpkin_core::permission::PermissionLvl;
    #[test]
    fn test_dynamic_command() {
        let mut dispatcher = default_dispatcher();
        let tree = CommandTree::new(["test"], "test_desc");
        dispatcher.register(tree, PermissionLvl::Zero);
    }
}
