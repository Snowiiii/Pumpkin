use std::str::FromStr;

use num_derive::FromPrimitive;
use serde::{Deserialize, Serialize};
#[cfg(feature = "schemars")]
use schemars::JsonSchema;

#[derive(Debug, PartialEq, Eq)]
pub struct ParseGameModeError;

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize, FromPrimitive)]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
#[repr(i8)]
pub enum GameMode {
    Undefined = -1,
    Survival,
    Creative,
    Adventure,
    Spectator,
}

impl FromStr for GameMode {
    type Err = ParseGameModeError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "survival" => Ok(Self::Survival),
            "creative" => Ok(Self::Creative),
            "adventure" => Ok(Self::Adventure),
            "spectator" => Ok(Self::Spectator),
            _ => Err(ParseGameModeError),
        }
    }
}
