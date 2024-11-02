use async_trait::async_trait;

use crate::command::dispatcher::InvalidTreeError;
use crate::command::tree::{ConsumedArgs, RawArgs};
use crate::command::CommandSender;
use crate::server::Server;

use super::tree::ArgumentConsumer;

/// yaw and pitch
pub(crate) struct RotationArgumentConsumer;

#[async_trait]
impl ArgumentConsumer for RotationArgumentConsumer {
    async fn consume<'a>(
        &self,
        _src: &CommandSender<'a>,
        _server: &Server,
        args: &mut RawArgs<'a>,
    ) -> Result<String, Option<String>> {
        let Some(yaw) = args.pop() else {
            return Err(None);
        };
        let Some(pitch) = args.pop() else {
            return Err(None);
        };

        let mut yaw = yaw.parse::<f32>().map_err(|err| Some(err.to_string()))?;
        let mut pitch = pitch.parse::<f32>().map_err(|err| Some(err.to_string()))?;

        yaw %= 360.0;
        if yaw >= 180.0 {
            yaw -= 360.0;
        };
        pitch %= 360.0;
        if pitch >= 180.0 {
            pitch -= 360.0;
        };

        Ok(format!("{yaw} {pitch}"))
    }
}

/// yaw and pitch
pub fn parse_arg_rotation(
    arg_name: &str,
    consumed_args: &ConsumedArgs<'_>,
) -> Result<(f32, f32), InvalidTreeError> {
    let s = consumed_args
        .get(arg_name)
        .ok_or(InvalidTreeError::InvalidConsumptionError(None))?;

    // these whitespaces will always be ascii
    let mut args = s.split_ascii_whitespace();

    let Some(yaw) = args.next() else {
        return Err(InvalidTreeError::InvalidConsumptionError(Some(s.into())));
    };
    let Some(pitch) = args.next() else {
        return Err(InvalidTreeError::InvalidConsumptionError(Some(s.into())));
    };

    let Ok(yaw) = yaw.parse() else {
        return Err(InvalidTreeError::InvalidConsumptionError(Some(s.into())));
    };
    let Ok(pitch) = pitch.parse() else {
        return Err(InvalidTreeError::InvalidConsumptionError(Some(s.into())));
    };

    Ok((yaw, pitch))
}
