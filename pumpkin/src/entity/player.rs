use std::str::FromStr;

use num_derive::{FromPrimitive, ToPrimitive};
use pumpkin_entity::{entity_type::EntityType, Entity, EntityId};
use pumpkin_protocol::VarInt;
use serde::{Deserialize, Serialize};

pub struct Player {
    pub entity: Entity,

    // Client side value, Should be not trusted
    pub on_ground: bool,

    pub sneaking: bool,
    pub sprinting: bool,

    // Current awaiting teleport id, None if did not teleport
    pub awaiting_teleport: Option<VarInt>,
}

impl Player {
    pub fn new(entity_id: EntityId) -> Self {
        Self {
            entity: Entity::new(entity_id, EntityType::Player),
            on_ground: false,
            awaiting_teleport: None,
            sneaking: false,
            sprinting: false,
        }
    }

    pub fn entity_id(&self) -> EntityId {
        self.entity.entity_id
    }
}

#[derive(FromPrimitive)]
pub enum Hand {
    Main,
    Off,
}

#[derive(FromPrimitive)]
pub enum ChatMode {
    Enabled,
    CommandsOnly,
    Hidden,
}

#[derive(Clone, Copy, PartialEq, Serialize, Deserialize, FromPrimitive, ToPrimitive)]
pub enum GameMode {
    Undefined = -1,
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
