use std::sync::{
    atomic::{AtomicI32, AtomicU8},
    Arc,
};

use crossbeam::atomic::AtomicCell;
use num_derive::FromPrimitive;
use num_traits::ToPrimitive;
use parking_lot::Mutex;
use pumpkin_core::{
    math::{boundingbox::BoundingBox, position::WorldPosition, vector3::Vector3},
    text::TextComponent,
    GameMode,
};
use pumpkin_entity::{entity_type::EntityType, EntityId};
use pumpkin_inventory::player::PlayerInventory;
use pumpkin_protocol::{
    bytebuf::{packet_id::Packet, DeserializerError},
    client::play::{
        CGameEvent, CPlayDisconnect, CPlayerAbilities, CPlayerInfoUpdate, CSetHealth,
        CSyncPlayerPosition, CSystemChatMessage, GameEvent, PlayerAction,
    },
    server::play::{
        SChatCommand, SChatMessage, SClickContainer, SClientInformationPlay, SConfirmTeleport,
        SInteract, SPlayPingRequest, SPlayerAction, SPlayerCommand, SPlayerPosition,
        SPlayerPositionRotation, SPlayerRotation, SSetCreativeSlot, SSetHeldItem, SSetPlayerGround,
        SSwingArm, SUseItem, SUseItemOn,
    },
    RawPacket, ServerPacket, VarInt,
};

use pumpkin_protocol::server::play::{SCloseContainer, SKeepAlive};
use pumpkin_world::item::ItemStack;

use crate::{
    client::{authentication::GameProfile, Client, PlayerConfig},
    server::Server,
    world::World,
};

use super::{living::LivingEntity, Entity};

/// Represents a Minecraft player entity.
///
/// A `Player` is a special type of entity that represents a human player connected to the server.
pub struct Player {
    /// The underlying living entity object that represents the player.
    pub living_entity: LivingEntity,
    /// The player's game profile information, including their username and UUID.
    pub gameprofile: GameProfile,
    /// The client connection associated with the player.
    pub client: Arc<Client>,
    /// The player's configuration settings. Changes when the Player changes their settings.
    pub config: Mutex<PlayerConfig>,
    /// The player's current gamemode (e.g., Survival, Creative, Adventure).
    pub gamemode: AtomicCell<GameMode>,
    /// The player's hunger level.
    pub food: AtomicI32,
    /// The player's food saturation level.
    pub food_saturation: AtomicCell<f32>,
    /// The player's inventory, containing items and equipment.
    pub inventory: Mutex<PlayerInventory>,
    /// The ID of the currently open container (if any).
    pub open_container: AtomicCell<Option<u64>>,
    /// The item currently being held by the player.
    pub carried_item: AtomicCell<Option<ItemStack>>,

    /// send `send_abilties_update` when changed
    /// The player's abilities and special powers.
    ///
    /// This field represents the various abilities that the player possesses, such as flight, invulnerability, and other special effects.
    ///
    /// **Note:** When the `abilities` field is updated, the server should send a `send_abilities_update` packet to the client to notify them of the changes.
    pub abilities: PlayerAbilities,
    /// The player's last known position.
    ///
    /// This field is used to calculate the player's movement delta for network synchronization and other purposes.
    pub last_position: AtomicCell<Vector3<f64>>,

    /// The current stage of the block the player is breaking.
    pub current_block_destroy_stage: AtomicU8,
    /// A counter for teleport IDs used to track pending teleports.
    pub teleport_id_count: AtomicI32,
    /// The pending teleport information, including the teleport ID and target location.
    pub awaiting_teleport: Mutex<Option<(VarInt, Vector3<f64>)>>,

    /// The coordinates of the chunk section the player is currently watching.
    pub watched_section: AtomicCell<Vector3<i32>>,
}

impl Player {
    pub fn new(
        client: Arc<Client>,
        world: Arc<World>,
        entity_id: EntityId,
        gamemode: GameMode,
    ) -> Self {
        let gameprofile = client.gameprofile.lock().clone().map_or_else(
            || {
                log::error!("No gameprofile?. Impossible");
                GameProfile {
                    id: uuid::Uuid::new_v4(),
                    name: "".to_string(),
                    properties: vec![],
                    profile_actions: None,
                }
            },
            |profile| profile,
        );
        let config = client.config.lock().clone().unwrap_or_default();
        Self {
            living_entity: LivingEntity::new(Entity::new(
                entity_id,
                world,
                EntityType::Player,
                1.62,
            )),
            config: Mutex::new(config),
            gameprofile,
            client,
            awaiting_teleport: Mutex::new(None),
            // TODO: Load this from previous instance
            food: AtomicI32::new(20),
            food_saturation: AtomicCell::new(20.0),
            current_block_destroy_stage: AtomicU8::new(0),
            inventory: Mutex::new(PlayerInventory::new()),
            open_container: AtomicCell::new(None),
            carried_item: AtomicCell::new(None),
            teleport_id_count: AtomicI32::new(0),
            abilities: PlayerAbilities::default(),
            gamemode: AtomicCell::new(gamemode),
            watched_section: AtomicCell::new(Vector3::new(0, 0, 0)),
            last_position: AtomicCell::new(Vector3::new(0.0, 0.0, 0.0)),
        }
    }

