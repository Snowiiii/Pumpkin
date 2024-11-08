use async_trait::async_trait;

use crate::command::args::arg_gamemode::GamemodeArgumentConsumer;
use crate::command::args::GetCloned;

use crate::TextComponent;

use crate::command::args::arg_players::PlayersArgumentConsumer;

use crate::command::args::{Arg, ConsumedArgs};
use crate::command::dispatcher::InvalidTreeError;
use crate::command::dispatcher::InvalidTreeError::{
    InvalidConsumptionError, InvalidRequirementError,
};
use crate::command::tree::CommandTree;
use crate::command::tree_builder::{argument, require};
use crate::command::CommandSender::Player;
use crate::command::{CommandExecutor, CommandSender};
use crate::server::Server;

const NAMES: [&str; 1] = ["gamemode"];

const DESCRIPTION: &str = "Change a player's gamemode.";

const ARG_GAMEMODE: &str = "gamemode";
const ARG_TARGET: &str = "target";

struct GamemodeTargetSelf;

#[async_trait]
impl CommandExecutor for GamemodeTargetSelf {
    async fn execute<'a>(
        &self,
        sender: &mut CommandSender<'a>,
        _server: &Server,
        args: &ConsumedArgs<'a>,
    ) -> Result<(), InvalidTreeError> {
        let Some(Arg::GameMode(gamemode)) = args.get_cloned(&ARG_GAMEMODE) else {
            return Err(InvalidConsumptionError(Some(ARG_GAMEMODE.into())));
        };

        if let Player(target) = sender {
            if target.gamemode.load() == gamemode {
                target
                    .send_system_message(&TextComponent::text(&format!(
                        "You already in {gamemode:?} gamemode"
                    )))
                    .await;
            } else {
                target.set_gamemode(gamemode).await;
                target
                    .send_system_message(&TextComponent::text(&format!(
                        "Game mode was set to {gamemode:?}"
                    )))
                    .await;
            }
            Ok(())
        } else {
            Err(InvalidRequirementError)
        }
    }
}

struct GamemodeTargetPlayer;

#[async_trait]
impl CommandExecutor for GamemodeTargetPlayer {
    async fn execute<'a>(
        &self,
        sender: &mut CommandSender<'a>,
        _server: &Server,
        args: &ConsumedArgs<'a>,
    ) -> Result<(), InvalidTreeError> {
        let Some(Arg::GameMode(gamemode)) = args.get_cloned(&ARG_GAMEMODE) else {
            return Err(InvalidConsumptionError(Some(ARG_GAMEMODE.into())));
        };
        let Some(Arg::Players(targets)) = args.get(ARG_TARGET) else {
            return Err(InvalidConsumptionError(Some(ARG_TARGET.into())));
        };

        let target_count = targets.len();

        for target in targets {
            if target.gamemode.load() == gamemode {
                if target_count == 1 {
                    sender
                        .send_message(TextComponent::text(&format!(
                            "{} is already in {:?} gamemode",
                            target.gameprofile.name, gamemode
                        )))
                        .await;
                }
            } else {
                target.set_gamemode(gamemode).await;
                if target_count == 1 {
                    sender
                        .send_message(TextComponent::text(&format!(
                            "{}'s Game mode was set to {:?}",
                            target.gameprofile.name, gamemode
                        )))
                        .await;
                }
            }
        }

        Ok(())
    }
}

#[allow(clippy::redundant_closure_for_method_calls)]
pub fn init_command_tree<'a>() -> CommandTree<'a> {
    CommandTree::new(NAMES, DESCRIPTION).with_child(
        require(&|sender| sender.permission_lvl() >= 2).with_child(
            argument(ARG_GAMEMODE, &GamemodeArgumentConsumer)
                .with_child(require(&|sender| sender.is_player()).execute(&GamemodeTargetSelf))
                .with_child(
                    argument(ARG_TARGET, &PlayersArgumentConsumer).execute(&GamemodeTargetPlayer),
                ),
        ),
    )
}
