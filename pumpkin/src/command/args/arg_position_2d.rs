use async_trait::async_trait;
use pumpkin_core::math::vector2::Vector2;

use crate::command::dispatcher::InvalidTreeError;
use crate::command::tree::RawArgs;
use crate::command::CommandSender;
use crate::server::Server;

use super::super::args::ArgumentConsumer;
use super::{Arg, DefaultNameArgConsumer, FindArg};

/// x and z coordinates only
///
/// todo: implememnt ~ ^ notations
pub(crate) struct Position2DArgumentConsumer;

#[async_trait]
impl ArgumentConsumer for Position2DArgumentConsumer {
    async fn consume<'a>(
        &self,
        _src: &CommandSender<'a>,
        _server: &'a Server,
        args: &mut RawArgs<'a>,
    ) -> Option<Arg<'a>> {
        let x = args.pop()?.parse::<f64>().ok()?;
        let z = args.pop()?.parse::<f64>().ok()?;

        Some(Arg::Pos2D(Vector2::new(x, z)))
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
