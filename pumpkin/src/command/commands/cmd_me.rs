use async_trait::async_trait;
use pumpkin_core::text::TextComponent;
use pumpkin_protocol::client::play::CSystemChatMessage;

use crate::{
    command::{
        args::{arg_message::MsgArgConsumer, Arg, ConsumedArgs},
        tree::CommandTree,
        tree_builder::{argument, require},
        CommandError, CommandExecutor, CommandSender,
    },
    entity::player::PermissionLvl,
};
use CommandError::InvalidConsumption;

const NAMES: [&str; 1] = ["me"];

const DESCRIPTION: &str = "Broadcasts a narrative message about yourself.";

const ARG_ACTION: &str = "action";

struct MeExecutor;

#[async_trait]
impl CommandExecutor for MeExecutor {
    async fn execute<'a>(
        &self,
        sender: &mut CommandSender<'a>,
        server: &crate::server::Server,
        args: &ConsumedArgs<'a>,
    ) -> Result<(), CommandError> {
        let Some(Arg::Msg(action)) = args.get(ARG_ACTION) else {
            return Err(InvalidConsumption(Some(ARG_ACTION.into())));
        };

        server
            .broadcast_packet_all(&CSystemChatMessage::new(
                &TextComponent::text(&format!("* {sender} {action}")),
                false,
            ))
            .await;
        Ok(())
    }
}

pub fn init_command_tree<'a>() -> CommandTree<'a> {
    CommandTree::new(NAMES, DESCRIPTION).with_child(
        require(&|sender| sender.has_permission_lvl(PermissionLvl::Zero))
            .with_child(argument(ARG_ACTION, &MsgArgConsumer).execute(&MeExecutor)),
    )
}
