use async_trait::async_trait;
use pumpkin_core::text::TextComponent;

use crate::command::args::arg_block::BlockArgumentConsumer;
use crate::command::args::arg_item::ItemArgumentConsumer;
use crate::command::args::arg_postition_block::BlockPosArgumentConsumer;
use crate::command::args::{ConsumedArgs, FindArg, FindArgDefaultName};
use crate::command::tree::CommandTree;
use crate::command::tree_builder::{argument, argument_default_name, literal, require};
use crate::command::{CommandError, CommandExecutor, CommandSender};
use crate::entity::player::PermissionLvl;

const NAMES: [&str; 1] = ["setblock"];

const DESCRIPTION: &str = "Place a block.";

const ARG_BLOCK: &str = "block";

#[derive(Clone, Copy)]
enum Mode {
    /// with particles
    Destroy,

    /// only replaces air
    Keep,

    /// default; without particles
    Replace,
}

struct SetblockExecutor(Mode);

#[async_trait]
impl CommandExecutor for SetblockExecutor {
    async fn execute<'a>(
        &self,
        sender: &mut CommandSender<'a>,
        _server: &crate::server::Server,
        args: &ConsumedArgs<'a>,
    ) -> Result<(), CommandError> {
        let block = BlockArgumentConsumer::find_arg(args, ARG_BLOCK)?;
        let pos = BlockPosArgumentConsumer.find_arg_default_name(args)?;
        let mode = self.0;
        let world = sender.world().ok_or(CommandError::InvalidRequirement)?;

        let success = match mode {
            Mode::Destroy => {
                world.break_block(pos).await;
                world.set_block(pos, block.id).await;
                true
            }
            Mode::Replace => {
                world.set_block(pos, block.id).await;
                true
            }
            Mode::Keep => {
                match world.get_block(pos).await {
                    // todo: include other air blocks (I think there's cave air etc?)
                    Some(old_block) if old_block.id == 0 => {
                        world.set_block(pos, block.id).await;
                        true
                    }
                    _ => false,
                }
            }
        };

        sender
            .send_message(if success {
                TextComponent::text_string(format!("Placed block {} at {pos}", block.name,))
            } else {
                TextComponent::text_string(format!("Kept block at {pos}"))
            })
            .await;

        Ok(())
    }
}

pub fn init_command_tree<'a>() -> CommandTree<'a> {
    CommandTree::new(NAMES, DESCRIPTION).with_child(
        require(&|sender| {
            sender.has_permission_lvl(PermissionLvl::Two) && sender.world().is_some()
        })
        .with_child(
            argument_default_name(&BlockPosArgumentConsumer).with_child(
                argument(ARG_BLOCK, &ItemArgumentConsumer)
                    .with_child(literal("replace").execute(&SetblockExecutor(Mode::Replace)))
                    .with_child(literal("destroy").execute(&SetblockExecutor(Mode::Destroy)))
                    .with_child(literal("keep").execute(&SetblockExecutor(Mode::Keep)))
                    .execute(&SetblockExecutor(Mode::Replace)),
            ),
        ),
    )
}
