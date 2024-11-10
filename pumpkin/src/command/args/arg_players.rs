use std::borrow::Cow;
use std::sync::Arc;

use async_trait::async_trait;
use pumpkin_protocol::client::play::{
    CommandSuggestion, ProtoCmdArgParser, ProtoCmdArgSuggestionType, StringProtoArgBehavior,
};

use crate::command::dispatcher::InvalidTreeError;
use crate::command::tree::RawArgs;
use crate::command::CommandSender;
use crate::entity::player::Player;
use crate::server::Server;

use super::super::args::ArgumentConsumer;
use super::{
    Arg, DefaultNameArgConsumer, FindArg, GetClientSideArgParser,
    SplitSingleWhitespaceIncludingEmptyParts,
};

/// Select zero, one or multiple players
pub(crate) struct PlayersArgumentConsumer;

impl GetClientSideArgParser for PlayersArgumentConsumer {
    fn get_client_side_parser(&self) -> ProtoCmdArgParser {
        ProtoCmdArgParser::String(StringProtoArgBehavior::SingleWord)

        // todo: investigate why this does not accept target selectors
        //ProtoCmdArgParser::Entity {
        //    flags: ProtoCmdArgParser::ENTITY_FLAG_PLAYERS_ONLY,
        //}
    }

    fn get_client_side_suggestion_type_override(&self) -> Option<ProtoCmdArgSuggestionType> {
        Some(ProtoCmdArgSuggestionType::AskServer)
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

    async fn suggest<'a>(
        &self,
        _sender: &CommandSender<'a>,
        server: &'a Server,
        input: &'a str,
    ) -> Result<Option<Vec<CommandSuggestion<'a>>>, InvalidTreeError> {
        let Some(input) = input.split_single_whitespace_including_empty_parts().last() else {
            return Ok(None);
        };

        let target_selectors = ["@s", "@a", "@e", "@r", "@p", "@n"]
            .iter()
            .map(|s| Cow::Borrowed(s as &str));
        let players = server.get_all_players().await;
        let player_names = players
            .iter()
            .map(|p| Cow::Owned(p.gameprofile.name.to_string()));

        let suggestions = target_selectors
            .chain(player_names)
            .filter(|suggestion| suggestion.starts_with(input))
            .map(|suggestion| CommandSuggestion::new(suggestion, None))
            .collect();

        Ok(Some(suggestions))
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
