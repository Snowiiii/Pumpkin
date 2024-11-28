use std::str::FromStr;

use pumpkin_world::{WORLD_LOWEST_Y, WORLD_MAX_Y};

pub enum MaybeRelativeCoordinate<const IS_Y: bool> {
    Absolute(f64),
    Relative(f64),
}

impl<const IS_Y: bool> TryFrom<&str> for MaybeRelativeCoordinate<IS_Y> {
    type Error = <f64 as FromStr>::Err;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        if let Some(s) = s.strip_prefix('~') {
            let offset = if s.is_empty() { 0.0 } else { s.parse()? };
            Ok(Self::Relative(offset))
        } else {
            let mut v = s.parse()?;

            // set position to block center if no decimal place is given
            if !IS_Y && !s.contains('.') {
                v += 0.5;
            }

            Ok(Self::Absolute(v))
        }
    }
}

impl<const IS_Y: bool> MaybeRelativeCoordinate<IS_Y> {
    pub fn into_absolute(self, origin: Option<f64>) -> Option<f64> {
        match self {
            Self::Absolute(v) => Some(v),
            Self::Relative(offset) => Some(origin? + offset),
        }
    }
}

#[derive(Debug)]
pub enum MaybeRelativeBlockCoordinate<const IS_Y: bool> {
    Absolute(i32),
    Relative(i32),
}

impl<const IS_Y: bool> TryFrom<&str> for MaybeRelativeBlockCoordinate<IS_Y> {
    type Error = <i32 as FromStr>::Err;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        if let Some(s) = s.strip_prefix('~') {
            let offset = if s.is_empty() { 0 } else { s.parse()? };
            Ok(Self::Relative(offset))
        } else {
            Ok(Self::Absolute(s.parse()?))
        }
    }
}

impl<const IS_Y: bool> MaybeRelativeBlockCoordinate<IS_Y> {
    pub fn into_absolute(self, origin: Option<f64>) -> Option<i32> {
        let abs = match self {
            Self::Absolute(v) => v,
            Self::Relative(offset) => origin?.floor() as i32 + offset,
        };

        if IS_Y && (abs < WORLD_LOWEST_Y.into() || abs >= WORLD_MAX_Y.into()) {
            return None;
        }

        Some(abs)
    }
}