    /// Removes the Player out of the current World
    pub async fn remove(&self) {
        self.living_entity.entity.world.remove_player(self);
    }

    pub const fn entity_id(&self) -> EntityId {
        self.living_entity.entity.entity_id
    }

    /// Updates the current abilities the Player has
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

    pub fn teleport(&self, x: f64, y: f64, z: f64, yaw: f32, pitch: f32) {
        // this is the ultra special magic code used to create the teleport id
        // This returns the old value
        let i = self
            .teleport_id_count
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        if i + 2 == i32::MAX {
            self.teleport_id_count
                .store(0, std::sync::atomic::Ordering::Relaxed);
        }
        let teleport_id = i + 1;
        let entity = &self.living_entity.entity;
        entity.set_pos(x, y, z);
        entity.set_rotation(yaw, pitch);
        *self.awaiting_teleport.lock() = Some((teleport_id.into(), Vector3::new(x, y, z)));
        self.client.send_packet(&CSyncPlayerPosition::new(
            x,
            y,
            z,
            yaw,
            pitch,
            0,
            teleport_id.into(),
        ));
    }

    pub fn block_interaction_range(&self) -> f64 {
        if self.gamemode.load() == GameMode::Creative {
            5.0
        } else {
            4.5
        }
    }

    pub fn can_interact_with_block_at(&self, pos: &WorldPosition, additional_range: f64) -> bool {
        let d = self.block_interaction_range() + additional_range;
        let box_pos = BoundingBox::from_block(pos);
        let entity_pos = self.living_entity.entity.pos.load();
        let standing_eye_height = self.living_entity.entity.standing_eye_height;
        box_pos.squared_magnitude(Vector3 {
            x: entity_pos.x,
            y: entity_pos.y + standing_eye_height as f64,
            z: entity_pos.z,
        }) < d * d
    }

    /// Kicks the Client with a reason depending on the connection state
    pub fn kick(&self, reason: TextComponent) {
        assert!(!self
            .client
            .closed
            .load(std::sync::atomic::Ordering::Relaxed));

        self.client
            .try_send_packet(&CPlayDisconnect::new(&reason))
            .unwrap_or_else(|_| self.client.close());
        log::info!(
            "Kicked {} for {}",
            self.gameprofile.name,
            reason.to_pretty_console()
        );
        self.client.close()
    }

    pub fn set_health(&self, health: f32, food: i32, food_saturation: f32) {
        self.living_entity.set_health(health);
        self.food.store(food, std::sync::atomic::Ordering::Relaxed);
        self.food_saturation.store(food_saturation);
        self.client
            .send_packet(&CSetHealth::new(health, food.into(), food_saturation));
    }

    pub fn set_gamemode(&self, gamemode: GameMode) {
        // We could send the same gamemode without problems. But why waste bandwidth ?
        let current_gamemode = self.gamemode.load();
        assert!(
            current_gamemode != gamemode,
            "Setting the same gamemode as already is"
        );
        self.gamemode.store(gamemode);
        // So a little story time. I actually made an abilties_from_gamemode function. I looked at vanilla and they always send the abilties from the gamemode. But the funny thing actually is. That the client
        // does actually use the same method and set the abilties when receiving the CGameEvent gamemode packet. Just Mojang nonsense
        self.living_entity
            .entity
            .world
            .broadcast_packet_all(&CPlayerInfoUpdate::new(
                0x04,
                &[pumpkin_protocol::client::play::Player {
                    uuid: self.gameprofile.id,
                    actions: vec![PlayerAction::UpdateGameMode((gamemode as i32).into())],
                }],
            ));
        self.client.send_packet(&CGameEvent::new(
            GameEvent::ChangeGameMode,
            gamemode.to_f32().unwrap(),
        ));
    }

    pub fn send_system_message(&self, text: TextComponent) {
        self.client
            .send_packet(&CSystemChatMessage::new(text, false));
    }
}

