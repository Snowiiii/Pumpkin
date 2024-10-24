use async_trait::async_trait;
use pumpkin_inventory::OpenContainer;

use crate::commands::tree::CommandTree;

use super::RunFunctionType;

const NAMES: [&str; 2] = ["echest", "enderchest"];

const DESCRIPTION: &str =
    "Show your personal enderchest (this command is used for testing container behaviour)";

struct EchestExecutor {}

#[async_trait]
impl RunFunctionType for EchestExecutor {
    async fn execute(
        &self,
        sender: &mut super::CommandSender,
        server: &crate::server::Server,
        _args: &super::tree::ConsumedArgs,
    ) -> Result<(), super::dispatcher::InvalidTreeError> {
        if let Some(player) = sender.as_player() {
            let entity_id = player.entity_id();
            player.open_container.store(Some(0));
            {
                let mut open_containers = server.open_containers.write().await;
                if let Some(ender_chest) = open_containers.get_mut(&0) {
                    ender_chest.add_player(entity_id);
                } else {
                    let open_container = OpenContainer::empty(entity_id);
                    open_containers.insert(0, open_container);
                }
            }
            player.open_container(server, "minecraft:generic_9x3").await;
        }

        Ok(())
    }
}

pub fn init_command_tree<'a>() -> CommandTree<'a> {
    CommandTree::new(NAMES, DESCRIPTION).execute(&EchestExecutor {})
}
