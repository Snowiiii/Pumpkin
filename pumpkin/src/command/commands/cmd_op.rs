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
use pumpkin_config::{op::Op, BASIC_CONFIG};
use pumpkin_core::text::TextComponent;
use CommandError::InvalidConsumption;

const NAMES: [&str; 1] = ["op"];
const DESCRIPTION: &str = "Grants operator status to a player.";
const ARG_TARGET: &str = "player";

struct OpExecutor;

#[async_trait]
impl CommandExecutor for OpExecutor {
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

        let new_level = BASIC_CONFIG
            .op_permission_level
            .min(sender.permission_lvl());

        if let Some(op) = config
            .ops
            .iter_mut()
            .find(|o| o.uuid == player.gameprofile.id)
        {
            op.level = new_level;
        } else {
            let op_entry = Op::new(
                player.gameprofile.id,
                player.gameprofile.name.clone(),
                new_level,
                false,
            );
            config.ops.push(op_entry);
        }

        config.save();

        player
            .set_permission_lvl(new_level, &server.command_dispatcher)
            .await;

        let player_name = &player.gameprofile.name;
        let message = format!("Made {player_name} a server operator.");
        let msg = TextComponent::text(message);
        sender.send_message(msg).await;
        player
            .send_system_message(&TextComponent::text("You are now a server operator."))
            .await;

        Ok(())
    }
}

pub fn init_command_tree() -> CommandTree {
    CommandTree::new(NAMES, DESCRIPTION)
        .with_child(argument(ARG_TARGET, PlayersArgumentConsumer).execute(OpExecutor))
}
