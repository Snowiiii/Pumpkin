use async_trait::async_trait;
use pumpkin_core::math::vector3::Vector3;
use pumpkin_protocol::client::play::{
    CommandSuggestion, ProtoCmdArgParser, ProtoCmdArgSuggestionType,
};

use crate::command::dispatcher::CommandError;
use crate::command::tree::RawArgs;
use crate::command::CommandSender;
use crate::server::Server;

use super::super::args::ArgumentConsumer;
use super::coordinate::MaybeRelativeCoordinate;
use super::{Arg, DefaultNameArgConsumer, FindArg, GetClientSideArgParser};

/// x, y and z coordinates
pub(crate) struct Position3DArgumentConsumer;

impl GetClientSideArgParser for Position3DArgumentConsumer {
    fn get_client_side_parser(&self) -> ProtoCmdArgParser {
        ProtoCmdArgParser::Vec3
    }

    fn get_client_side_suggestion_type_override(&self) -> Option<ProtoCmdArgSuggestionType> {
        None
    }
}

#[async_trait]
impl ArgumentConsumer for Position3DArgumentConsumer {
    async fn consume<'a>(
        &self,
        src: &CommandSender<'a>,
        _server: &'a Server,
        args: &mut RawArgs<'a>,
    ) -> Option<Arg<'a>> {
        let pos = MaybeRelativePosition3D::try_new(args.pop()?, args.pop()?, args.pop()?)?;

        let vec3 = pos.try_to_absolute(src.position())?;

        Some(Arg::Pos3D(vec3))
    }

    async fn suggest<'a>(
        &self,
        _sender: &CommandSender<'a>,
        _server: &'a Server,
        _input: &'a str,
    ) -> Result<Option<Vec<CommandSuggestion<'a>>>, CommandError> {
        Ok(None)
    }
}

struct MaybeRelativePosition3D(
    MaybeRelativeCoordinate<false>,
    MaybeRelativeCoordinate<true>,
    MaybeRelativeCoordinate<false>,
);

impl MaybeRelativePosition3D {
    fn try_new(x: &str, y: &str, z: &str) -> Option<Self> {
        Some(Self(
            x.try_into().ok()?,
            y.try_into().ok()?,
            z.try_into().ok()?,
        ))
    }

    fn try_to_absolute(self, origin: Option<Vector3<f64>>) -> Option<Vector3<f64>> {
        Some(Vector3::new(
            self.0.into_absolute(origin.map(|o| o.x))?,
            self.1.into_absolute(origin.map(|o| o.y))?,
            self.2.into_absolute(origin.map(|o| o.z))?,
        ))
    }
}

impl DefaultNameArgConsumer for Position3DArgumentConsumer {
    fn default_name(&self) -> &'static str {
        "pos"
    }

    fn get_argument_consumer(&self) -> &dyn ArgumentConsumer {
        &Position3DArgumentConsumer
    }
}

impl<'a> FindArg<'a> for Position3DArgumentConsumer {
    type Data = Vector3<f64>;

    fn find_arg(args: &'a super::ConsumedArgs, name: &'a str) -> Result<Self::Data, CommandError> {
        match args.get(name) {
            Some(Arg::Pos3D(data)) => Ok(*data),
            _ => Err(CommandError::InvalidConsumption(Some(name.to_string()))),
        }
    }
}
