use std::num::ParseFloatError;

use async_trait::async_trait;
use pumpkin_core::math::vector3::Vector3;

use crate::command::dispatcher::CommandError;
use crate::command::tree::RawArgs;
use crate::command::CommandSender;
use crate::server::Server;

use super::super::args::ArgumentConsumer;
use super::{Arg, DefaultNameArgConsumer, FindArg};

/// x, y and z coordinates
pub(crate) struct Position3DArgumentConsumer;

#[async_trait]
impl ArgumentConsumer for Position3DArgumentConsumer {
    async fn consume<'a>(
        &self,
        src: &CommandSender<'a>,
        _server: &'a Server,
        args: &mut RawArgs<'a>,
    ) -> Option<Arg<'a>> {
        let pos = Position3D::try_new(args.pop(), args.pop(), args.pop())?;

        let vec3 = pos.try_get_values(src.position())?;

        Some(Arg::Pos3D(vec3))
    }
}

struct Position3D(Coordinate, Coordinate, Coordinate);

impl Position3D {
    fn try_new(x: Option<&str>, y: Option<&str>, z: Option<&str>) -> Option<Self> {
        Some(Self(
            x?.try_into().ok()?,
            y?.try_into().ok()?,
            z?.try_into().ok()?,
        ))
    }

    fn try_get_values(self, origin: Option<Vector3<f64>>) -> Option<Vector3<f64>> {
        Some(Vector3::new(
            self.0.value(origin.map(|o| o.x))?,
            self.1.value(origin.map(|o| o.y))?,
            self.2.value(origin.map(|o| o.z))?,
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

enum Coordinate {
    Absolute(f64),
    Relative(f64),
}

impl TryFrom<&str> for Coordinate {
    type Error = ParseFloatError;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        if let Some(s) = s.strip_prefix('~') {
            let offset = if s.is_empty() { 0.0 } else { s.parse()? };
            Ok(Self::Relative(offset))
        } else {
            Ok(Self::Absolute(s.parse()?))
        }
    }
}

impl Coordinate {
    fn value(self, origin: Option<f64>) -> Option<f64> {
        match self {
            Self::Absolute(v) => Some(v),
            Self::Relative(offset) => Some(origin? + offset),
        }
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
