use std::sync::Arc;

use async_trait::async_trait;

use crate::command::dispatcher::InvalidTreeError;
use crate::command::dispatcher::InvalidTreeError::InvalidConsumptionError;
use crate::command::tree::{ConsumedArgs, RawArgs};
use crate::command::CommandSender;
use crate::server::Server;

use super::tree::ArgumentConsumer;

/// todo: implement for entitites that aren't players
///
/// For selecting a single entity, eg. using @s, a player name or entity uuid.
///
/// Use [`super::arg_entities::EntitiesArgumentConsumer`] when there may be multiple targets.
pub(crate) struct EntityArgumentConsumer;

#[async_trait]
impl ArgumentConsumer for EntityArgumentConsumer {
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
            // @r is valid when there is at least one player
            "@r" => {
                if server.has_n_players(1).await {
                    Ok(s.into())
                } else {
                    Err(None)
                }
            }
            // @a/@e/@r are not valid because we're looking for a single entity
            "@a" | "@e" => Err(None),
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

/// todo: implement for entitites that aren't players
pub(crate) async fn parse_arg_entity<'a>(
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
        "@s" => match src {
            CommandSender::Player(p) => Ok(p.clone()),
            _ => Err(InvalidConsumptionError(Some(s.into()))),
        },
        "@n" | "@p" => match src {
            CommandSender::Player(p) => Ok(p.clone()),
            // todo: implement for non-players: how should this behave when sender is console/rcon?
            _ => Err(InvalidConsumptionError(Some(s.into()))),
        },
        "@r" => {
            if let Some(p) = server.get_random_player().await {
                Ok(p.clone())
            } else {
                Err(InvalidConsumptionError(Some(s.into())))
            }
        }
        "@a" | "@e" => Err(InvalidConsumptionError(Some(s.into()))),
        name => {
            if let Some(p) = server.get_player_by_name(name).await {
                Ok(p)
            } else {
                Err(InvalidConsumptionError(Some(s.into())))
            }
        }
    }
}
