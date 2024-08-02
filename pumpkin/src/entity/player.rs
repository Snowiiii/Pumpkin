use pumpkin_protocol::VarInt;

use super::{Entity, EntityId};

pub struct Player {
    pub entity: Entity,
}

impl Player {
    pub fn new(entity: Entity) -> Self {
        Self { entity }
    }

    pub fn entity_id(&self) -> EntityId {
        self.entity.entity_id
    }
}

pub enum Hand {
    Main,
    Off,
}

impl Hand {
    pub fn from_varint(varint: VarInt) -> Self {
        match varint {
            0 => Self::Off,
            1 => Self::Main,
            _ => {
                log::info!("Unexpected Hand {}", varint);
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

impl ChatMode {
    pub fn from_varint(varint: VarInt) -> Self {
        match varint {
            0 => Self::Enabled,
            1 => Self::CommandsOnly,
            2 => Self::Hidden,
            _ => {
                log::info!("Unexpected ChatMode {}", varint);
                Self::Enabled
            }
        }
    }
}

#[derive(Clone, Copy)]
pub enum GameMode {
    Undefined,
    Survival,
    Creative,
    Adventure,
    Spectator,
}

impl GameMode {
    pub fn from_byte(byte: i8) -> Self {
        match byte {
            -1 => GameMode::Undefined,
            0 => GameMode::Survival,
            1 => GameMode::Creative,
            2 => GameMode::Adventure,
            3 => GameMode::Spectator,
            _ => {
                log::info!("Unexpected GameMode {}", byte);
                Self::Survival
            }
        }
    }

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
