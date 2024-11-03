use std::num::ParseFloatError;

use async_trait::async_trait;
use pumpkin_core::math::vector3::Vector3;

use crate::command::dispatcher::InvalidTreeError;
use crate::command::tree::{ConsumedArgs, RawArgs};
use crate::command::CommandSender;
use crate::server::Server;

use super::tree::ArgumentConsumer;

/// x, y and z coordinates
pub(crate) struct Position3DArgumentConsumer;

#[async_trait]
impl ArgumentConsumer for Position3DArgumentConsumer {
    async fn consume<'a>(
        &self,
        src: &CommandSender<'a>,
        _server: &Server,
        args: &mut RawArgs<'a>,
    ) -> Result<String, Option<String>> {
        let pos = Position3D::try_new(args.pop(), args.pop(), args.pop()).ok_or(None)?;

        let Vector3 { x, y, z } = pos.try_get_values(src.position()).ok_or(None)?;

        Ok(format!("{x} {y} {z}"))
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

/// x, y and z coordinates
pub fn parse_arg_position_3d(
    arg_name: &str,
    consumed_args: &ConsumedArgs<'_>,
) -> Result<Vector3<f64>, InvalidTreeError> {
    let s = consumed_args
        .get(arg_name)
        .ok_or(InvalidTreeError::InvalidConsumptionError(None))?;

    // these whitespaces will always be ascii
    let mut args = s.split_ascii_whitespace();

    let Some(x) = args.next() else {
        return Err(InvalidTreeError::InvalidConsumptionError(Some(s.into())));
    };
    let Some(y) = args.next() else {
        return Err(InvalidTreeError::InvalidConsumptionError(Some(s.into())));
    };
    let Some(z) = args.next() else {
        return Err(InvalidTreeError::InvalidConsumptionError(Some(s.into())));
    };

    let Ok(x) = x.parse::<f64>() else {
        return Err(InvalidTreeError::InvalidConsumptionError(Some(s.into())));
    };
    let Ok(y) = y.parse::<f64>() else {
        return Err(InvalidTreeError::InvalidConsumptionError(Some(s.into())));
    };
    let Ok(z) = z.parse::<f64>() else {
        return Err(InvalidTreeError::InvalidConsumptionError(Some(s.into())));
    };

    Ok(Vector3::new(x, y, z))
}
