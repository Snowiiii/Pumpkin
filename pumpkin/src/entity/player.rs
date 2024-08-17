use std::str::FromStr;

use num_derive::{FromPrimitive, ToPrimitive};
use num_traits::Float;
use pumpkin_entity::{entity_type::EntityType, Entity, EntityId};
use pumpkin_inventory::player::PlayerInventory;
use pumpkin_protocol::VarInt;
use pumpkin_world::vector3::Vector3;
use serde::{Deserialize, Serialize};

pub struct Player {
    pub entity: Entity,
    // current gamemode
    pub gamemode: GameMode,
    // TODO: prbly should put this into an Living Entitiy or something
    pub health: f32,
    pub food: i32,
    pub food_saturation: f32,
    pub inventory: PlayerInventory,

    // Client side value, Should be not trusted
    pub on_ground: bool,

    pub sneaking: bool,
    pub sprinting: bool,

    // TODO: prbly should put this into an Living Entitiy or something
    pub velocity: Vector3<f64>,

    // Current awaiting teleport id, None if did not teleport
    pub awaiting_teleport: Option<VarInt>,
}

impl Player {
    pub fn new(entity_id: EntityId, gamemode: GameMode) -> Self {
        Self {
            entity: Entity::new(entity_id, EntityType::Player),
            on_ground: false,
            awaiting_teleport: None,
            sneaking: false,
            sprinting: false,
            // TODO: Load this from previous instance
            health: 20.0,
            food: 20,
            food_saturation: 20.0,
            velocity: Vector3::new(0.0, 0.0, 0.0),
            inventory: PlayerInventory::new(),
            gamemode,
        }
    }

    pub fn entity_id(&self) -> EntityId {
        self.entity.entity_id
    }

    pub fn knockback(&mut self, y: f64, x: f64, z: f64) {
        // This has some vanilla magic
        let mut x = x;
        let mut z = z;
        while x * x + z * z < 9.999999747378752E-6 {
            x = (rand::random::<f64>() - rand::random::<f64>()) * 0.01;
            z = (rand::random::<f64>() - rand::random::<f64>()) * 0.01;
        }

        let var8 = Vector3::new(x, 0.0, z).normalize() * y;
        let var7 = self.velocity;
        self.velocity = Vector3::new(
            var7.x / 2.0 - var8.x,
            if self.on_ground {
                (var7.y / 2.0 + x).min(0.4)
            } else {
                var7.y
            },
            var7.z / 2.0 - var8.z,
        );
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

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize, FromPrimitive, ToPrimitive)]
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
