use std::borrow::Cow;
use std::sync::Arc;

use async_trait::async_trait;
use pumpkin_protocol::client::play::{CommandSuggestion, ProtoCmdArgParser, ProtoCmdArgSuggestionType, StringProtoArgBehavior};

use crate::command::dispatcher::InvalidTreeError;
use crate::command::tree::RawArgs;
use crate::command::CommandSender;
use crate::entity::player::Player;
use crate::server::Server;

use super::super::args::ArgumentConsumer;
use super::{Arg, DefaultNameArgConsumer, FindArg, GetClientSideArgParser, SplitSingleWhitespaceIncludingEmptyParts};

/// todo: implement for entitites that aren't players
///
/// For selecting a single entity, eg. using @s, a player name or entity uuid.
///
/// Use [`super::arg_entities::EntitiesArgumentConsumer`] when there may be multiple targets.
pub(crate) struct EntityArgumentConsumer;

impl GetClientSideArgParser for EntityArgumentConsumer {
    fn get_client_side_parser(&self) -> ProtoCmdArgParser {
        ProtoCmdArgParser::String(StringProtoArgBehavior::SingleWord)
    
        // todo: investigate why this does not accept target selectors 
        //ProtoCmdArgParser::Entity {
        //    flags: ProtoCmdArgParser::ENTITY_FLAG_ONLY_SINGLE,
        //}
    }

    fn get_client_side_suggestion_type_override(&self) -> Option<ProtoCmdArgSuggestionType> {
        Some(ProtoCmdArgSuggestionType::AskServer)
    }
}

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

    async fn suggest<'a>(
        &self,
        _sender: &CommandSender<'a>,
        server: &'a Server,
        input: &'a str,
    ) -> Result<Option<Vec<CommandSuggestion<'a>>>, InvalidTreeError> {

        dbg!(input);

        let Some(input) = input.split_single_whitespace_including_empty_parts().last() else {
            return Ok(None);
        };

        let target_selectors = ["@s", "@r", "@p", "@n"].iter().map(|s| Cow::Borrowed(s as &str));
        let players = server.get_all_players().await;
        let player_names = players.iter().map(|p| Cow::Owned(p.gameprofile.name.to_string()));

        let suggestions = target_selectors.chain(player_names)
            .filter(|suggestion| suggestion.starts_with(input))
            .map(|suggestion| CommandSuggestion::new(suggestion, None))
            .collect();
        Ok(Some(suggestions))
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
