use crate::{
    command::{
        args::{arg_players::PlayersArgumentConsumer, Arg, ConsumedArgs},
        tree::CommandTree,
        tree_builder::{argument, require},
        CommandError, CommandExecutor, CommandSender,
    },
    entity::player::PermissionLvl,
    server::json_config::{SaveJSONConfiguration, OPERATOR_CONFIG},
};
use async_trait::async_trait;
use pumpkin_config::{op::Op, BASIC_CONFIG};
use pumpkin_core::text::TextComponent;
use CommandError::InvalidConsumption;

const NAMES: [&str; 1] = ["op"];
const DESCRIPTION: &str = "Specifies one or more game profiles (player profiles). Must be a player name (should be a real one if the server is in online mode) or a player-type target selector";
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

        // log each player to the console.
        for player in targets {
            let op_entry = Op {
                uuid: player.gameprofile.id,
                name: player.gameprofile.name.clone(),
                level: BASIC_CONFIG.op_permission_level,
                bypasses_player_limit: false,
            };
            if let Some(op) = config
                .ops
                .iter_mut()
                .find(|o| o.uuid == player.gameprofile.id)
            {
                op.level = BASIC_CONFIG.op_permission_level;
            } else {
                config.ops.push(op_entry);
            }
            config.save();

            player
                .set_permission_lvl(
                    BASIC_CONFIG.op_permission_level.into(),
                    &server.command_dispatcher,
                )
                .await;

            let player_name = player.gameprofile.name.clone();
            let message = format!("Made {player_name} a server operator.");
            let msg = TextComponent::text(&message);
            sender.send_message(msg).await;
        }

        Ok(())
    }
}

pub fn init_command_tree<'a>() -> CommandTree<'a> {
    CommandTree::new(NAMES, DESCRIPTION).with_child(
        require(&|sender| sender.has_permission_lvl(PermissionLvl::Four))
            .with_child(argument(ARG_TARGET, &PlayersArgumentConsumer).execute(&OpExecutor)),
    )
}
