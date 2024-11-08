use pumpkin_macros::client_packet;

use crate::{bytebuf::ByteBuffer, ClientPacket, VarInt};

/// this is a bit ugly, but ClientPacket depends on CommandTree in pumpkin(bin), from where ClientPacket cannot be implemented
#[client_packet("play:commands")]
pub struct CCommands<'a> {
    pub nodes: Vec<ProtoNode<'a>>,
    pub root_node_index: VarInt,
}

impl<'a> CCommands<'a> {
    pub fn new(nodes: Vec<ProtoNode<'a>>, root_node_index: impl Into<VarInt>) -> Self {
        Self {
            nodes,
            root_node_index: root_node_index.into(),
        }
    }
}

impl<'a> ClientPacket for CCommands<'a> {
    fn write(&self, bytebuf: &mut ByteBuffer) {
        bytebuf.put_list(&self.nodes, |bytebuf, node: &ProtoNode| {
            node.write_to(bytebuf)
        });
        bytebuf.put_var_int(&self.root_node_index);
    }
}

pub struct ProtoNode<'a> {
    pub children: Vec<VarInt>,
    pub node_type: ProtoNodeType<'a>,
}

pub enum ProtoNodeType<'a> {
    Root,
    Literal { name: &'a str, is_executable: bool },
    Argument { name: &'a str, is_executable: bool },
}

impl<'a> ProtoNode<'a> {
    const FLAG_IS_EXECUTABLE: i8 = 4;
    const FLAG_HAS_REDIRECT: i8 = 8;
    const FLAG_HAS_SUGGESTION_TYPE: i8 = 16;

    pub fn new_root(children: Vec<VarInt>) -> Self {
        Self {
            children,
            node_type: ProtoNodeType::Root,
        }
    }

    pub fn new_literal(children: Vec<VarInt>, name: &'a str) -> Self {
        Self {
            children,
            node_type: ProtoNodeType::Literal {
                name,
                is_executable: true,
            },
        }
    }

    pub fn new_argument(children: Vec<VarInt>, name: &'a str) -> Self {
        Self {
            children,
            node_type: ProtoNodeType::Argument {
                name,
                is_executable: true, // todo
            },
        }
    }

    /// https://wiki.vg/Command_Data
    pub fn write_to(&self, bytebuf: &mut ByteBuffer) {
        // flags
        let flags = match self.node_type {
            ProtoNodeType::Root => 0,
            ProtoNodeType::Literal {
                name: _,
                is_executable,
            } => {
                let mut n = 1;
                if is_executable {
                    n |= Self::FLAG_IS_EXECUTABLE
                }
                n
            }
            ProtoNodeType::Argument {
                name: _,
                is_executable,
            } => {
                let mut n = 2 | Self::FLAG_HAS_SUGGESTION_TYPE;
                if is_executable {
                    n |= Self::FLAG_IS_EXECUTABLE
                }
                n
            }
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
            ProtoNodeType::Argument { name, .. } | ProtoNodeType::Literal { name, .. } => {
                bytebuf.put_string_len(name, 32767)
            }
            ProtoNodeType::Root => {}
        }

        // parser id + properties
        match self.node_type {
            ProtoNodeType::Argument { .. } => {
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
