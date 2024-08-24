use crate::commands::dispatcher::InvalidTreeError;
use crate::commands::dispatcher::InvalidTreeError::InvalidConsumptionError;
use crate::commands::tree::{ConsumedArgs, RawArgs};
use crate::commands::CommandSender;
use crate::commands::CommandSender::Player;

/// todo: implement (so far only own name + @s/@p is implemented)
pub fn consume_arg_player(src: &CommandSender, args: &mut RawArgs) -> Option<String> {
    let s = args.pop()?;

    match s {
        "@s" if src.is_player() => Some(s.into()),
        "@p" if src.is_player() => Some(s.into()),
        "@r" => None,        // todo: implement random player target selector
        "@a" | "@e" => None, // todo: implement all players target selector
        _ => {
            // todo: implement any other player than sender
            if let Player(player) = src {
                if let Some(profile) = &player.client.gameprofile {
                    if profile.name == s {
                        return Some(s.into());
                    };
                };
            };
            None
        }
    }
}

/// todo: implement (so far only own name + @s/@p is implemented)
pub fn parse_arg_player<'a>(
    src: &'a mut CommandSender,
    arg_name: &str,
    consumed_args: &ConsumedArgs,
) -> Result<&'a mut crate::entity::player::Player, InvalidTreeError> {
    let s = consumed_args
        .get(arg_name)
        .ok_or(InvalidConsumptionError(None))?
        .as_str();

    match s {
        "@s" if src.is_player() => Ok(src.as_mut_player().unwrap()),
        "@p" if src.is_player() => Ok(src.as_mut_player().unwrap()),
        "@r" => Err(InvalidConsumptionError(Some(s.into()))), // todo: implement random player target selector
        "@a" | "@e" => Err(InvalidConsumptionError(Some(s.into()))), // todo: implement all players target selector
        _ => {
            // todo: implement any other player than sender
            if let Player(player) = src {
                if let Some(profile) = &player.client.gameprofile {
                    if profile.name == s {
                        return Ok(player);
                    };
                };
            };
            Err(InvalidConsumptionError(Some(s.into())))
        }
    }
}
