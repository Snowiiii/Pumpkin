use std::sync::Arc;

use async_trait::async_trait;
use pumpkin_protocol::client::play::{
    CommandSuggestion, ProtoCmdArgParser, ProtoCmdArgSuggestionType,
};

use crate::command::dispatcher::InvalidTreeError;
use crate::command::tree::RawArgs;
use crate::command::CommandSender;
use crate::entity::player::Player;
use crate::server::Server;

use super::super::args::ArgumentConsumer;
use super::arg_players::PlayersArgumentConsumer;
use super::{Arg, DefaultNameArgConsumer, FindArg, GetClientSideArgParser};

/// todo: implement (currently just calls [`super::arg_player::PlayerArgumentConsumer`])
///
/// For selecting zero, one or multiple entities, eg. using @s, a player name, @a or @e
pub(crate) struct EntitiesArgumentConsumer;

impl GetClientSideArgParser for EntitiesArgumentConsumer {
    fn get_client_side_parser(&self) -> ProtoCmdArgParser {
        //ProtoCmdArgParser::String(StringProtoArgBehavior::SingleWord)

        // todo: investigate why this does not accept target selectors
        ProtoCmdArgParser::Entity {
            flags: ProtoCmdArgParser::ENTITY_FLAG_ONLY_SINGLE,
        }
    }

    fn get_client_side_suggestion_type_override(&self) -> Option<ProtoCmdArgSuggestionType> {
        None //Some(ProtoCmdArgSuggestionType::AskServer)
    }
}

#[async_trait]
impl ArgumentConsumer for EntitiesArgumentConsumer {
    async fn consume<'a>(
        &self,
        src: &CommandSender<'a>,
        server: &'a Server,
        args: &mut RawArgs<'a>,
    ) -> Option<Arg<'a>> {
        // todo
        match PlayersArgumentConsumer.consume(src, server, args).await {
            Some(Arg::Players(p)) => Some(Arg::Entities(p)),
            _ => None,
        }
    }

    async fn suggest<'a>(
        &self,
        sender: &CommandSender<'a>,
        server: &'a Server,
        input: &'a str,
    ) -> Result<Option<Vec<CommandSuggestion<'a>>>, InvalidTreeError> {
        // todo
        PlayersArgumentConsumer.suggest(sender, server, input).await
    }
}

impl DefaultNameArgConsumer for EntitiesArgumentConsumer {
    fn default_name(&self) -> &'static str {
        "targets"
    }

    fn get_argument_consumer(&self) -> &dyn ArgumentConsumer {
        &EntitiesArgumentConsumer
    }
}

impl<'a> FindArg<'a> for EntitiesArgumentConsumer {
    type Data = &'a [Arc<Player>];

    fn find_arg(
        args: &'a super::ConsumedArgs,
        name: &'a str,
    ) -> Result<Self::Data, InvalidTreeError> {
        match args.get(name) {
            Some(Arg::Entities(data)) => Ok(data),
            _ => Err(InvalidTreeError::InvalidConsumptionError(Some(
                name.to_string(),
            ))),
        }
    }
}
