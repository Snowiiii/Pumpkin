use async_trait::async_trait;
use pumpkin_core::math::vector2::Vector2;
use pumpkin_protocol::client::play::{
    CommandSuggestion, ProtoCmdArgParser, ProtoCmdArgSuggestionType,
};

use crate::command::dispatcher::InvalidTreeError;
use crate::command::tree::RawArgs;
use crate::command::CommandSender;
use crate::server::Server;

use super::super::args::ArgumentConsumer;
use super::{Arg, DefaultNameArgConsumer, FindArg, GetClientSideArgParser};

/// x and z coordinates only
///
/// todo: implememnt ~ ^ notations
pub(crate) struct Position2DArgumentConsumer;

impl GetClientSideArgParser for Position2DArgumentConsumer {
    fn get_client_side_parser(&self) -> ProtoCmdArgParser {
        ProtoCmdArgParser::Vec2
    }

    fn get_client_side_suggestion_type_override(&self) -> Option<ProtoCmdArgSuggestionType> {
        Some(ProtoCmdArgSuggestionType::AskServer)
    }
}

#[async_trait]
impl ArgumentConsumer for Position2DArgumentConsumer {
    async fn consume<'a>(
        &self,
        _src: &CommandSender<'a>,
        _server: &'a Server,
        args: &mut RawArgs<'a>,
    ) -> Option<Arg<'a>> {
        let x_str = args.pop()?;
        let z_str = args.pop()?;

        let mut x = x_str.parse::<f64>().ok()?;
        let mut z = z_str.parse::<f64>().ok()?;

        // set position to block center if no decimal place is given
        if !x_str.contains('.') {
            x += 0.5;
        }
        if !z_str.contains('.') {
            z += 0.5;
        }

        Some(Arg::Pos2D(Vector2::new(x, z)))
    }

    async fn suggest<'a>(
        &self,
        _sender: &CommandSender<'a>,
        _server: &'a Server,
        _input: &'a str,
    ) -> Result<Option<Vec<CommandSuggestion<'a>>>, InvalidTreeError> {
        Ok(None) // todo
    }
}

impl DefaultNameArgConsumer for Position2DArgumentConsumer {
    fn default_name(&self) -> &'static str {
        "pos2d"
    }

    fn get_argument_consumer(&self) -> &dyn ArgumentConsumer {
        &Position2DArgumentConsumer
    }
}

impl<'a> FindArg<'a> for Position2DArgumentConsumer {
    type Data = Vector2<f64>;

    fn find_arg(
        args: &'a super::ConsumedArgs,
        name: &'a str,
    ) -> Result<Self::Data, InvalidTreeError> {
        match args.get(name) {
            Some(Arg::Pos2D(data)) => Ok(*data),
            _ => Err(InvalidTreeError::InvalidConsumptionError(Some(
                name.to_string(),
            ))),
        }
    }
}
