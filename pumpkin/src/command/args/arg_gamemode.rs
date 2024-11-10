use std::{borrow::Cow, str::FromStr};

use async_trait::async_trait;
use num_traits::FromPrimitive;
use pumpkin_core::{text::TextComponent, GameMode};
use pumpkin_protocol::client::play::{
    CommandSuggestion, ProtoCmdArgParser, ProtoCmdArgSuggestionType, StringProtoArgBehavior
};

use crate::{
    command::{dispatcher::InvalidTreeError, tree::RawArgs, CommandSender},
    server::Server,
};

use super::{Arg, ArgumentConsumer, DefaultNameArgConsumer, FindArg, GetClientSideArgParser, SplitSingleWhitespaceIncludingEmptyParts};

pub(crate) struct GamemodeArgumentConsumer;

impl GetClientSideArgParser for GamemodeArgumentConsumer {
    fn get_client_side_parser(&self) -> ProtoCmdArgParser {
        ProtoCmdArgParser::String(StringProtoArgBehavior::SingleWord)
    }

    fn get_client_side_suggestion_type_override(&self) -> Option<ProtoCmdArgSuggestionType> {
        Some(ProtoCmdArgSuggestionType::AskServer)
    }
}

#[async_trait]
impl ArgumentConsumer for GamemodeArgumentConsumer {
    async fn consume<'a>(
        &self,
        _sender: &CommandSender<'a>,
        _server: &'a Server,
        args: &mut RawArgs<'a>,
    ) -> Option<Arg<'a>> {
        let s = args.pop()?;

        if let Ok(id) = s.parse::<u8>() {
            match GameMode::from_u8(id) {
                None | Some(GameMode::Undefined) => {}
                Some(gamemode) => return Some(Arg::GameMode(gamemode)),
            };
        };

        match GameMode::from_str(s) {
            Err(_) | Ok(GameMode::Undefined) => None,
            Ok(gamemode) => Some(Arg::GameMode(gamemode)),
        }
    }

    async fn suggest<'a>(
        &self,
        _sender: &CommandSender<'a>,
        _server: &'a Server,
        input: &'a str,
    ) -> Result<Option<Vec<CommandSuggestion<'a>>>, InvalidTreeError> {

        let Some(input) = input.split_single_whitespace_including_empty_parts().last() else {
            return Ok(None);
        };

        let modes = ["0", "1", "2", "3", "survival", "creative", "adventure", "spectator"];
        let suggestions = modes.iter()
            .filter(|suggestion| suggestion.starts_with(input))
            .map(|suggestion| CommandSuggestion::new(Cow::Borrowed(suggestion as &str), None))
            .collect();
        Ok(Some(suggestions))
    }
}

impl DefaultNameArgConsumer for GamemodeArgumentConsumer {
    fn default_name(&self) -> &'static str {
        "gamemode"
    }

    fn get_argument_consumer(&self) -> &dyn ArgumentConsumer {
        &GamemodeArgumentConsumer
    }
}

impl<'a> FindArg<'a> for GamemodeArgumentConsumer {
    type Data = GameMode;

    fn find_arg(
        args: &'a super::ConsumedArgs,
        name: &'a str,
    ) -> Result<Self::Data, InvalidTreeError> {
        match args.get(name) {
            Some(Arg::GameMode(data)) => Ok(*data),
            _ => Err(InvalidTreeError::InvalidConsumptionError(Some(
                name.to_string(),
            ))),
        }
    }
}
