use std::str::FromStr;

use num_traits::FromPrimitive;
use pumpkin_core::text::TextComponent;
use pumpkin_core::GameMode;

use crate::commands::arg_player::{consume_arg_player, parse_arg_player};

use crate::commands::dispatcher::InvalidTreeError;
use crate::commands::dispatcher::InvalidTreeError::{
    InvalidConsumptionError, InvalidRequirementError,
};
use crate::commands::tree::{CommandTree, ConsumedArgs, RawArgs};
use crate::commands::tree_builder::{argument, require};
use crate::commands::CommandSender;
use crate::commands::CommandSender::Player;

const NAMES: [&str; 1] = ["gamemode"];

const DESCRIPTION: &str = "Change a player's gamemode.";

const ARG_GAMEMODE: &str = "gamemode";
const ARG_TARGET: &str = "target";

pub fn consume_arg_gamemode(_src: &CommandSender, args: &mut RawArgs) -> Option<String> {
    let s = args.pop()?;

    if let Ok(id) = s.parse::<u8>() {
        match GameMode::from_u8(id) {
            None | Some(GameMode::Undefined) => {}
            Some(_) => return Some(s.into()),
        };
    };

    match GameMode::from_str(s) {
        Err(_) | Ok(GameMode::Undefined) => None,
        Ok(_) => Some(s.into()),
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

pub(crate) fn init_command_tree<'a>() -> CommandTree<'a> {
    CommandTree::new(NAMES, DESCRIPTION).with_child(
        require(&|sender| sender.permission_lvl() >= 2).with_child(
            argument(ARG_GAMEMODE, consume_arg_gamemode)
                .with_child(
                    require(&|sender| sender.is_player()).execute(&|sender, _, args| {
                        let gamemode = parse_arg_gamemode(args)?;

                        return if let Player(target) = sender {
                            if target.gamemode.load() == gamemode {
                                target.send_system_message(TextComponent::text(&format!(
                                    "You already in {:?} gamemode",
                                    gamemode
                                )));
                            } else {
                                // TODO
                                target.set_gamemode(gamemode);
                                target.send_system_message(TextComponent::text(&format!(
                                    "Game mode was set to {:?}",
                                    gamemode
                                )));
                            }
                            Ok(())
                        } else {
                            Err(InvalidRequirementError)
                        };
                    }),
                )
                .with_child(argument(ARG_TARGET, consume_arg_player).execute(
                    &|sender, server, args| {
                        let gamemode = parse_arg_gamemode(args)?;
                        let target = parse_arg_player(sender, server, ARG_TARGET, args)?;

                        if target.gamemode.load() == gamemode {
                            target.send_system_message(TextComponent::text(&format!(
                                "You already in {:?} gamemode",
                                gamemode
                            )));
                        } else {
                            // TODO
                            target.set_gamemode(gamemode);
                            target.send_system_message(TextComponent::text(&format!(
                                "Game mode was set to {:?}",
                                gamemode
                            )));
                        }

                        Ok(())
                    },
                )),
        ),
    )
}
