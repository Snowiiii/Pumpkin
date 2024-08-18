use crate::commands::tree::CommandTree;
use crate::commands::tree_builder::require;

pub(crate) const NAME: &str = "stop";

const DESCRIPTION: &str = "Stops the server";

pub(crate) fn init_command_tree<'a>() -> CommandTree<'a> {
    CommandTree::new(DESCRIPTION).with_child(
        require(&|sender| { sender.permission_lvl() >= 4 })
            .execute(&|_sender, _args| {
                std::process::exit(0)
            })
    )
}