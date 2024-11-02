use std::sync::Arc;

use async_trait::async_trait;

use crate::command::dispatcher::InvalidTreeError;
use crate::command::dispatcher::InvalidTreeError::InvalidConsumptionError;
use crate::command::tree::{ConsumedArgs, RawArgs};
use crate::command::CommandSender;
use crate::server::Server;

use super::tree::ArgumentConsumer;

/// Select zero, one or multiple players
pub(crate) struct PlayersArgumentConsumer;

#[async_trait]
impl ArgumentConsumer for PlayersArgumentConsumer {
    async fn consume<'a>(
        &self,
        src: &CommandSender<'a>,
        server: &Server,
        args: &mut RawArgs<'a>,
    ) -> Result<String, Option<String>> {
        let Some(s) = args.pop() else {
            return Err(None);
        };

        match s {
            // @s is always valid when sender is a player
            "@s" => {
                return match src {
                    CommandSender::Player(..) => Ok(s.into()),
                    _ => Err(Some("You are not a Player".into())),
                }
            }
            // @n/@p are always valid when sender is a player
            "@n" | "@p" => {
                return match src {
                    CommandSender::Player(..) => Ok(s.into()),
                    // todo: implement for non-players: how should this behave when sender is console/rcon?
                    _ => Err(Some("You are not a Player".into())),
                };
            }
            // @a/@e/@r are always valid
            "@a" | "@e" | "@r" => Ok(s.into()),
            // player name is only valid if player is online
            name => {
                if server.get_player_by_name(name).await.is_some() {
                    Ok(s.into())
                } else {
                    Err(Some(format!("Player not found: {name}")))
                }
            }
        }
    }
}

pub async fn parse_arg_players<'a>(
    src: &mut CommandSender<'a>,
    server: &Server,
    arg_name: &str,
    consumed_args: &ConsumedArgs<'a>,
) -> Result<Vec<Arc<crate::entity::player::Player>>, InvalidTreeError> {
    let s = consumed_args
        .get(arg_name)
        .ok_or(InvalidConsumptionError(None))?
        .as_str();

    match s {
        "@s" => match src {
            CommandSender::Player(p) => Ok(vec![p.clone()]),
            _ => Err(InvalidConsumptionError(Some(s.into()))),
        },
        #[allow(clippy::match_same_arms)] // todo: implement for non-players and remove this line
        "@n" | "@p" => match src {
            CommandSender::Player(p) => Ok(vec![p.clone()]),
            // todo: implement for non-players: how should this behave when sender is console/rcon?
            _ => Err(InvalidConsumptionError(Some(s.into()))),
        },
        "@r" => {
            if let Some(p) = server.get_random_player().await {
                Ok(vec![p.clone()])
            } else {
                Ok(vec![])
            }
        }
        "@a" | "@e" => Ok(server.get_all_players().await),
        name => {
            if let Some(p) = server.get_player_by_name(name).await {
                Ok(vec![p])
            } else {
                Err(InvalidConsumptionError(Some(s.into())))
            }
        }
    }
}
