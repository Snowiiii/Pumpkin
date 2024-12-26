use crate::command::args::ArgumentConsumer;
use crate::command::tree::{CommandTree, Node, NodeType};
use crate::command::CommandSender;

use super::args::DefaultNameArgConsumer;
use super::CommandExecutor;

impl<'a> CommandTree<'a> {
    /// Add a child [Node] to the root of this [`CommandTree`].
    #[must_use]
    pub fn with_child(mut self, child: impl NodeBuilder<'a>) -> Self {
        let node = child.build(&mut self);
        self.children.push(self.nodes.len());
        self.nodes.push(node);
        self
    }

    /// provide at least one name
    #[must_use]
    pub fn new<const NAME_COUNT: usize>(
        names: [&'a str; NAME_COUNT],
        description: &'a str,
    ) -> Self {
        assert!(NAME_COUNT > 0);

        let mut names_vec = Vec::with_capacity(NAME_COUNT);

        for name in names {
            names_vec.push(name);
        }

        Self {
            nodes: Vec::new(),
            children: Vec::new(),
            names: names_vec,
            description,
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
    pub fn execute(mut self, executor: &'a dyn CommandExecutor) -> Self {
        let node = Node {
            node_type: NodeType::ExecuteLeaf { executor },
            children: Vec::new(),
        };

        self.children.push(self.nodes.len());
        self.nodes.push(node);

        self
    }
}

pub trait NodeBuilder<'a> {
    fn build(self, tree: &mut CommandTree<'a>) -> Node<'a>;
}

struct LeafNodeBuilder<'a> {
    node_type: NodeType<'a>,
}

impl<'a> NodeBuilder<'a> for LeafNodeBuilder<'a> {
    fn build(self, _tree: &mut CommandTree<'a>) -> Node<'a> {
        Node {
            children: Vec::new(),
            node_type: self.node_type,
        }
    }
}

pub struct NonLeafNodeBuilder<'a> {
    node_type: NodeType<'a>,
    child_nodes: Vec<NonLeafNodeBuilder<'a>>,
    leaf_nodes: Vec<LeafNodeBuilder<'a>>,
}

impl<'a> NodeBuilder<'a> for NonLeafNodeBuilder<'a> {
    fn build(self, tree: &mut CommandTree<'a>) -> Node<'a> {
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

impl<'a> NonLeafNodeBuilder<'a> {
    /// Add a child [Node] to this one.
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
    pub fn execute(mut self, executor: &'a dyn CommandExecutor) -> Self {
        self.leaf_nodes.push(LeafNodeBuilder {
            node_type: NodeType::ExecuteLeaf { executor },
        });

        self
    }
}

/// Matches a sting literal.
pub const fn literal(string: &str) -> NonLeafNodeBuilder {
    NonLeafNodeBuilder {
        node_type: NodeType::Literal { string },
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
pub fn argument<'a>(name: &'a str, consumer: &'a dyn ArgumentConsumer) -> NonLeafNodeBuilder<'a> {
    NonLeafNodeBuilder {
        node_type: NodeType::Argument { name, consumer },
        child_nodes: Vec::new(),
        leaf_nodes: Vec::new(),
    }
}

/// same as [`crate::command::tree_builder::argument`], but uses default arg name of consumer
pub fn argument_default_name(consumer: &dyn DefaultNameArgConsumer) -> NonLeafNodeBuilder<'_> {
    NonLeafNodeBuilder {
        node_type: NodeType::Argument {
            name: consumer.default_name(),
            consumer: consumer.get_argument_consumer(),
        },
        child_nodes: Vec::new(),
        leaf_nodes: Vec::new(),
    }
}

/// ```predicate``` should return ```false``` if requirement for reaching following [Node]s is not
/// met.
pub fn require(predicate: &(dyn Fn(&CommandSender) -> bool + Sync)) -> NonLeafNodeBuilder {
    NonLeafNodeBuilder {
        node_type: NodeType::Require { predicate },
        child_nodes: Vec::new(),
        leaf_nodes: Vec::new(),
    }
}
