use async_trait::async_trait;
use pumpkin_core::math::vector2::Vector2;

use crate::command::dispatcher::InvalidTreeError;
use crate::command::tree::{ConsumedArgs, RawArgs};
use crate::command::CommandSender;
use crate::server::Server;

use super::tree::ArgumentConsumer;

/// x and z coordinates only
///
/// todo: implememnt ~ ^ notations
pub(crate) struct Position2DArgumentConsumer;

#[async_trait]
impl ArgumentConsumer for Position2DArgumentConsumer {
    async fn consume<'a>(
        &self,
        _src: &CommandSender<'a>,
        _server: &Server,
        args: &mut RawArgs<'a>,
    ) -> Result<String, Option<String>> {
        let Some(x) = args.pop() else {
            return Err(None);
        };
        let Some(z) = args.pop() else {
            return Err(None);
        };

        let x = x.parse::<f64>().map_err(|err| Some(err.to_string()))?;
        let z = z.parse::<f64>().map_err(|err| Some(err.to_string()))?;

        Ok(format!("{x} {z}"))
    }
}

/// x and z coordinates only
pub fn parse_arg_position_2d(
    arg_name: &str,
    consumed_args: &ConsumedArgs<'_>,
) -> Result<Vector2<f64>, InvalidTreeError> {
    let s = consumed_args
        .get(arg_name)
        .ok_or(InvalidTreeError::InvalidConsumptionError(None))?;

    // these whitespaces will always be ascii
    let mut args = s.split_ascii_whitespace();

    let Some(x) = args.next() else {
        return Err(InvalidTreeError::InvalidConsumptionError(Some(s.into())));
    };
    let Some(z) = args.next() else {
        return Err(InvalidTreeError::InvalidConsumptionError(Some(s.into())));
    };

    let Ok(x) = x.parse::<f64>() else {
        return Err(InvalidTreeError::InvalidConsumptionError(Some(s.into())));
    };
    let Ok(z) = z.parse::<f64>() else {
        return Err(InvalidTreeError::InvalidConsumptionError(Some(s.into())));
    };

    Ok(Vector2::new(x, z))
}
