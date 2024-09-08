use pumpkin_inventory::OpenContainer;

use crate::commands::tree::CommandTree;

const NAMES: [&str; 2] = ["echest", "enderchest"];

const DESCRIPTION: &str =
    "Show your personal enderchest (this command is used for testing container behaviour)";

pub(crate) fn init_command_tree<'a>() -> CommandTree<'a> {
    CommandTree::new(NAMES, DESCRIPTION).execute(&|sender, server, _| {
        if let Some(player) = sender.as_mut_player() {
            let entity_id = player.entity_id();
            player.open_container = Some(0);
            match server.open_containers.get_mut(&0) {
                Some(ender_chest) => {
                    ender_chest.add_player(entity_id);
                }
                None => {
                    let open_container = OpenContainer::empty(entity_id);
                    server.open_containers.insert(0, open_container);
                }
            }
            player.open_container(server, "minecraft:generic_9x3");
        }

        Ok(())
    })
}
