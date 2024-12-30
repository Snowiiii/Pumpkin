use async_trait::async_trait;
use pumpkin_protocol::client::play::{
    CommandSuggestion, ProtoCmdArgParser, ProtoCmdArgSuggestionType,
};

use crate::command::dispatcher::CommandError;
use crate::command::tree::RawArgs;
use crate::command::CommandSender;
use crate::server::Server;

use super::super::args::ArgumentConsumer;
use super::{Arg, DefaultNameArgConsumer, FindArg, GetClientSideArgParser};

/// yaw and pitch
pub struct RotationArgumentConsumer;

impl GetClientSideArgParser for RotationArgumentConsumer {
    fn get_client_side_parser(&self) -> ProtoCmdArgParser {
        ProtoCmdArgParser::Rotation
    }

    fn get_client_side_suggestion_type_override(&self) -> Option<ProtoCmdArgSuggestionType> {
        None
    }
}

#[async_trait]
impl ArgumentConsumer for RotationArgumentConsumer {
    async fn consume<'a>(
        &'a self,
        _src: &CommandSender<'a>,
        _server: &'a Server,
        args: &mut RawArgs<'a>,
    ) -> Option<Arg<'a>> {
        let yaw = args.pop()?;
        let pitch = args.pop()?;

        let mut yaw = yaw.parse::<f32>().ok()?;
        let mut pitch = pitch.parse::<f32>().ok()?;

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

    async fn suggest<'a>(
        &'a self,
        _sender: &CommandSender<'a>,
        _server: &'a Server,
        _input: &'a str,
    ) -> Result<Option<Vec<CommandSuggestion>>, CommandError> {
        Ok(None)
    }
}

impl DefaultNameArgConsumer for RotationArgumentConsumer {
    fn default_name(&self) -> String {
        "rotation".to_string()
    }
}

impl<'a> FindArg<'a> for RotationArgumentConsumer {
    type Data = (f32, f32);

    fn find_arg(args: &'a super::ConsumedArgs, name: &str) -> Result<Self::Data, CommandError> {
        match args.get(name) {
            Some(Arg::Rotation(yaw, pitch)) => Ok((*yaw, *pitch)),
            _ => Err(CommandError::InvalidConsumption(Some(name.to_string()))),
        }
    }
}
