use std::str::FromStr;

use async_trait::async_trait;
use num_traits::FromPrimitive;
use pumpkin_core::GameMode;
use pumpkin_protocol::client::play::{
    CommandSuggestion, ProtoCmdArgParser, ProtoCmdArgSuggestionType,
};

use crate::{
    command::{dispatcher::CommandError, tree::RawArgs, CommandSender},
    server::Server,
};

use super::{Arg, ArgumentConsumer, DefaultNameArgConsumer, FindArg, GetClientSideArgParser};

pub(crate) struct GamemodeArgumentConsumer;

impl GetClientSideArgParser for GamemodeArgumentConsumer {
    fn get_client_side_parser(&self) -> ProtoCmdArgParser {
        ProtoCmdArgParser::Gamemode
    }

    fn get_client_side_suggestion_type_override(&self) -> Option<ProtoCmdArgSuggestionType> {
        None
    }
}

#[async_trait]
impl ArgumentConsumer for GamemodeArgumentConsumer {
    async fn consume<'a>(
        &'a self,
        _sender: &CommandSender<'a>,
        _server: &'a Server,
        args: &mut RawArgs<'a>,
    ) -> Result<Option<Arg<'a>>, CommandError> {
        let s = args.pop().ok_or(CommandError::InvalidConsumption(None))?;

        if let Ok(id) = s.parse::<u8>() {
            match GameMode::from_u8(id) {
                None | Some(GameMode::Undefined) => {}
                Some(gamemode) => return Ok(Some(Arg::GameMode(gamemode))),
            };
        };

        match GameMode::from_str(s) {
            Err(_) | Ok(GameMode::Undefined) => {
                Err(CommandError::InvalidConsumption(Some(s.to_string())))
            }
            Ok(gamemode) => Ok(Some(Arg::GameMode(gamemode))),
        }
    }

    async fn suggest<'a>(
        &'a self,
        _sender: &CommandSender<'a>,
        _server: &'a Server,
        _input: &'a str,
    ) -> Result<Option<Vec<CommandSuggestion<'a>>>, CommandError> {
        Ok(None)
    }
}

impl DefaultNameArgConsumer for GamemodeArgumentConsumer {
    fn default_name(&self) -> String {
        "gamemode".to_string()
    }
}

impl<'a> FindArg<'a> for GamemodeArgumentConsumer {
    type Data = GameMode;

    fn find_arg(args: &'a super::ConsumedArgs, name: &str) -> Result<Self::Data, CommandError> {
        match args.get(name) {
            Some(Arg::GameMode(data)) => Ok(*data),
            _ => Err(CommandError::InvalidConsumption(Some(name.to_string()))),
        }
    }
}
