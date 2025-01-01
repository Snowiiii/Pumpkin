use crate::{
    command::{
        args::{arg_players::PlayersArgumentConsumer, Arg, ConsumedArgs},
        tree::CommandTree,
        tree_builder::argument,
        CommandError, CommandExecutor, CommandSender,
    },
    data::{op_data::OPERATOR_CONFIG, SaveJSONConfiguration},
};
use async_trait::async_trait;
use pumpkin_core::text::TextComponent;
use CommandError::InvalidConsumption;

const NAMES: [&str; 1] = ["deop"];
const DESCRIPTION: &str = "Revokes operator status from a player.";
const ARG_TARGET: &str = "player";

struct DeopExecutor;

#[async_trait]
impl CommandExecutor for DeopExecutor {
    async fn execute<'a>(
        &self,
        sender: &mut CommandSender<'a>,
        server: &crate::server::Server,
        args: &ConsumedArgs<'a>,
    ) -> Result<(), CommandError> {
        let mut config = OPERATOR_CONFIG.write().await;

        let Some(Arg::Players(targets)) = args.get(&ARG_TARGET) else {
            return Err(InvalidConsumption(Some(ARG_TARGET.into())));
        };

        // from the command tree, the command can only be executed with one player
        let player = &targets[0];

        if let Some(op_index) = config
            .ops
            .iter()
            .position(|o| o.uuid == player.gameprofile.id)
        {
            config.ops.remove(op_index);
        }
        config.save();

        player
            .set_permission_lvl(
                pumpkin_core::PermissionLvl::Zero,
                &server.command_dispatcher,
            )
            .await;

        let player_name = &player.gameprofile.name;
        let message = format!("Revoked {player_name}'s server operator status.");
        let msg = TextComponent::text(message);
        sender.send_message(msg).await;
        player
            .send_system_message(&TextComponent::text("You are no longer a server operator."))
            .await;

        Ok(())
    }
}

pub fn init_command_tree() -> CommandTree {
    CommandTree::new(NAMES, DESCRIPTION)
        .with_child(argument(ARG_TARGET, PlayersArgumentConsumer).execute(DeopExecutor))
}
