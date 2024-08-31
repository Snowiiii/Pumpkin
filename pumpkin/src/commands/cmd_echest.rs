use pumpkin_inventory::OpenContainer;

use crate::commands::tree::CommandTree;

const NAMES: [&str; 2] = ["echest", "enderchest"];

const DESCRIPTION: &str =
    "Show your personal enderchest (this command is used for testing container behaviour)";

pub(crate) fn init_command_tree<'a>() -> CommandTree<'a> {
    CommandTree::new(NAMES, DESCRIPTION).execute(&|sender, server, _| {
        if let Some(player) = sender.as_mut_player() {
            if let Some(container) = server
                .open_containers
                .entry(0)
                .or_insert(OpenContainer::empty(player.entity_id()))
                .try_open(player.entity_id())
            {
                player.open_container(
                    container.window_type(),
                    "minecraft:generic_9x3",
                    Some("Ender Chest"),
                    Some(container.all_slots_ref()),
                    None,
                );
            }
        }

        Ok(())
    })
}
