use crate::commands::tree::CommandTree;
use crate::commands::CommandExecutor;
use async_trait::async_trait;
use pumpkin_inventory::{CraftingTable, OpenContainer};

const NAMES: [&str; 1] = ["craft"];

const DESCRIPTION: &str = "Open a crafting table";

struct CraftingTableExecutor {}

#[async_trait]
impl CommandExecutor for CraftingTableExecutor {
    async fn execute<'a>(
        &self,
        sender: &mut super::CommandSender<'a>,
        server: &crate::server::Server,
        _args: &super::tree::ConsumedArgs<'a>,
    ) -> Result<(), super::dispatcher::InvalidTreeError> {
        if let Some(player) = sender.as_player() {
            let entity_id = player.entity_id();
            player.open_container.store(Some(1));
            {
                let mut open_containers = server.open_containers.write().await;
                if let Some(ender_chest) = open_containers.get_mut(&1) {
                    ender_chest.add_player(entity_id);
                } else {
                    let open_container =
                        OpenContainer::new_empty_container::<CraftingTable>(entity_id);
                    open_containers.insert(1, open_container);
                }
            }
            player.open_container(server, "minecraft:crafting").await;
        }

        Ok(())
    }
}

pub fn init_command_tree<'a>() -> CommandTree<'a> {
    CommandTree::new(NAMES, DESCRIPTION).execute(&CraftingTableExecutor {})
}
