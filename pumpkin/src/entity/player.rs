use std::str::FromStr;

use pumpkin_protocol::VarInt;
use serde::{Deserialize, Serialize};

use super::{Entity, EntityId};

pub struct Player {
    pub entity: Entity,
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub yaw: f32,
    pub pitch: f32,

    // Client side value, Should be not trusted
    pub on_ground: bool,

    // Current awaiting teleport id, None if did not teleport
    pub awaiting_teleport: Option<VarInt>,
}

impl Player {
    pub fn new(entity: Entity) -> Self {
        Self {
            entity,
            x: 0.0,
            y: 0.0,
            z: 0.0,
            yaw: 0.0,
            pitch: 0.0,
            on_ground: false,
            awaiting_teleport: None,
        }
    }

    pub fn entity_id(&self) -> EntityId {
        self.entity.entity_id
    }
}

pub enum Hand {
    Main,
    Off,
}

impl From<VarInt> for Hand {
    fn from(value: VarInt) -> Self {
        match value {
            0 => Self::Off,
            1 => Self::Main,
            _ => {
                log::info!("Unexpected Hand {}", value);
                Self::Main
            }
        }
    }
}

pub enum ChatMode {
    Enabled,
    CommandsOnly,
    Hidden,
}

impl From<VarInt> for ChatMode {
    fn from(value: VarInt) -> Self {
        match value {
            0 => Self::Enabled,
            1 => Self::CommandsOnly,
            2 => Self::Hidden,
            _ => {
                log::info!("Unexpected ChatMode {}", value);
                Self::Enabled
            }
        }
    }
}

#[derive(Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum GameMode {
    Undefined,
    Survival,
    Creative,
    Adventure,
    Spectator,
}

#[derive(Debug, PartialEq, Eq)]
pub struct ParseGameModeError;

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

impl From<i8> for GameMode {
    fn from(value: i8) -> Self {
        match value {
            -1 => GameMode::Undefined,
            0 => GameMode::Survival,
            1 => GameMode::Creative,
            2 => GameMode::Adventure,
            3 => GameMode::Spectator,
            _ => {
                log::info!("Unexpected GameMode {}", value);
                Self::Survival
            }
        }
    }
}

impl GameMode {
    pub fn to_byte(self) -> i8 {
        match self {
            Self::Undefined => -1,
            Self::Survival => 0,
            Self::Creative => 1,
            Self::Adventure => 2,
            Self::Spectator => 3,
        }
    }
}