impl Player {
    pub async fn process_packets(&self, server: &Arc<Server>) {
        let mut packets = self.client.client_packets_queue.lock();
        while let Some(mut packet) = packets.pop() {
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
        &self,
        server: &Arc<Server>,
        packet: &mut RawPacket,
    ) -> Result<(), DeserializerError> {
        let bytebuf = &mut packet.bytebuf;
        match packet.id.0 {
            SConfirmTeleport::PACKET_ID => {
                self.handle_confirm_teleport(SConfirmTeleport::read(bytebuf)?);
                Ok(())
            }
            SChatCommand::PACKET_ID => {
                self.handle_chat_command(server, SChatCommand::read(bytebuf)?);
                Ok(())
            }
            SPlayerPosition::PACKET_ID => {
                self.handle_position(SPlayerPosition::read(bytebuf)?).await;
                Ok(())
            }
            SPlayerPositionRotation::PACKET_ID => {
                self.handle_position_rotation(SPlayerPositionRotation::read(bytebuf)?)
                    .await;
                Ok(())
            }
            SPlayerRotation::PACKET_ID => {
                self.handle_rotation(SPlayerRotation::read(bytebuf)?).await;
                Ok(())
            }
            SSetPlayerGround::PACKET_ID => {
                self.handle_player_ground(SSetPlayerGround::read(bytebuf)?);
                Ok(())
            }
            SPlayerCommand::PACKET_ID => {
                self.handle_player_command(SPlayerCommand::read(bytebuf)?)
                    .await;
                Ok(())
            }
            SSwingArm::PACKET_ID => {
                self.handle_swing_arm(SSwingArm::read(bytebuf)?).await;
                Ok(())
            }
            SChatMessage::PACKET_ID => {
                self.handle_chat_message(SChatMessage::read(bytebuf)?).await;
                Ok(())
            }
            SClientInformationPlay::PACKET_ID => {
                self.handle_client_information_play(SClientInformationPlay::read(bytebuf)?);
                Ok(())
            }
            SInteract::PACKET_ID => {
                self.handle_interact(server, SInteract::read(bytebuf)?)
                    .await;
                Ok(())
            }
            SPlayerAction::PACKET_ID => {
                self.handle_player_action(SPlayerAction::read(bytebuf)?)
                    .await;
                Ok(())
            }
            SUseItemOn::PACKET_ID => {
                self.handle_use_item_on(SUseItemOn::read(bytebuf)?).await;
                Ok(())
            }
            SUseItem::PACKET_ID => {
                self.handle_use_item(SUseItem::read(bytebuf)?);
                Ok(())
            }
            SSetHeldItem::PACKET_ID => {
                self.handle_set_held_item(SSetHeldItem::read(bytebuf)?);
                Ok(())
            }
            SSetCreativeSlot::PACKET_ID => {
                self.handle_set_creative_slot(SSetCreativeSlot::read(bytebuf)?)
                    .unwrap();
                Ok(())
            }
            SPlayPingRequest::PACKET_ID => {
                self.handle_play_ping_request(SPlayPingRequest::read(bytebuf)?);
                Ok(())
            }
            SClickContainer::PACKET_ID => {
                // TODO
                // self.handle_click_container(server, SClickContainer::read(bytebuf)?)
                //     .await
                //     .unwrap();
                Ok(())
            }
            SCloseContainer::PACKET_ID => {
                self.handle_close_container(server, SCloseContainer::read(bytebuf)?);
                Ok(())
            }
            SKeepAlive::PACKET_ID => {
                self.client
                    .keep_alive_sender
                    .send(SKeepAlive::read(bytebuf)?.keep_alive_id)
                    .await
                    .unwrap();
                Ok(())
            }
            _ => {
                log::error!("Failed to handle player packet id {:#04x}", packet.id.0);
                Ok(())
            }
        }
    }
}

/// Represents a player's abilities and special powers.
///
/// This struct contains information about the player's current abilities, such as flight, invulnerability, and creative mode.
pub struct PlayerAbilities {
    /// Indicates whether the player is invulnerable to damage.
    pub invulnerable: bool,
    /// Indicates whether the player is currently flying.
    pub flying: bool,
    /// Indicates whether the player is allowed to fly (if enabled).
    pub allow_flying: bool,
    /// Indicates whether the player is in creative mode.
    pub creative: bool,
    /// The player's flying speed.
    pub fly_speed: f32,
    /// The field of view adjustment when the player is walking or sprinting.
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

/// Represents the player's dominant hand.
#[derive(FromPrimitive, Clone)]
pub enum Hand {
    /// The player's primary hand (usually the right hand).
    Main,
    /// The player's off-hand (usually the left hand).
    Off,
}

/// Represents the player's chat mode settings.
#[derive(FromPrimitive, Clone)]
pub enum ChatMode {
    /// Chat is enabled for the player.
    Enabled,
    /// The player should only see chat messages from commands
    CommandsOnly,
    /// All messages should be hidden
    Hidden,
}
