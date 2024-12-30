use std::sync::Arc;

use super::args::DefaultNameArgConsumer;
use super::CommandExecutor;
use crate::command::args::ArgumentConsumer;
use crate::command::tree::{CommandTree, Node, NodeType};
use crate::command::CommandSender;

impl CommandTree {
    /// Add a child [Node] to the root of this [`CommandTree`].
    #[must_use]
    pub fn with_child(mut self, child: impl NodeBuilder) -> Self {
        let node = child.build(&mut self);
        self.children.push(self.nodes.len());
        self.nodes.push(node);
        self
    }

    /// provide at least one name
    #[must_use]
    pub fn new(
        names: impl IntoIterator<Item: Into<String>>,
        description: impl Into<String>,
    ) -> Self {
        let names_vec = names.into_iter().map(Into::into).collect();

        Self {
            nodes: Vec::new(),
            children: Vec::new(),
            names: names_vec,
            description: description.into(),
        }
    }

    /// Executes if a command terminates at this [Node], i.e. without any arguments.
    ///
    /// [`ConsumedArgs`] maps the names of all
    /// arguments to the result of their consumption, i.e. a string that can be parsed to the
    /// desired type.
    ///
    /// Also see [`NonLeafNodeBuilder::execute`].
    #[must_use]
    pub fn execute(mut self, executor: impl CommandExecutor + 'static + Send) -> Self {
        let node = Node {
            node_type: NodeType::ExecuteLeaf {
                executor: Arc::new(executor),
            },
            children: Vec::new(),
        };

        self.children.push(self.nodes.len());
        self.nodes.push(node);

        self
    }
}

pub trait NodeBuilder {
    fn build(self, tree: &mut CommandTree) -> Node;
}

struct LeafNodeBuilder {
    node_type: NodeType,
}

impl NodeBuilder for LeafNodeBuilder {
    fn build(self, _tree: &mut CommandTree) -> Node {
        Node {
            children: Vec::new(),
            node_type: self.node_type,
        }
    }
}

pub struct NonLeafNodeBuilder {
    node_type: NodeType,
    child_nodes: Vec<NonLeafNodeBuilder>,
    leaf_nodes: Vec<LeafNodeBuilder>,
}

impl NodeBuilder for NonLeafNodeBuilder {
    fn build(self, tree: &mut CommandTree) -> Node {
        let mut child_indices = Vec::new();

        for node_builder in self.child_nodes {
            let node = node_builder.build(tree);
            child_indices.push(tree.nodes.len());
            tree.nodes.push(node);
        }

        for node_builder in self.leaf_nodes {
            let node = node_builder.build(tree);
            child_indices.push(tree.nodes.len());
            tree.nodes.push(node);
        }

        Node {
            children: child_indices,
            node_type: self.node_type,
        }
    }
}

impl NonLeafNodeBuilder {
    /// Add a child [Node] to this one.
    #[must_use]
    pub fn with_child(mut self, child: Self) -> Self {
        self.child_nodes.push(child);
        self
    }

    /// Executes if a command terminates at this [Node].
    ///
    /// [`ConsumedArgs`] maps the names of all
    /// arguments to the result of their consumption, i.e. a string that can be parsed to the
    /// desired type.
    ///
    /// Also see [`CommandTree::execute`].
    #[must_use]
    pub fn execute(mut self, executor: impl CommandExecutor + 'static + Send) -> Self {
        self.leaf_nodes.push(LeafNodeBuilder {
            node_type: NodeType::ExecuteLeaf {
                executor: Arc::new(executor),
            },
        });

        self
    }
}

/// Matches a sting literal.
#[must_use]
pub fn literal(string: impl Into<String>) -> NonLeafNodeBuilder {
    NonLeafNodeBuilder {
        node_type: NodeType::Literal {
            string: string.into(),
        },
        child_nodes: Vec::new(),
        leaf_nodes: Vec::new(),
    }
}

/// ```name``` identifies this argument in [`ConsumedArgs`].
///
/// ```consumer: ArgumentConsumer``` has the purpose of validating arguments. Conversion may start
/// here, as long as the result remains a [String] (e.g. convert offset to absolute position actual
/// coordinates), because the result of this function will be passed to following
/// [`NonLeafNodeBuilder::execute`] nodes in a [`ConsumedArgs`] instance. It must remove consumed arg(s)
/// from [`RawArgs`] and return them. It must return None if [`RawArgs`] are invalid. [`RawArgs`] is
/// reversed, so [`Vec::pop`] can be used to obtain args in ltr order.
pub fn argument(
    name: impl Into<String>,
    consumer: impl ArgumentConsumer + 'static + Send,
) -> NonLeafNodeBuilder {
    NonLeafNodeBuilder {
        node_type: NodeType::Argument {
            name: name.into(),
            consumer: Arc::new(consumer),
        },
        child_nodes: Vec::new(),
        leaf_nodes: Vec::new(),
    }
}

/// same as [`crate::command::tree_builder::argument`], but uses default arg name of consumer
pub fn argument_default_name(
    consumer: impl DefaultNameArgConsumer + 'static + Send,
) -> NonLeafNodeBuilder {
    NonLeafNodeBuilder {
        node_type: NodeType::Argument {
            name: consumer.default_name(),
            consumer: Arc::new(consumer),
        },
        child_nodes: Vec::new(),
        leaf_nodes: Vec::new(),
    }
}

/// ```predicate``` should return ```false``` if requirement for reaching following [Node]s is not
/// met.
pub fn require(
    predicate: impl Fn(&CommandSender) -> bool + Send + Sync + 'static,
) -> NonLeafNodeBuilder {
    NonLeafNodeBuilder {
        node_type: NodeType::Require {
            predicate: Arc::new(predicate),
        },
        child_nodes: Vec::new(),
        leaf_nodes: Vec::new(),
    }
}
