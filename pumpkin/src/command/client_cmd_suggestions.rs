use std::sync::Arc;

use pumpkin_protocol::client::play::{CCommands, ProtoNode, ProtoNodeType};
use tokio::sync::RwLock;

use crate::entity::player::Player;

use super::{
    dispatcher::CommandDispatcher,
    tree::{Node, NodeType},
};

pub async fn send_c_commands_packet(player: &Arc<Player>, dispatcher: &RwLock<CommandDispatcher>) {
    let cmd_src = super::CommandSender::Player(player.clone());
    let mut first_level = Vec::new();

    let dispatcher = dispatcher.read().await;
    for key in dispatcher.commands.keys() {
        let Ok(tree) = dispatcher.get_tree(key) else {
            continue;
        };

        let Some(permission) = dispatcher.get_permission_lvl(key) else {
            continue;
        };

        if !cmd_src.has_permission_lvl(permission) {
            continue;
        }

        let (is_executable, child_nodes) =
            nodes_to_proto_node_builders(&cmd_src, &tree.nodes, &tree.children);

        let proto_node = ProtoNodeBuilder {
            child_nodes,
            node_type: ProtoNodeType::Literal {
                name: key,
                is_executable,
            },
        };

        first_level.push(proto_node);
    }

    let root = ProtoNodeBuilder {
        child_nodes: first_level,
        node_type: ProtoNodeType::Root,
    };

    let mut proto_nodes = Vec::new();
    let root_node_index = root.build(&mut proto_nodes);

    let packet = CCommands::new(proto_nodes, root_node_index.into());
    player.client.send_packet(&packet).await;
}

#[derive(Debug)]
struct ProtoNodeBuilder<'a> {
    child_nodes: Vec<ProtoNodeBuilder<'a>>,
    node_type: ProtoNodeType<'a>,
}

impl<'a> ProtoNodeBuilder<'a> {
    fn build(self, buffer: &mut Vec<ProtoNode<'a>>) -> usize {
        let mut children = Vec::new();
        for node in self.child_nodes {
            let i = node.build(buffer);
            children.push(i.into());
        }

        let i = buffer.len();
        buffer.push(ProtoNode {
            children,
            node_type: self.node_type,
        });
        i
    }
}

fn nodes_to_proto_node_builders<'a>(
    cmd_src: &super::CommandSender,
    nodes: &'a [Node],
    children: &[usize],
) -> (bool, Vec<ProtoNodeBuilder<'a>>) {
    let mut child_nodes = Vec::new();
    let mut is_executable = false;

    for i in children {
        let node = &nodes[*i];
        match &node.node_type {
            NodeType::Argument { name, consumer } => {
                let (node_is_executable, node_children) =
                    nodes_to_proto_node_builders(cmd_src, nodes, &node.children);
                child_nodes.push(ProtoNodeBuilder {
                    child_nodes: node_children,
                    node_type: ProtoNodeType::Argument {
                        name,
                        is_executable: node_is_executable,
                        parser: consumer.get_client_side_parser(),
                        override_suggestion_type: consumer
                            .get_client_side_suggestion_type_override(),
                    },
                });
            }

            NodeType::Literal { string, .. } => {
                let (node_is_executable, node_children) =
                    nodes_to_proto_node_builders(cmd_src, nodes, &node.children);
                child_nodes.push(ProtoNodeBuilder {
                    child_nodes: node_children,
                    node_type: ProtoNodeType::Literal {
                        name: string,
                        is_executable: node_is_executable,
                    },
                });
            }

            NodeType::ExecuteLeaf { .. } => is_executable = true,

            NodeType::Require { predicate } => {
                if predicate(cmd_src) {
                    let (node_is_executable, node_children) =
                        nodes_to_proto_node_builders(cmd_src, nodes, &node.children);
                    if node_is_executable {
                        is_executable = true;
                    }
                    child_nodes.extend(node_children);
                }
            }
        }
    }

    (is_executable, child_nodes)
}
