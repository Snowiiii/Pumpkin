use async_trait::async_trait;

use crate::command::dispatcher::InvalidTreeError;
use crate::command::tree::RawArgs;
use crate::command::CommandSender;
use crate::server::Server;

use super::super::args::ArgumentConsumer;
use super::{Arg, DefaultNameArgConsumer, FindArg};

/// yaw and pitch
pub(crate) struct RotationArgumentConsumer;

#[async_trait]
impl ArgumentConsumer for RotationArgumentConsumer {
    async fn consume<'a>(
        &self,
        _src: &CommandSender<'a>,
        _server: &'a Server,
        args: &mut RawArgs<'a>,
    ) -> Option<Arg<'a>> {
        let mut yaw = args.pop()?.parse::<f32>().ok()?;
        let mut pitch = args.pop()?.parse::<f32>().ok()?;

        yaw %= 360.0;
        if yaw >= 180.0 {
            yaw -= 360.0;
        };
        pitch %= 360.0;
        if pitch >= 180.0 {
            pitch -= 360.0;
        };

        Some(Arg::Rotation(yaw, pitch))
    }
}

impl DefaultNameArgConsumer for RotationArgumentConsumer {
    fn default_name(&self) -> &'static str {
        "rotation"
    }

    fn get_argument_consumer(&self) -> &dyn ArgumentConsumer {
        &RotationArgumentConsumer
    }
}

impl<'a> FindArg<'a> for RotationArgumentConsumer {

    type Data = (f32, f32);

    fn find_arg(args: &'a super::ConsumedArgs, name: &'a str) -> Result<Self::Data, InvalidTreeError> {
        match args.get(name) {
            Some(Arg::Rotation(yaw, pitch)) => Ok((*yaw, *pitch)),
            _ => Err(InvalidTreeError::InvalidConsumptionError(Some(name.to_string())))
        }
    }
}