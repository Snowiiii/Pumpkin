use std::str::FromStr;

use num_derive::{FromPrimitive, ToPrimitive};
use num_traits::ToPrimitive;
use pumpkin_core::text::TextComponent;
use pumpkin_entity::{entity_type::EntityType, Entity, EntityId};
use pumpkin_inventory::player::PlayerInventory;
use pumpkin_protocol::{
    bytebuf::packet_id::Packet,
    client::play::{CGameEvent, CPlayDisconnect, CSyncPlayerPosition, CSystemChatMessage},
    server::play::{
        SChatCommand, SChatMessage, SClientInformationPlay, SConfirmTeleport, SInteract,
        SPlayPingRequest, SPlayerAction, SPlayerCommand, SPlayerPosition, SPlayerPositionRotation,
        SPlayerRotation, SSetCreativeSlot, SSetHeldItem, SSwingArm, SUseItemOn,
    },
    ConnectionState, RawPacket, ServerPacket, VarInt,
};
use pumpkin_world::vector3::Vector3;
use serde::{Deserialize, Serialize};

use crate::{client::Client, server::Server};

pub struct Player {
    pub client: Client,
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

    // TODO: This is currently unused, We have to calculate the block breaking speed our own and then break the block our own if its done
    pub current_block_destroy_stage: u8,

    // TODO: prbly should put this into an Living Entitiy or something
    pub velocity: Vector3<f64>,

    // Current awaiting teleport id, None if did not teleport
    pub awaiting_teleport: Option<VarInt>,
}

impl Player {
    pub fn new(client: Client, entity_id: EntityId, gamemode: GameMode) -> Self {
        Self {
            client,
            entity: Entity::new(entity_id, EntityType::Player),
            on_ground: false,
            awaiting_teleport: None,
            sneaking: false,
            sprinting: false,
            // TODO: Load this from previous instance
            health: 20.0,
            food: 20,
            food_saturation: 20.0,
            current_block_destroy_stage: 0,
            velocity: Vector3::new(0.0, 0.0, 0.0),
            inventory: PlayerInventory::new(),
            gamemode,
        }
    }

    pub fn entity_id(&self) -> EntityId {
        self.entity.entity_id
    }

    pub fn knockback(&mut self, strength: f64, x: f64, z: f64) {
        // This has some vanilla magic
        let mut x = x;
        let mut z = z;
        while x * x + z * z < 1.0E-5 {
            x = (rand::random::<f64>() - rand::random::<f64>()) * 0.01;
            z = (rand::random::<f64>() - rand::random::<f64>()) * 0.01;
        }

        let var8 = Vector3::new(x, 0.0, z).normalize() * strength;
        let var7 = self.velocity;
        self.velocity = Vector3::new(
            var7.x / 2.0 - var8.x,
            if self.on_ground {
                (var7.y / 2.0 + strength).min(0.4)
            } else {
                var7.y
            },
            var7.z / 2.0 - var8.z,
        );
    }

    pub fn teleport(&mut self, x: f64, y: f64, z: f64, yaw: f32, pitch: f32) {
        // TODO
        let id = 0;
        let entity = &mut self.entity;
        entity.x = x;
        entity.y = y;
        entity.z = z;
        entity.lastx = x;
        entity.lasty = y;
        entity.lastz = z;
        entity.yaw = yaw;
        entity.pitch = pitch;
        self.awaiting_teleport = Some(id.into());
        self.client
            .send_packet(&CSyncPlayerPosition::new(x, y, z, yaw, pitch, 0, id.into()));
    }

    /// Kicks the Client with a reason depending on the connection state
    pub fn kick(&mut self, reason: TextComponent) {
        assert!(self.client.connection_state == ConnectionState::Play);
        dbg!(&reason);
        self.client
            .try_send_packet(&CPlayDisconnect::new(reason))
            .unwrap_or_else(|_| self.client.close());

        self.client.close()
    }

    pub fn update_health(&mut self, health: f32, food: i32, food_saturation: f32) {
        self.health = health;
        self.food = food;
        self.food_saturation = food_saturation;
    }

    pub fn set_gamemode(&mut self, gamemode: GameMode) {
        self.gamemode = gamemode;
        self.client
            .send_packet(&CGameEvent::new(3, gamemode.to_f32().unwrap()));
    }

    pub fn send_system_message(&mut self, text: TextComponent) {
        self.client
            .send_packet(&CSystemChatMessage::new(text, false));
    }
}

impl Player {
    pub fn process_packets(&mut self, server: &mut Server) {
        let mut i = 0;
        while i < self.client.client_packets_queue.len() {
            let mut packet = self.client.client_packets_queue.remove(i).unwrap();
            self.handle_play_packet(server, &mut packet);
            i += 1;
        }
    }

    pub fn handle_play_packet(&mut self, server: &mut Server, packet: &mut RawPacket) {
        let bytebuf = &mut packet.bytebuf;
        match packet.id.0 {
            SConfirmTeleport::PACKET_ID => {
                self.handle_confirm_teleport(server, SConfirmTeleport::read(bytebuf).unwrap())
            }
            SChatCommand::PACKET_ID => {
                self.handle_chat_command(server, SChatCommand::read(bytebuf).unwrap())
            }
            SPlayerPosition::PACKET_ID => {
                self.handle_position(server, SPlayerPosition::read(bytebuf).unwrap())
            }
            SPlayerPositionRotation::PACKET_ID => self
                .handle_position_rotation(server, SPlayerPositionRotation::read(bytebuf).unwrap()),
            SPlayerRotation::PACKET_ID => {
                self.handle_rotation(server, SPlayerRotation::read(bytebuf).unwrap())
            }
            SPlayerCommand::PACKET_ID => {
                self.handle_player_command(server, SPlayerCommand::read(bytebuf).unwrap())
            }
            SSwingArm::PACKET_ID => {
                self.handle_swing_arm(server, SSwingArm::read(bytebuf).unwrap())
            }
            SChatMessage::PACKET_ID => {
                self.handle_chat_message(server, SChatMessage::read(bytebuf).unwrap())
            }
            SClientInformationPlay::PACKET_ID => self.handle_client_information_play(
                server,
                SClientInformationPlay::read(bytebuf).unwrap(),
            ),
            SInteract::PACKET_ID => self.handle_interact(server, SInteract::read(bytebuf).unwrap()),
            SPlayerAction::PACKET_ID => {
                self.handle_player_action(server, SPlayerAction::read(bytebuf).unwrap())
            }
            SUseItemOn::PACKET_ID => {
                self.handle_use_item_on(server, SUseItemOn::read(bytebuf).unwrap())
            }
            SSetHeldItem::PACKET_ID => {
                self.handle_set_held_item(server, SSetHeldItem::read(bytebuf).unwrap())
            }
            SSetCreativeSlot::PACKET_ID => {
                self.handle_set_creative_slot(server, SSetCreativeSlot::read(bytebuf).unwrap())
            }
            SPlayPingRequest::PACKET_ID => {
                self.handle_play_ping_request(server, SPlayPingRequest::read(bytebuf).unwrap())
            }
            _ => log::error!("Failed to handle player packet id {:#04x}", packet.id.0),
        }
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
