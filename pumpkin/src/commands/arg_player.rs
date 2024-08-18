use crate::client::Client;
use crate::commands::CommandSender;
use crate::commands::CommandSender::Player;
use crate::commands::dispatcher::InvalidTreeError;
use crate::commands::dispatcher::InvalidTreeError::InvalidConsumptionError;
use crate::commands::tree::{ConsumedArgs, RawArgs};

/// todo: implement (so far only own name + @s is implemented)
pub fn consume_arg_player(src: &CommandSender, args: &mut RawArgs) -> Option<String> {
    let s = args.pop()?;

    if let Player(client) = src {
        if s == "@s" {
            return Some(s.into())
        }
        if let Some(profile) = &client.gameprofile {
            if profile.name == s {
                return Some(s.into())
            };
        };
    };

    None
}

/// todo: implement (so far only own name + @s is implemented)
pub fn parse_arg_player<'a>(src: &'a mut CommandSender, arg_name: &str, consumed_args: &ConsumedArgs) -> Result<&'a mut Client, InvalidTreeError> {
    let s = consumed_args.get(arg_name)
        .ok_or(InvalidConsumptionError(None))?;

    if let Player(client) = src {
        if s == "@s" {
            return Ok(client)
        }
        if let Some(profile) = &client.gameprofile {
            if profile.name == s.as_ref() {
                return Ok(client)
            };
        };
    };

    Err(InvalidConsumptionError(Some(s.into())))
}