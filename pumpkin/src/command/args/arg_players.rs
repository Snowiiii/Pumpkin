use std::sync::Arc;

use async_trait::async_trait;
use pumpkin_protocol::client::play::{
    CommandSuggestion, ProtoCmdArgParser, ProtoCmdArgSuggestionType,
};

use crate::command::dispatcher::CommandError;
use crate::command::tree::RawArgs;
use crate::command::CommandSender;
use crate::entity::player::Player;
use crate::server::Server;

use super::super::args::ArgumentConsumer;
use super::{Arg, DefaultNameArgConsumer, FindArg, GetClientSideArgParser};

/// Select zero, one or multiple players
pub struct PlayersArgumentConsumer;

impl GetClientSideArgParser for PlayersArgumentConsumer {
    fn get_client_side_parser(&self) -> ProtoCmdArgParser {
        // todo: investigate why this does not accept target selectors
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
        &'a self,
        src: &CommandSender<'a>,
        server: &'a Server,
        args: &mut RawArgs<'a>,
    ) -> Option<Arg<'a>> {
        let s = args.pop()?;

        let players = match s {
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
                (server.get_random_player().await).map_or_else(|| Some(vec![]), |p| Some(vec![p]))
            }
            "@a" | "@e" => Some(server.get_all_players().await),
            name => server.get_player_by_name(name).await.map(|p| vec![p]),
        };

        players.map(Arg::Players)
    }

    async fn suggest<'a>(
        &'a self,
        _sender: &CommandSender<'a>,
        _server: &'a Server,
        _input: &'a str,
    ) -> Result<Option<Vec<CommandSuggestion>>, CommandError> {
        Ok(None)
    }
}

impl DefaultNameArgConsumer for PlayersArgumentConsumer {
    fn default_name(&self) -> String {
        "player".to_string()
    }
}

impl<'a> FindArg<'a> for PlayersArgumentConsumer {
    type Data = &'a [Arc<Player>];

    fn find_arg(args: &'a super::ConsumedArgs, name: &str) -> Result<Self::Data, CommandError> {
        match args.get(name) {
            Some(Arg::Players(data)) => Ok(data),
            _ => Err(CommandError::InvalidConsumption(Some(name.to_string()))),
        }
    }
}
