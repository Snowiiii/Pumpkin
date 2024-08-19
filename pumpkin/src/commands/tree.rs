use std::collections::{HashMap, VecDeque};

use crate::commands::CommandSender;
use crate::commands::dispatcher::InvalidTreeError;

pub(crate) type RawArgs<'a> = Vec<&'a str>;

pub(crate) type ConsumedArgs<'a> = HashMap<&'a str, String>;

pub(crate) type ArgumentConsumer<'a> = fn(&CommandSender, &mut RawArgs) -> Option<String>;

pub(crate) struct Node<'a> {
    pub(crate) children: Vec<usize>,
    pub(crate) node_type: NodeType<'a>,
}

pub(crate) enum NodeType<'a> {
    ExecuteLeaf {
        run: &'a dyn Fn(&mut CommandSender, &ConsumedArgs) -> Result<(), InvalidTreeError>,
    },
    Literal {
        string: &'a str,
    },
    Argument {
        name: &'a str,
        consumer: ArgumentConsumer<'a>,
    },
    Require {
        predicate: &'a dyn Fn(&CommandSender) -> bool,
    }
}

pub(crate) struct CommandTree<'a> {
    pub(crate) nodes: Vec<Node<'a>>,
    pub(crate) children: Vec<usize>,
    pub(crate) description: &'a str,
}

impl <'a> CommandTree<'a> {
    /// iterate over all possible paths that end in a [NodeType::ExecuteLeaf]
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

    pub(crate) fn paths_formatted(&'a self, name: &str) -> String {
        let paths: Vec<Vec<&NodeType>> = self.iter_paths()
            .map(|path| path.iter().map(|&i| &self.nodes[i].node_type).collect())
            .collect();
        
        let len = paths.iter()
            .map(|path| path.iter()
                .map(|node| match node {
                    NodeType::ExecuteLeaf { .. } => 0,
                    NodeType::Literal { string } => string.len() + 1,
                    NodeType::Argument { name, .. } => name.len() + 3,
                    NodeType::Require { .. } => 0,
                })
                .sum::<usize>() + name.len() + 2
            )
            .sum::<usize>();
        
        let mut s = String::with_capacity(len);

        for path in paths.iter() {
            s.push(if paths.len() > 1 { '\n' } else { ' ' });
            s.push('/');
            s.push_str(name);
            for node in path {
                match node {
                    NodeType::Literal { string } => {
                        s.push(' ');
                        s.push_str(string);
                    }
                    NodeType::Argument { name, .. } => {
                        s.push(' ');
                        s.push('<');
                        s.push_str(name);
                        s.push('>');
                    }
                    _ => {}
                }
            }
        }
        
        s
    }
}

struct TraverseAllPathsIter<'a> {
    tree: &'a CommandTree<'a>,
    path: Vec<usize>,
    /// (depth, i)
    todo: VecDeque<(usize, usize)>,
}

impl <'a>Iterator for TraverseAllPathsIter<'a> {

    type Item = Vec<usize>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let (depth, i) = self.todo.pop_front()?;
            let node = &self.tree.nodes[i];

            // add new children to front
            self.todo.reserve(node.children.len());
            node.children.iter().rev().for_each(|&c| self.todo.push_front((depth + 1, c)));

            // update path
            while self.path.len() > depth {
                self.path.pop();
            };
            self.path.push(i);

            if let NodeType::ExecuteLeaf { .. } = node.node_type {
                return Some(self.path.clone());
            }
        }
    }
}

