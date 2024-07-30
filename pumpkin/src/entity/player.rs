
use crate::{
    client::Client,
    protocol::{ClientPacket, RawPacket, VarInt},
};

use super::{Entity, EntityId};

pub struct Player {
    pub entity: Entity,
    // All networking stuff
    pub client: Client,
}

impl Player {
    pub fn new(entity: Entity, client: Client) -> Self {
        Self { entity, client }
    }

    pub fn entity_id(&self) -> EntityId {
        self.entity.entity_id
    }

    pub fn handle_packet(&mut self, _packet: &mut RawPacket) {}

    pub fn send_packet<P: ClientPacket>(&mut self, packet: P) {
        self.client.send_packet(packet);
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
