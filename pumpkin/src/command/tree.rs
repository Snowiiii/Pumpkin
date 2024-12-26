use super::{args::ArgumentConsumer, CommandExecutor};
use crate::command::CommandSender;
use std::{collections::VecDeque, fmt::Debug, sync::Arc};

/// see [`crate::commands::tree_builder::argument`]
pub type RawArgs<'a> = Vec<&'a str>;

#[derive(Debug, Clone)]
pub struct Node {
    pub(crate) children: Vec<usize>,
    pub(crate) node_type: NodeType,
}

#[derive(Clone)]
pub enum NodeType {
    ExecuteLeaf {
        executor: Arc<dyn CommandExecutor + Send>,
    },
    Literal {
        string: String,
    },
    Argument {
        name: String,
        consumer: Arc<dyn ArgumentConsumer + Send>,
    },
    Require {
        predicate: Arc<dyn Fn(&CommandSender) -> bool + Send + Sync>,
    },
}

impl Debug for NodeType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ExecuteLeaf { .. } => f
                .debug_struct("ExecuteLeaf")
                .field("executor", &"..")
                .finish(),
            Self::Literal { string } => f.debug_struct("Literal").field("string", string).finish(),
            Self::Argument { name, .. } => f
                .debug_struct("Argument")
                .field("name", name)
                .field("consumer", &"..")
                .finish(),
            Self::Require { .. } => f.debug_struct("Require").field("predicate", &"..").finish(),
        }
    }
}

pub enum Command {
    Tree(CommandTree),
    Alias(String),
}

#[derive(Debug, Clone)]
pub struct CommandTree {
    pub(crate) nodes: Vec<Node>,
    pub(crate) children: Vec<usize>,
    pub(crate) names: Vec<String>,
    pub(crate) description: String,
}

impl CommandTree {
    /// iterate over all possible paths that end in a [`NodeType::ExecuteLeaf`]
    pub(crate) fn iter_paths(&self) -> impl Iterator<Item = Vec<usize>> + use<'_> {
        let mut todo = VecDeque::<(usize, usize)>::new();

        // add root's children
        todo.extend(self.children.iter().map(|&i| (0, i)));

        TraverseAllPathsIter {
            tree: self,
            path: Vec::<usize>::new(),
            todo,
        }
    }
}

struct TraverseAllPathsIter<'a> {
    tree: &'a CommandTree,
    path: Vec<usize>,
    /// (depth, i)
    todo: VecDeque<(usize, usize)>,
}

impl Iterator for TraverseAllPathsIter<'_> {
    type Item = Vec<usize>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let (depth, i) = self.todo.pop_front()?;
            let node = &self.tree.nodes[i];

            // add new children to front
            self.todo.reserve(node.children.len());
            node.children
                .iter()
                .rev()
                .for_each(|&c| self.todo.push_front((depth + 1, c)));

            // update path
            while self.path.len() > depth {
                self.path.pop();
            }
            self.path.push(i);

            if let NodeType::ExecuteLeaf { .. } = node.node_type {
                return Some(self.path.clone());
            }
        }
    }
}
