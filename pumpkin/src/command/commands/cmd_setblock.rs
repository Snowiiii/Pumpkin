use async_trait::async_trait;
use pumpkin_core::text::color::NamedColor;
use pumpkin_core::text::TextComponent;

use crate::command::args::arg_block::BlockArgumentConsumer;
use crate::command::args::arg_position_block::BlockPosArgumentConsumer;
use crate::command::args::{ConsumedArgs, FindArg};
use crate::command::tree::CommandTree;
use crate::command::tree_builder::{argument, literal, require};
use crate::command::{CommandError, CommandExecutor, CommandSender};
use crate::entity::player::PermissionLvl;

const NAMES: [&str; 1] = ["setblock"];

const DESCRIPTION: &str = "Place a block.";

const ARG_BLOCK: &str = "block";
const ARG_BLOCK_POS: &str = "position";

#[derive(Clone, Copy)]
enum Mode {
    /// with particles + item drops
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
        let block_state_id = block.default_state_id;
        let pos = BlockPosArgumentConsumer::find_arg(args, ARG_BLOCK_POS)?;
        let mode = self.0;
        // TODO: allow console to use the command (seed sender.world)
        let world = sender.world().ok_or(CommandError::InvalidRequirement)?;

        let success = match mode {
            Mode::Destroy => {
                world.break_block(pos, None).await;
                world.set_block_state(pos, block_state_id).await;
                true
            }
            Mode::Replace => {
                world.set_block_state(pos, block_state_id).await;
                true
            }
            Mode::Keep => match world.get_block_state(pos).await {
                Ok(old_state) if old_state.air => {
                    world.set_block_state(pos, block_state_id).await;
                    true
                }
                Ok(_) => false,
                Err(e) => return Err(CommandError::OtherPumpkin(e.into())),
            },
        };

        sender
            .send_message(if success {
                TextComponent::text_string(format!("Placed block {} at {pos}", block.name,))
            } else {
                TextComponent::text_string(format!("Kept block at {pos}"))
                    .color_named(NamedColor::Red)
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
            argument(ARG_BLOCK_POS, &BlockPosArgumentConsumer).with_child(
                argument(ARG_BLOCK, &BlockArgumentConsumer)
                    .with_child(literal("replace").execute(&SetblockExecutor(Mode::Replace)))
                    .with_child(literal("destroy").execute(&SetblockExecutor(Mode::Destroy)))
                    .with_child(literal("keep").execute(&SetblockExecutor(Mode::Keep)))
                    .execute(&SetblockExecutor(Mode::Replace)),
            ),
        ),
    )
}
