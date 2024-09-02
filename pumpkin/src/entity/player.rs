use std::sync::Arc;

use num_derive::FromPrimitive;
use num_traits::ToPrimitive;
use pumpkin_core::{
    math::{boundingbox::BoundingBox, position::WorldPosition, vector3::Vector3},
    text::TextComponent,
    GameMode,
};
use pumpkin_entity::{entity_type::EntityType, pose::EntityPose, Entity, EntityId};
use pumpkin_inventory::player::PlayerInventory;
use pumpkin_protocol::{
    bytebuf::{packet_id::Packet, DeserializerError},
    client::play::{
        CGameEvent, CPlayDisconnect, CPlayerAbilities, CPlayerInfoUpdate, CSetEntityMetadata,
        CSyncPlayerPosition, CSystemChatMessage, Metadata, PlayerAction,
    },
    server::play::{
        SChatCommand, SChatMessage, SClickContainer, SClientInformationPlay, SConfirmTeleport,
        SInteract, SPlayPingRequest, SPlayerAction, SPlayerCommand, SPlayerPosition,
        SPlayerPositionRotation, SPlayerRotation, SSetCreativeSlot, SSetHeldItem, SSetPlayerGround,
        SSwingArm, SUseItem, SUseItemOn,
    },
    ConnectionState, RawPacket, ServerPacket, VarInt,
};

use pumpkin_protocol::server::play::SCloseContainer;
use pumpkin_world::item::ItemStack;

use crate::{
    client::{authentication::GameProfile, Client, PlayerConfig},
    server::Server,
    world::World,
};

pub struct PlayerAbilities {
    pub invulnerable: bool,
    pub flying: bool,
    pub allow_flying: bool,
    pub creative: bool,
    pub fly_speed: f32,
    pub walk_speed_fov: f32,
}

impl Default for PlayerAbilities {
    fn default() -> Self {
        Self {
            invulnerable: false,
            flying: false,
            allow_flying: false,
            creative: false,
            fly_speed: 0.5,
            walk_speed_fov: 0.1,
        }
    }
}

pub struct Player {
    pub gameprofile: GameProfile,
    pub client: Client,
    pub entity: Entity,
    pub config: PlayerConfig,
    // TODO: Put this into entity
    pub world: Arc<tokio::sync::Mutex<World>>,
    /// Current gamemode
    pub gamemode: GameMode,
    // TODO: prbly should put this into an Living Entitiy or something
    pub health: f32,
    pub food: i32,
    pub food_saturation: f32,
    pub inventory: PlayerInventory,
    pub open_container: Option<u64>,
    pub carried_item: Option<ItemStack>,

    /// send `send_abilties_update` when changed
    pub abilities: PlayerAbilities,

    pub lastx: f64,
    pub lasty: f64,
    pub lastz: f64,
    // Client side value, Should be not trusted
    pub on_ground: bool,

    pub sneaking: bool,
    pub sprinting: bool,

    // TODO: This is currently unused, We have to calculate the block breaking speed our own and then break the block our own if its done
    pub current_block_destroy_stage: u8,

    // TODO: prbly should put this into an Living Entitiy or something
    pub velocity: Vector3<f64>,

    pub teleport_id_count: i32,
    // Current awaiting teleport id and location, None if did not teleport
    pub awaiting_teleport: Option<(VarInt, Vector3<f64>)>,

    pub watched_section: Vector3<i32>,
}

impl Player {
    pub fn new(
        client: Client,
        world: Arc<tokio::sync::Mutex<World>>,
        entity_id: EntityId,
        gamemode: GameMode,
    ) -> Self {
        let gameprofile = match client.gameprofile.clone() {
            Some(profile) => profile,
            None => {
                log::error!("No gameprofile?. Impossible");
                GameProfile {
                    id: uuid::Uuid::new_v4(),
                    name: "".to_string(),
                    properties: vec![],
                    profile_actions: None,
                }
            }
        };
        let config = client.config.clone().unwrap_or_default();
        Self {
            config,
            gameprofile,
            client,
            entity: Entity::new(entity_id, EntityType::Player, 1.62),
            world,
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
            open_container: None,
            carried_item: None,
            teleport_id_count: 0,
            abilities: PlayerAbilities::default(),
            gamemode,
            watched_section: Vector3::new(0, 0, 0),
            lastx: 0.0,
            lasty: 0.0,
            lastz: 0.0,
        }
    }

