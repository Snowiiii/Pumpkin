use std::sync::Arc;

use async_trait::async_trait;

use crate::commands::dispatcher::InvalidTreeError;
use crate::commands::dispatcher::InvalidTreeError::InvalidConsumptionError;
use crate::commands::tree::{ConsumedArgs, RawArgs};
use crate::commands::CommandSender;
use crate::server::Server;

use super::tree::ArgumentConsumer;

/// TODO: Seperate base functionality of these two methods into single method

/// todo: implement (so far only own name + @s/@p is implemented)
pub(crate) struct PlayerArgumentConsumer {}

#[async_trait]
impl ArgumentConsumer for PlayerArgumentConsumer {
    async fn consume<'a>(
        &self,
        src: &CommandSender<'a>,
        server: &Server,
        args: &mut RawArgs<'a>,
    ) -> Result<String, Option<String>> {
        if let Some(arg) = args.pop() {
            match arg {
                "@s" => {
                    if src.is_player() {
                        return Ok(arg.into());
                    }
                    return Err(Some("You are not a Player".into()));
                }
                "@p" if src.is_player() => return Ok(arg.into()),
                "@r" => todo!(), // todo: implement random player target selector
                "@a" | "@e" => todo!(), // todo: implement all players target selector
                name => {
                    // todo: implement any other player than sender
                    for world in &server.worlds {
                        if world.get_player_by_name(name).await.is_some() {
                            return Ok(name.into());
                        }
                    }
                    return Err(Some(format!("Player not found: {arg}")));
                }
            }
        }
        Err(None)
    }
}

/// todo: implement (so far only own name + @s/@p is implemented)
pub async fn parse_arg_player<'a>(
    src: &mut CommandSender<'a>,
    server: &Server,
    arg_name: &str,
    consumed_args: &ConsumedArgs<'a>,
) -> Result<Arc<crate::entity::player::Player>, InvalidTreeError> {
    let s = consumed_args
        .get(arg_name)
        .ok_or(InvalidConsumptionError(None))?
        .as_str();

    match s {
        "@s" if src.is_player() => Ok(src.as_player().unwrap()),
        "@p" => todo!(),
        "@r" => todo!(),        // todo: implement random player target selector
        "@a" | "@e" => todo!(), // todo: implement all players target selector
        name => {
            for world in &server.worlds {
                if let Some(player) = world.get_player_by_name(name).await {
                    return Ok(player);
                }
            }
            Err(InvalidConsumptionError(Some(s.into())))
        }
    }
}
