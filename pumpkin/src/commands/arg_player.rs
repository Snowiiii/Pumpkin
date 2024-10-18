use std::sync::Arc;

use crate::commands::dispatcher::InvalidTreeError;
use crate::commands::dispatcher::InvalidTreeError::InvalidConsumptionError;
use crate::commands::tree::{ConsumedArgs, RawArgs};
use crate::commands::CommandSender;
use crate::server::Server;

/// todo: implement (so far only own name + @s/@p is implemented)
pub fn consume_arg_player(
    src: &CommandSender,
    server: &Server,
    args: &mut RawArgs,
) -> Result<String, Option<String>> {
    if let Some(arg) = args.pop() {
        match arg {
            "@s" => {
                if src.is_player() {
                    return Ok(arg.into());
                } else {
                    return Err(Some("You are not a Player".into()));
                }
            }
            "@p" if src.is_player() => return Ok(arg.into()),
            "@r" => todo!(),        // todo: implement random player target selector
            "@a" | "@e" => todo!(), // todo: implement all players target selector
            name => {
                // todo: implement any other player than sender
                for world in &server.worlds {
                    if world.get_player_by_name(name).is_some() {
                        return Ok(name.into());
                    }
                }
                return Err(Some(format!("Player not found: {}", arg)));
            }
        }
    }
    Err(None)
}

/// todo: implement (so far only own name + @s/@p is implemented)
pub fn parse_arg_player(
    src: &mut CommandSender,
    server: &Server,
    arg_name: &str,
    consumed_args: &ConsumedArgs,
) -> Result<Arc<crate::entity::player::Player>, InvalidTreeError> {
    let s = consumed_args
        .get(arg_name)
        .ok_or(InvalidConsumptionError(None))?
        .as_str();

    match s {
        "@s" if src.is_player() => Ok(src.as_mut_player().unwrap()),
        "@p" if src.is_player() => Ok(src.as_mut_player().unwrap()),
        "@r" => Err(InvalidConsumptionError(Some(s.into()))), // todo: implement random player target selector
        "@a" | "@e" => Err(InvalidConsumptionError(Some(s.into()))), // todo: implement all players target selector
        name => {
            for world in &server.worlds {
                if let Some(player) = world.get_player_by_name(name) {
                    return Ok(player);
                }
            }
            Err(InvalidConsumptionError(Some(s.into())))
        }
    }
}
