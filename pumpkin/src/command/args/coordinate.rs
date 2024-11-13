use std::str::FromStr;

pub enum Coordinate<const IS_Y: bool> {
    Absolute(f64),
    Relative(f64),
}

impl<const IS_Y: bool> TryFrom<&str> for Coordinate<IS_Y> {
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

impl<const IS_Y: bool> Coordinate<IS_Y> {
    pub fn value(self, origin: Option<f64>) -> Option<f64> {
        match self {
            Self::Absolute(v) => Some(v),
            Self::Relative(offset) => Some(origin? + offset),
        }
    }
}

#[derive(Debug)]
pub enum BlockCoordinate {
    Absolute(i32),
    Relative(i32),
}

impl TryFrom<&str> for BlockCoordinate {
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

impl BlockCoordinate {
    pub fn value(self, origin: Option<f64>) -> Option<i32> {
        match self {
            Self::Absolute(v) => Some(v),
            Self::Relative(offset) => Some(origin?.floor() as i32 + offset),
        }
    }
}
