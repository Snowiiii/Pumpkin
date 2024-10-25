use super::CommandExecutor;
use crate::{commands::CommandSender, server::Server};
use std::collections::{HashMap, VecDeque};

/// see [`crate::commands::tree_builder::argument`]
pub type RawArgs<'a> = Vec<&'a str>;

/// see [`crate::commands::tree_builder::argument`] and [`CommandTree::execute`]/[`crate::commands::tree_builder::NonLeafNodeBuilder::execute`]
pub type ConsumedArgs<'a> = HashMap<&'a str, String>;

/// see [`crate::commands::tree_builder::argument`]
/// Provide value or an Optional error message, If no Error message provided the default will be used
pub type ArgumentConsumer<'a> =
    fn(&CommandSender, &Server, &mut RawArgs) -> Result<String, Option<String>>;

pub struct Node<'a> {
    pub(crate) children: Vec<usize>,
    pub(crate) node_type: NodeType<'a>,
}

pub enum NodeType<'a> {
    ExecuteLeaf {
        executor: &'a dyn CommandExecutor,
    },
    Literal {
        string: &'a str,
    },
    Argument {
        name: &'a str,
        consumer: ArgumentConsumer<'a>,
    },
    Require {
        predicate: &'a (dyn Fn(&CommandSender) -> bool + Sync),
    },
}

pub enum Command<'a> {
    Tree(CommandTree<'a>),
    Alias(&'a str),
}

pub struct CommandTree<'a> {
    pub(crate) nodes: Vec<Node<'a>>,
    pub(crate) children: Vec<usize>,
    pub(crate) names: Vec<&'a str>,
    pub(crate) description: &'a str,
}

impl<'a> CommandTree<'a> {
    /// iterate over all possible paths that end in a [`NodeType::ExecuteLeaf`]
    pub(crate) fn iter_paths(&'a self) -> impl Iterator<Item = Vec<usize>> + 'a {
        let mut todo = VecDeque::<(usize, usize)>::new();

        // add root's children
        todo.extend(self.children.iter().map(|&i| (0, i)));

        TraverseAllPathsIter::<'a> {
            tree: self,
            path: Vec::<usize>::new(),
            todo,
        }
    }
}

struct TraverseAllPathsIter<'a> {
    tree: &'a CommandTree<'a>,
    path: Vec<usize>,
    /// (depth, i)
    todo: VecDeque<(usize, usize)>,
}

impl<'a> Iterator for TraverseAllPathsIter<'a> {
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
