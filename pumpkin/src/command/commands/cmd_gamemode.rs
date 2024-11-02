use std::str::FromStr;

use async_trait::async_trait;
use num_traits::FromPrimitive;
use pumpkin_core::GameMode;

use crate::TextComponent;

use crate::command::arg_player::{parse_arg_players, PlayersArgumentConsumer};

use crate::command::dispatcher::InvalidTreeError;
use crate::command::dispatcher::InvalidTreeError::{
    InvalidConsumptionError, InvalidRequirementError,
};
use crate::command::tree::{ArgumentConsumer, CommandTree, ConsumedArgs, RawArgs};
use crate::command::tree_builder::{argument, require};
use crate::command::CommandSender::Player;
use crate::command::{CommandExecutor, CommandSender};
use crate::server::Server;

const NAMES: [&str; 1] = ["gamemode"];

const DESCRIPTION: &str = "Change a player's gamemode.";

const ARG_GAMEMODE: &str = "gamemode";
const ARG_TARGET: &str = "target";

struct GamemodeArgumentConsumer {}

#[async_trait]
impl ArgumentConsumer for GamemodeArgumentConsumer {
    async fn consume<'a>(
        &self,
        _sender: &CommandSender<'a>,
        _server: &Server,
        args: &mut RawArgs<'a>,
    ) -> Result<String, Option<String>> {
        if let Some(arg) = args.pop() {
            if let Ok(id) = arg.parse::<u8>() {
                match GameMode::from_u8(id) {
                    None | Some(GameMode::Undefined) => {}
                    Some(_) => return Ok(arg.into()),
                };
            };

            match GameMode::from_str(arg) {
                Err(_) | Ok(GameMode::Undefined) => {
                    return Err(Some(format!("Gamemode not found: {arg}")))
                }
                Ok(_) => return Ok(arg.into()),
            }
        }
        Err(None)
    }
}

pub fn parse_arg_gamemode(consumed_args: &ConsumedArgs) -> Result<GameMode, InvalidTreeError> {
    let s = consumed_args
        .get(ARG_GAMEMODE)
        .ok_or(InvalidConsumptionError(None))?;

    if let Ok(id) = s.parse::<u8>() {
        match GameMode::from_u8(id) {
            None | Some(GameMode::Undefined) => {}
            Some(gamemode) => return Ok(gamemode),
        };
    };

    match GameMode::from_str(s) {
        Err(_) | Ok(GameMode::Undefined) => Err(InvalidConsumptionError(Some(s.into()))),
        Ok(gamemode) => Ok(gamemode),
    }
}

struct GamemodeTargetSelf {}

#[async_trait]
impl CommandExecutor for GamemodeTargetSelf {
    async fn execute<'a>(
        &self,
        sender: &mut CommandSender<'a>,
        _server: &Server,
        args: &ConsumedArgs<'a>,
    ) -> Result<(), InvalidTreeError> {
        let gamemode = parse_arg_gamemode(args)?;

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

struct GamemodeTargetPlayer {}

#[async_trait]
impl CommandExecutor for GamemodeTargetPlayer {
    async fn execute<'a>(
        &self,
        sender: &mut CommandSender<'a>,
        server: &Server,
        args: &ConsumedArgs<'a>,
    ) -> Result<(), InvalidTreeError> {
        let gamemode = parse_arg_gamemode(args)?;
        let targets = parse_arg_players(sender, server, ARG_TARGET, args).await?;

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
            argument(ARG_GAMEMODE, &GamemodeArgumentConsumer {})
                .with_child(require(&|sender| sender.is_player()).execute(&GamemodeTargetSelf {}))
                .with_child(
                    argument(ARG_TARGET, &PlayersArgumentConsumer {})
                        .execute(&GamemodeTargetPlayer {}),
                ),
        ),
    )
}
