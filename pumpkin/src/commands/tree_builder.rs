use crate::commands::CommandSender;
use crate::commands::dispatcher::InvalidTreeError;
use crate::commands::tree::{ArgumentConsumer, ConsumedArgs, Node, NodeType, CommandTree};

impl <'a> CommandTree<'a> {
    pub fn with_child(mut self, child: impl NodeBuilder<'a>) -> Self {
        let node = child.build(&mut self);
        self.children.push(self.nodes.len());
        self.nodes.push(node);
        self
    }
    
    pub fn new(description: &'a str) -> Self {
        Self {
            nodes: Vec::new(),
            children: Vec::new(),
            description,
        }
    }

    /// Executes if a command terminates at this [Node], i.e. without any arguments.
    pub fn execute(mut self, run: &'a dyn Fn(&mut CommandSender, &ConsumedArgs) -> Result<(), InvalidTreeError>) -> Self {
        let node = Node {
            node_type: NodeType::ExecuteLeaf {
                run,
            },
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

impl <'a>NodeBuilder<'a> for LeafNodeBuilder<'a> {
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
    leaf_nodes: Vec<LeafNodeBuilder<'a>>
}

impl <'a>NodeBuilder<'a> for NonLeafNodeBuilder<'a> {
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

impl <'a>NonLeafNodeBuilder<'a> {
    pub fn with_child(mut self, child: NonLeafNodeBuilder<'a>) -> Self {
        self.child_nodes.push(child);
        self
    }

    /// Executes if a command terminates at this [Node].
    pub fn execute(mut self, run: &'a dyn Fn(&mut CommandSender, &ConsumedArgs) -> Result<(), InvalidTreeError>) -> Self {
        self.leaf_nodes.push(LeafNodeBuilder {
            node_type: NodeType::ExecuteLeaf {
                run
            },
        });
        
        self
    }
}

/// Matches a sting literal.
pub fn literal(string: &str) -> NonLeafNodeBuilder {
    NonLeafNodeBuilder {
        node_type: NodeType::Literal {
            string
        },
        child_nodes: Vec::new(),
        leaf_nodes: Vec::new(),
    }
}

/// ```name``` identifies this argument.
/// ```consumer: [ArgumentConsumer]``` has the purpose of validating arguments. Conversion may start here, as long as the result remains a [String] (e.g. convert offset to absolute position actual coordinates). It must remove consumed arg(s) from [RawArgs] and return them. It must return None if [RawArgs] are invalid. [RawArgs] is reversed, so [Vec::pop] can be used to obtain args in ltr order.
pub fn argument<'a>(name: &'a str, consumer: ArgumentConsumer) -> NonLeafNodeBuilder<'a> {
    NonLeafNodeBuilder {
        node_type: NodeType::Argument {
            name,
            consumer,
        },
        child_nodes: Vec::new(),
        leaf_nodes: Vec::new(),
    }
}

/// ```predicate``` should return ```false``` if requirement for reaching following [Node]s is not met.
pub fn require(predicate: &dyn Fn(&CommandSender) -> bool) -> NonLeafNodeBuilder {
    NonLeafNodeBuilder {
        node_type: NodeType::Require {
            predicate
        },
        child_nodes: Vec::new(),
        leaf_nodes: Vec::new(),
    }
}