    // TODO: Put this into entity
    /// Removes the Player out of the current World
    pub async fn remove(&mut self) {
        self.world.lock().await.remove_player(self);
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

    pub fn send_abilties_update(&mut self) {
        let mut b = 0i8;
        let abilities = &self.abilities;

        if abilities.invulnerable {
            b |= 1;
        }
        if abilities.flying {
            b |= 2;
        }
        if abilities.allow_flying {
            b |= 4;
        }
        if abilities.creative {
            b |= 8;
        }
        self.client.send_packet(&CPlayerAbilities::new(
            b,
            abilities.fly_speed,
            abilities.walk_speed_fov,
        ));
    }

    pub async fn set_sneaking(&mut self, sneaking: bool) {
        assert!(self.sneaking != sneaking);
        self.sneaking = sneaking;
        self.set_flag(Self::SNEAKING_FLAG_INDEX, sneaking).await;
        // if sneaking {
        //     self.set_pose(EntityPose::Crouching).await;
        // } else {
        //     self.set_pose(EntityPose::Standing).await;
        // }
    }

    pub async fn set_sprinting(&mut self, sprinting: bool) {
        assert!(self.sprinting != sprinting);
        self.sprinting = sprinting;
        self.set_flag(Self::SPRINTING_FLAG_INDEX, sprinting).await;
    }

    pub const ON_FIRE_FLAG_INDEX: u32 = 0;
    pub const SNEAKING_FLAG_INDEX: u32 = 1;
    pub const SPRINTING_FLAG_INDEX: u32 = 3;
    pub const SWIMMING_FLAG_INDEX: u32 = 4;
    pub const INVISIBLE_FLAG_INDEX: u32 = 5;
    pub const GLOWING_FLAG_INDEX: u32 = 6;
    pub const FALL_FLYING_FLAG_INDEX: u32 = 7;
    async fn set_flag(&mut self, index: u32, value: bool) {
        let mut b = 0i8;
        if value {
            b |= 1 << index;
        } else {
            b &= !(1 << index);
        }
        let packet =
            CSetEntityMetadata::new(self.entity_id().into(), Metadata::new(0, 0.into(), b));
        self.client.send_packet(&packet);
        self.world
            .lock()
            .await
            .broadcast_packet(&[&self.client.token], &packet);
    }

    pub async fn set_pose(&mut self, pose: EntityPose) {
        self.entity.pose = pose;
        let pose = self.entity.pose as i32;
        let packet = CSetEntityMetadata::<VarInt>::new(
            self.entity_id().into(),
            Metadata::new(6, 20.into(), (pose).into()),
        );
        self.client.send_packet(&packet);
        self.world
            .lock()
            .await
            .broadcast_packet(&[&self.client.token], &packet)
    }

    pub fn teleport(&mut self, x: f64, y: f64, z: f64, yaw: f32, pitch: f32) {
        // this is the ultra special magic code used to create the teleport id
        self.teleport_id_count += 1;
        if self.teleport_id_count == i32::MAX {
            self.teleport_id_count = 0;
        }
        let entity = &mut self.entity;
        entity.set_pos(x, y, z);
        self.lastx = x;
        self.lasty = y;
        self.lastz = z;
        entity.yaw = yaw;
        entity.pitch = pitch;
        self.awaiting_teleport = Some((self.teleport_id_count.into(), Vector3::new(x, y, z)));
        self.client.send_packet(&CSyncPlayerPosition::new(
            x,
            y,
            z,
            yaw,
            pitch,
            0,
            self.teleport_id_count.into(),
        ));
    }

    pub fn block_interaction_range(&self) -> f64 {
        if self.gamemode == GameMode::Creative {
            5.0
        } else {
            4.5
        }
    }

    pub fn can_interact_with_block_at(&self, pos: &WorldPosition, additional_range: f64) -> bool {
        let d = self.block_interaction_range() + additional_range;
        let box_pos = BoundingBox::from_block(pos);
        box_pos.squared_magnitude(Vector3 {
            x: self.entity.pos.x,
            y: self.entity.pos.y + self.entity.standing_eye_height as f64,
            z: self.entity.pos.z,
        }) < d * d
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

    pub async fn set_gamemode(&mut self, gamemode: GameMode) {
        // We could send the same gamemode without problems. But why waste bandwidth ?
        assert!(
            self.gamemode != gamemode,
            "Setting the same gamemode as already is"
        );
        self.gamemode = gamemode;
        // So a little story time. I actually made an abitlties_from_gamemode function. I looked at vanilla and they always send the abilties from the gamemode. But the funny thing actually is. That the client
        // does actually use the same method and set the abilties when receiving the CGameEvent gamemode packet. Just Mojang nonsense

        // TODO: fix this ugly mess :c, It gives me a liftime error when saving packet as a var
        self.client.send_packet(&CPlayerInfoUpdate::new(
            0x04,
            &[pumpkin_protocol::client::play::Player {
                uuid: self.gameprofile.id,
                actions: vec![PlayerAction::UpdateGameMode((self.gamemode as i32).into())],
            }],
        ));
        self.world.lock().await.broadcast_packet(
            &[&self.client.token],
            &CPlayerInfoUpdate::new(
                0x04,
                &[pumpkin_protocol::client::play::Player {
                    uuid: self.gameprofile.id,
                    actions: vec![PlayerAction::UpdateGameMode((self.gamemode as i32).into())],
                }],
            ),
        );
        self.client
            .send_packet(&CGameEvent::new(3, gamemode.to_f32().unwrap()));
    }

    pub fn send_system_message(&mut self, text: TextComponent) {
        self.client
            .send_packet(&CSystemChatMessage::new(text, false));
    }
}

impl Player {
    pub async fn process_packets(&mut self, server: &mut Server) {
        while let Some(mut packet) = self.client.client_packets_queue.pop() {
            match self.handle_play_packet(server, &mut packet).await {
                Ok(_) => {}
                Err(e) => {
                    let text = format!("Error while reading incoming packet {}", e);
                    log::error!("{}", text);
                    self.kick(TextComponent::text(&text))
                }
            };
        }
    }

    pub async fn handle_play_packet(
        &mut self,
        server: &mut Server,
        packet: &mut RawPacket,
    ) -> Result<(), DeserializerError> {
        let bytebuf = &mut packet.bytebuf;
        match packet.id.0 {
            SConfirmTeleport::PACKET_ID => {
                self.handle_confirm_teleport(server, SConfirmTeleport::read(bytebuf)?);
                Ok(())
            }
            SChatCommand::PACKET_ID => {
                self.handle_chat_command(server, SChatCommand::read(bytebuf)?);
                Ok(())
            }
            SPlayerPosition::PACKET_ID => {
                self.handle_position(server, SPlayerPosition::read(bytebuf)?)
                    .await;
                Ok(())
            }
            SPlayerPositionRotation::PACKET_ID => {
                self.handle_position_rotation(server, SPlayerPositionRotation::read(bytebuf)?)
                    .await;
                Ok(())
            }
            SPlayerRotation::PACKET_ID => {
                self.handle_rotation(server, SPlayerRotation::read(bytebuf)?)
                    .await;
                Ok(())
            }
            SSetPlayerGround::PACKET_ID => {
                self.handle_player_ground(server, SSetPlayerGround::read(bytebuf)?);
                Ok(())
            }
            SPlayerCommand::PACKET_ID => {
                self.handle_player_command(server, SPlayerCommand::read(bytebuf)?)
                    .await;
                Ok(())
            }
            SSwingArm::PACKET_ID => {
                self.handle_swing_arm(server, SSwingArm::read(bytebuf)?)
                    .await;
                Ok(())
            }
            SChatMessage::PACKET_ID => {
                self.handle_chat_message(server, SChatMessage::read(bytebuf)?)
                    .await;
                Ok(())
            }
            SClientInformationPlay::PACKET_ID => {
                self.handle_client_information_play(server, SClientInformationPlay::read(bytebuf)?);
                Ok(())
            }
            SInteract::PACKET_ID => {
                self.handle_interact(server, SInteract::read(bytebuf)?)
                    .await;
                Ok(())
            }
            SPlayerAction::PACKET_ID => {
                self.handle_player_action(server, SPlayerAction::read(bytebuf)?)
                    .await;
                Ok(())
            }
            SUseItemOn::PACKET_ID => {
                self.handle_use_item_on(server, SUseItemOn::read(bytebuf)?)
                    .await;
                Ok(())
            }
            SUseItem::PACKET_ID => {
                self.handle_use_item(server, SUseItem::read(bytebuf)?);
                Ok(())
            }
            SSetHeldItem::PACKET_ID => {
                self.handle_set_held_item(server, SSetHeldItem::read(bytebuf)?);
                Ok(())
            }
            SSetCreativeSlot::PACKET_ID => {
                self.handle_set_creative_slot(server, SSetCreativeSlot::read(bytebuf)?);
                Ok(())
            }
            SPlayPingRequest::PACKET_ID => {
                self.handle_play_ping_request(server, SPlayPingRequest::read(bytebuf)?);
                Ok(())
            }
            SClickContainer::PACKET_ID => {
                self.handle_click_container(server, SClickContainer::read(bytebuf)?);
                Ok(())
            }
            SCloseContainer::PACKET_ID => {
                self.handle_close_container(server, SCloseContainer::read(bytebuf)?);
                Ok(())
            }
            _ => {
                log::error!("Failed to handle player packet id {:#04x}", packet.id.0);
                Ok(())
            }
        }
    }
}

#[derive(FromPrimitive, Clone)]
pub enum Hand {
    Main,
    Off,
}

#[derive(FromPrimitive, Clone)]
pub enum ChatMode {
    Enabled,
    CommandsOnly,
    Hidden,
}
