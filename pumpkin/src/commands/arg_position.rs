use async_trait::async_trait;

use crate::commands::dispatcher::InvalidTreeError;
use crate::commands::tree::{ConsumedArgs, RawArgs};
use crate::commands::CommandSender;
use crate::server::Server;

use super::tree::ArgumentConsumer;

/// TODO: Seperate base functionality of these two methods into single method

/// todo: implement (so far only own name + @s/@p is implemented)
pub(crate) struct PositionArgumentConsumer;

#[async_trait]
impl ArgumentConsumer for PositionArgumentConsumer {
    async fn consume<'a>(
        &self,
        _src: &CommandSender<'a>,
        _server: &Server,
        args: &mut RawArgs<'a>,
    ) -> Result<String, Option<String>> {
        let Some(arg) = args.pop() else {
            return Err(None);
        };

        // TODO implement ~ ^ notations
        let value = arg.parse::<f64>().map_err(|err| Some(err.to_string()))?;
        Ok(value.to_string())
    }
}

pub fn parse_arg_position(
    arg_name: &str,
    consumed_args: &ConsumedArgs<'_>,
) -> Result<f64, InvalidTreeError> {
    let s = consumed_args
        .get(arg_name)
        .ok_or(InvalidTreeError::InvalidConsumptionError(None))?;

    let value = s
        .parse::<f64>()
        .map_err(|err| InvalidTreeError::InvalidConsumptionError(Some(err.to_string())))?;
    Ok(value)
}
