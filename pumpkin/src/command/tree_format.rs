use crate::command::tree::{CommandTree, Node, NodeType};
use std::collections::VecDeque;
use std::fmt::{Display, Formatter, Write};

trait IsVisible {
    /// whether node should be printed in help command/usage hint
    fn is_visible(&self) -> bool;
}

impl IsVisible for Node {
    fn is_visible(&self) -> bool {
        matches!(
            self.node_type,
            NodeType::Literal { .. } | NodeType::Argument { .. }
        )
    }
}

impl Display for Node {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match &self.node_type {
            NodeType::Literal { string } => {
                f.write_str(string)?;
            }
            NodeType::Argument { name, .. } => {
                f.write_char('<')?;
                f.write_str(name)?;
                f.write_char('>')?;
            }
            _ => {}
        };

        Ok(())
    }
}

fn flatten_require_nodes(nodes: &[Node], children: &[usize]) -> Vec<usize> {
    let mut new_children = Vec::with_capacity(children.len());

    for &i in children {
        let node = &nodes[i];
        match &node.node_type {
            NodeType::Require { .. } => {
                new_children.extend(flatten_require_nodes(nodes, node.children.as_slice()));
            }
            _ => new_children.push(i),
        }
    }

    new_children
}

impl Display for CommandTree {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_char('/')?;
        f.write_str(&self.names[0])?;

        let mut todo = VecDeque::<&[usize]>::with_capacity(self.children.len());
        todo.push_back(&self.children);

        loop {
            let Some(children) = todo.pop_front() else {
                break;
            };

            let flattened_children = flatten_require_nodes(&self.nodes, children);
            let visible_children = flattened_children
                .iter()
                .copied()
                .filter(|&i| self.nodes[i].is_visible())
                .collect::<Vec<_>>();

            if visible_children.is_empty() {
                break;
            };

            f.write_char(' ')?;

            let is_optional = flattened_children
                .iter()
                .map(|&i| &self.nodes[i].node_type)
                .any(|node| matches!(node, NodeType::ExecuteLeaf { .. }));

            if is_optional {
                f.write_char('[')?;
            }

            match visible_children.as_slice() {
                [] => unreachable!(),
                [i] => {
                    let node = &self.nodes[*i];

                    node.fmt(f)?;

                    todo.push_back(&node.children);
                }
                _ => {
                    // todo: handle cases where one of these nodes has visible children
                    f.write_char('(')?;

                    let mut iter = visible_children.iter().map(|&i| &self.nodes[i]);

                    if let Some(node) = iter.next() {
                        node.fmt(f)?;
                    }

                    for node in iter {
                        f.write_str(" | ")?;
                        node.fmt(f)?;
                    }

                    f.write_char(')')?;
                }
            }

            if is_optional {
                f.write_char(']')?;
            }
        }

        Ok(())
    }
}
