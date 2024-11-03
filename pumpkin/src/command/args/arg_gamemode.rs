use std::str::FromStr;

use async_trait::async_trait;
use num_traits::FromPrimitive;
use pumpkin_core::GameMode;

use crate::{
    command::{dispatcher::InvalidTreeError, tree::RawArgs, CommandSender},
    server::Server,
};

use super::{Arg, ArgumentConsumer, DefaultNameArgConsumer, FindArg};

pub(crate) struct GamemodeArgumentConsumer;

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

    fn find_arg(args: &'a super::ConsumedArgs, name: &'a str) -> Result<Self::Data, InvalidTreeError> {
        match args.get(name) {
            Some(Arg::GameMode(data)) => Ok(*data),
            _ => Err(InvalidTreeError::InvalidConsumptionError(Some(name.to_string())))
        }
    }
}
