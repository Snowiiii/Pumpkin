use std::collections::HashMap;
use crate::commands::CommandSender;
use crate::commands::dispatcher::InvalidTreeError::{InvalidConsumptionError, InvalidRequirementError};
use crate::commands::tree::{ConsumedArgs, NodeType, RawArgs, CommandTree};

#[derive(Debug)]
pub(crate) enum InvalidTreeError {
    /// This error means that there was an error while parsing a previously consumed argument.
    /// That only happens when consumption is wrongly implemented, as it should ensure parsing may 
    /// never fail.
    InvalidConsumptionError(Option<String>),

    /// Return this if a condition that a [Node::Require] should ensure is met is not met.
    InvalidRequirementError,
}

pub(crate) struct CommandDispatcher<'a> {
    pub(crate) commands: HashMap<&'a str, CommandTree<'a>>
}

impl <'a> CommandDispatcher<'a> {
    pub(crate) fn dispatch(&'a self, src: &mut CommandSender, cmd: &str) -> Result<(), String> {

        let mut parts = cmd.split_ascii_whitespace();
        let key = parts.next().ok_or("Empty Command")?;
        let raw_args: Vec<&str> = parts.rev().collect();

        let tree = self.commands.get(key).ok_or("Command not found")?;

        for path in tree.iter_paths() {
            match Self::try_path(src, path, tree, raw_args.clone()) {
                Err(InvalidConsumptionError(s)) => {
                    println!("Error while parsing command \"{cmd}\": {s:?} was consumed, but couldn't be parsed");
                    return Err("Internal Error (See logs for details)".into())
                },
                Err(InvalidRequirementError) => {
                    println!("Error while parsing command \"{cmd}\": a requirement that was expected was not met.");
                    return Err("Internal Error (See logs for details)".into())
                },
                Ok(fitting_path) => {
                    if fitting_path { return Ok(()) }
                }
            }
        }

        Err(format!("Invalid Syntax. Usage:{}", tree.paths_formatted(key)))
    }

    fn try_path(src: &mut CommandSender, path: Vec<usize>, tree: &CommandTree, mut raw_args: RawArgs) -> Result<bool, InvalidTreeError> {

        let mut parsed_args: ConsumedArgs = HashMap::new();

        for node in path.iter().map(|&i| &tree.nodes[i]) {
            match node.node_type {
                NodeType::ExecuteLeaf { run } => {
                    return if raw_args.is_empty() {
                        run(src, &parsed_args)?;
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
                NodeType::Argument { consumer: consume, name, .. } => {
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
        };

        Ok(false)
    }
}