use pumpkin_protocol::{bytebuf::ByteBuffer, client::play::CCommands, VarInt};

use super::{dispatcher::CommandDispatcher, tree::{Node, NodeType}};

fn write_dispatcher_to_bytebuf<'a>(dispatcher: &&CommandDispatcher<'a>, bytebuf: &'a mut ByteBuffer) {
    let mut proto_nodes: Vec<ProtoNode> = Vec::new();
    let mut root_children: Vec<VarInt> = Vec::new();

    for (key, _) in &dispatcher.commands {
        let Ok(tree) = dispatcher.get_tree(key) else {
            continue;
        };

        let mut offset = proto_nodes.len() as i32;

        for node in &tree.nodes {
            match ProtoNode::from_node(node, offset) {
                Some(proto_node) => proto_nodes.push(proto_node),
                None => offset -= 1
            };
        }

        root_children.push(proto_nodes.len().into());

        let tree_children = tree.children.iter().map(|i| VarInt(*i as i32 + offset)).collect();
        proto_nodes.push(ProtoNode::new_literal(tree_children, key, true));
    }

    let root_index: VarInt = proto_nodes.len().into();
    proto_nodes.push(ProtoNode::new_root(root_children));
    bytebuf.put_list(&proto_nodes, |bytebuf, node: &ProtoNode| node.write_to(bytebuf));
    bytebuf.put_var_int(&root_index);
}

struct ProtoNode<'a> {
    children: Vec<VarInt>,
    node_type: ProtoNodeType<'a>,
}

enum ProtoNodeType<'a> {
    Root,
    Literal {
        name: &'a str,
        is_executable: bool,
    },
    Argument {
        name: &'a str,
        is_executable: bool,
    },
}

impl <'a>ProtoNode<'a> {
    const FLAG_IS_EXECUTABLE: i8 = 4;
    const FLAG_HAS_REDIRECT: i8 = 8;
    const FLAG_HAS_SUGGESTION_TYPE: i8 = 16;

    /// TODO: do this properly
    fn from_node(node: &Node<'a>, offset: i32) -> Option<Self> {
        let children = node.children.iter().map(|i| VarInt(*i as i32 + offset)).collect();

        match node.node_type {
            NodeType::Argument { name, .. } => Some(ProtoNode::new_argument(children, name, true)),
            NodeType::Literal { string, .. } => Some(ProtoNode::new_literal(children, string, true)),
            NodeType::Require { .. } => Some(ProtoNode::new_literal(children, "require", true)),
            _ => Some(ProtoNode::new_literal(children, "execute", true)),
        }
    }

    fn new_root(children: Vec<VarInt>) -> Self {
        Self { children, node_type: ProtoNodeType::Root }
    }

    fn new_literal(children: Vec<VarInt>, name: &'a str, is_executable: bool) -> Self {
        Self { children, node_type: ProtoNodeType::Literal { name, is_executable } }
    }

    fn new_argument(children: Vec<VarInt>, name: &'a str, is_executable: bool) -> Self {
        Self { children, node_type: ProtoNodeType::Argument { name, is_executable } }
    }

    /// https://wiki.vg/Command_Data
    fn write_to(&self, bytebuf: &mut ByteBuffer) {
        // flags 
        let flags = match self.node_type {
            ProtoNodeType::Root => 0,
            ProtoNodeType::Literal { name: _, is_executable } => {
                let mut n = 1;
                if is_executable {
                    n |= Self::FLAG_IS_EXECUTABLE
                }
                n
            },
            ProtoNodeType::Argument { name: _, is_executable } => {
                let mut n = 2 | Self::FLAG_HAS_SUGGESTION_TYPE;
                if is_executable {
                    n |= Self::FLAG_IS_EXECUTABLE
                }
                n
            },
        };
        bytebuf.put_i8(flags);

        // child count + children
        bytebuf.put_list(&self.children, |bytebuf, child| bytebuf.put_var_int(child));
        
        // redirect node
        if flags & Self::FLAG_HAS_REDIRECT != 0 {
            bytebuf.put_var_int(&1.into());
        }

        // name
        match self.node_type {
            ProtoNodeType::Argument { name, ..} | ProtoNodeType::Literal { name, ..} => bytebuf.put_string_len(name, 32767),
            ProtoNodeType::Root => {}
        }

        // parser id + properties
        match self.node_type {
            ProtoNodeType::Argument {..} => {
                bytebuf.put_var_int(&5.into()); // string arg type has id 5
                bytebuf.put_var_int(&0.into()); // match single word only
            }
            _ => {}
        }

        // suggestion type
        if flags & Self::FLAG_HAS_SUGGESTION_TYPE != 0 {
            bytebuf.put_string("minecraft:ask_server");
        }
    }
}

pub(crate) trait NewCCommandsPacket<'a> {
    fn new(dispatcher: &'a CommandDispatcher<'a>) -> CCommands<&'a CommandDispatcher<'a>>;
}

impl <'a>NewCCommandsPacket<'a> for CCommands<&'a CommandDispatcher<'a>> {
    fn new(dispatcher: &'a CommandDispatcher<'a>) -> CCommands<&'a CommandDispatcher<'a>> {
        Self {
            data: dispatcher,
            write_fn: write_dispatcher_to_bytebuf,
        }
    }
}