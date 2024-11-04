use std::sync::Arc;

use async_trait::async_trait;

use crate::command::dispatcher::InvalidTreeError;
use crate::command::tree::RawArgs;
use crate::command::CommandSender;
use crate::entity::player::Player;
use crate::server::Server;

use super::super::args::ArgumentConsumer;
use super::{Arg, DefaultNameArgConsumer, FindArg};

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
        server: &'a Server,
        args: &mut RawArgs<'a>,
    ) -> Option<Arg<'a>> {
        let s = args.pop()?;

        let entity = match s {
            // @s is always valid when sender is a player
            "@s" => match src {
                CommandSender::Player(p) => Some(p.clone()),
                _ => None,
            },
            // @n/@p are always valid when sender is a player
            #[allow(clippy::match_same_arms)] // todo: implement for non-players
            "@n" | "@p" => match src {
                CommandSender::Player(p) => Some(p.clone()),
                // todo: implement for non-players: how should this behave when sender is console/rcon?
                _ => None,
            },
            // @r is valid when there is at least one player
            "@r" => server.get_random_player().await,
            // @a/@e/@r are not valid because we're looking for a single entity
            "@a" | "@e" => None,
            // player name is only valid if player is online
            name => server.get_player_by_name(name).await,
        };

        entity.map(Arg::Entity)
    }
}

impl DefaultNameArgConsumer for EntityArgumentConsumer {
    fn default_name(&self) -> &'static str {
        "target"
    }

    fn get_argument_consumer(&self) -> &dyn ArgumentConsumer {
        &EntityArgumentConsumer
    }
}

impl<'a> FindArg<'a> for EntityArgumentConsumer {
    type Data = Arc<Player>;

    fn find_arg(
        args: &'a super::ConsumedArgs,
        name: &'a str,
    ) -> Result<Self::Data, InvalidTreeError> {
        match args.get(name) {
            Some(Arg::Entity(data)) => Ok(data.clone()),
            _ => Err(InvalidTreeError::InvalidConsumptionError(Some(
                name.to_string(),
            ))),
        }
    }
}
