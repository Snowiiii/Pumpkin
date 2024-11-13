use async_trait::async_trait;
use pumpkin_core::text::TextComponent;

use crate::command::args::arg_postition_block::BlockPosArgumentConsumer;
use crate::command::args::{ConsumedArgs, FindArg};
use crate::command::tree::CommandTree;
use crate::command::tree_builder::{argument, require};
use crate::command::{CommandError, CommandExecutor, CommandSender};
use crate::entity::player::PermissionLvl;

const NAMES: [&str; 2] = ["getblock", "gb"];

const DESCRIPTION: &str = "Print a block to chat. For debug purposes.";

const ARG_BLOCK_POS: &str = "position";

struct GetblockExecutor;

#[async_trait]
impl CommandExecutor for GetblockExecutor {
    async fn execute<'a>(
        &self,
        sender: &mut CommandSender<'a>,
        _server: &crate::server::Server,
        args: &ConsumedArgs<'a>,
    ) -> Result<(), CommandError> {
        let pos = BlockPosArgumentConsumer::find_arg(args, ARG_BLOCK_POS)?;
        let world = sender.world().ok_or(CommandError::InvalidRequirement)?;

        if let Some(block) = world.get_block(pos).await {
            sender
                .send_message(TextComponent::text_string(format!(
                    "Block {} at {pos}",
                    block.name,
                )))
                .await;
        }

        Ok(())
    }
}

pub fn init_command_tree<'a>() -> CommandTree<'a> {
    CommandTree::new(NAMES, DESCRIPTION).with_child(
        require(&|sender| {
            sender.has_permission_lvl(PermissionLvl::Two) && sender.world().is_some()
        })
        .with_child(argument(ARG_BLOCK_POS, &BlockPosArgumentConsumer).execute(&GetblockExecutor)),
    )
}
