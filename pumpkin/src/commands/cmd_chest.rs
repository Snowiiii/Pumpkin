use pumpkin_inventory::WindowType;

use crate::commands::tree::CommandTree;

pub(crate) const NAME: &str = "chest";

const DESCRIPTION: &str = "Open a chest containing lots of diamond blocks";

pub(crate) fn init_command_tree<'a>() -> CommandTree<'a> {
    CommandTree::new(DESCRIPTION).execute(&|sender, _| {
        sender.as_mut_player().unwrap().open_container(WindowType::Generic3x3,"minecraft:generic_9x3",None);
        Ok(())
    })
}
