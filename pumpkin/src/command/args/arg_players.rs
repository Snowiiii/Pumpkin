use std::sync::Arc;

use async_trait::async_trait;
use pumpkin_protocol::client::play::{ProtoCmdArgParser, ProtoCmdArgSuggestionType};

use crate::command::dispatcher::InvalidTreeError;
use crate::command::tree::RawArgs;
use crate::command::CommandSender;
use crate::entity::player::Player;
use crate::server::Server;

use super::super::args::ArgumentConsumer;
use super::{Arg, DefaultNameArgConsumer, FindArg, GetClientSideArgParser};

/// Select zero, one or multiple players
pub(crate) struct PlayersArgumentConsumer;

impl GetClientSideArgParser for PlayersArgumentConsumer {
    fn get_client_side_parser(&self) -> ProtoCmdArgParser {
        ProtoCmdArgParser::Entity {
            flags: ProtoCmdArgParser::ENTITY_FLAG_PLAYERS_ONLY,
        }
    }

    fn get_client_side_suggestion_type_override(&self) -> Option<ProtoCmdArgSuggestionType> {
        None
    }
}

#[async_trait]
impl ArgumentConsumer for PlayersArgumentConsumer {
    async fn consume<'a>(
        &self,
        src: &CommandSender<'a>,
        server: &'a Server,
        args: &mut RawArgs<'a>,
    ) -> Option<Arg<'a>> {
        let players = match args.pop()? {
            "@s" => match src {
                CommandSender::Player(p) => Some(vec![p.clone()]),
                _ => None,
            },
            #[allow(clippy::match_same_arms)]
            // todo: implement for non-players and remove this line
            "@n" | "@p" => match src {
                CommandSender::Player(p) => Some(vec![p.clone()]),
                // todo: implement for non-players: how should this behave when sender is console/rcon?
                _ => None,
            },
            "@r" => {
                if let Some(p) = server.get_random_player().await {
                    Some(vec![p.clone()])
                } else {
                    Some(vec![])
                }
            }
            "@a" | "@e" => Some(server.get_all_players().await),
            name => server.get_player_by_name(name).await.map(|p| vec![p]),
        };

        players.map(Arg::Players)
    }
}

impl DefaultNameArgConsumer for PlayersArgumentConsumer {
    fn default_name(&self) -> &'static str {
        "player"
    }

    fn get_argument_consumer(&self) -> &dyn ArgumentConsumer {
        &PlayersArgumentConsumer
    }
}

impl<'a> FindArg<'a> for PlayersArgumentConsumer {
    type Data = &'a [Arc<Player>];

    fn find_arg(
        args: &'a super::ConsumedArgs,
        name: &'a str,
    ) -> Result<Self::Data, InvalidTreeError> {
        match args.get(name) {
            Some(Arg::Players(data)) => Ok(data),
            _ => Err(InvalidTreeError::InvalidConsumptionError(Some(
                name.to_string(),
            ))),
        }
    }
}
