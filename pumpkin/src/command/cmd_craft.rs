use crate::commands::tree::CommandTree;
use pumpkin_inventory::{CraftingTable, OpenContainer};

const NAMES: [&str; 1] = ["craft"];

const DESCRIPTION: &str = "Open a crafting table";

#[allow(unused_variables)]

pub fn init_command_tree<'a>() -> CommandTree<'a> {
    CommandTree::new(NAMES, DESCRIPTION).execute(&|sender, server, _| {
        if let Some(player) = sender.as_mut_player() {
            let entity_id = player.entity_id();
            player.open_container.store(Some(1));
            {
                let mut open_containers = server.open_containers.write().unwrap();
                match open_containers.get_mut(&1) {
                    Some(ender_chest) => {
                        ender_chest.add_player(entity_id);
                    }
                    None => {
                        let open_container =
                            OpenContainer::new_empty_container::<CraftingTable>(entity_id);
                        open_containers.insert(1, open_container);
                    }
                }
            }
            {
                let server = server.clone();
                tokio::spawn(async move {
                    player.open_container(&server, "minecraft:crafting").await;
                });
            }
        }

        Ok(())
    })
}
