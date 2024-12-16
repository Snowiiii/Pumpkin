use std::{
    sync::{
        atomic::{AtomicBool, AtomicI32, AtomicI64, AtomicU32, AtomicU8},
        Arc,
    },
    time::{Duration, Instant},
};

use crossbeam::atomic::AtomicCell;
use num_derive::{FromPrimitive, ToPrimitive};
use num_traits::Pow;
use pumpkin_config::{ADVANCED_CONFIG, BASIC_CONFIG};
use pumpkin_core::{
    math::{
        boundingbox::{BoundingBox, BoundingBoxSize},
        position::WorldPosition,
        vector2::Vector2,
        vector3::Vector3,
    },
    text::TextComponent,
    GameMode,
};
use pumpkin_entity::{entity_type::EntityType, EntityId};
use pumpkin_inventory::player::PlayerInventory;
use pumpkin_macros::sound;
use pumpkin_protocol::client::play::CUpdateTime;
use pumpkin_protocol::server::play::{
    SCloseContainer, SCookieResponse as SPCookieResponse, SPlayPingRequest,
};
use pumpkin_protocol::{
    bytebuf::packet_id::Packet,
    client::play::{
        CCombatDeath, CEntityStatus, CGameEvent, CHurtAnimation, CKeepAlive, CPlayDisconnect,
        CPlayerAbilities, CPlayerInfoUpdate, CPlayerPosition, CSetHealth, CSystemChatMessage,
        GameEvent, PlayerAction,
    },
    server::play::{
        SChatCommand, SChatMessage, SClientCommand, SClientInformationPlay, SClientTickEnd,
        SCommandSuggestion, SConfirmTeleport, SInteract, SPlayerAbilities, SPlayerAction,
        SPlayerCommand, SPlayerInput, SPlayerPosition, SPlayerPositionRotation, SPlayerRotation,
        SSetCreativeSlot, SSetHeldItem, SSetPlayerGround, SSwingArm, SUseItem, SUseItemOn,
    },
    RawPacket, ServerPacket, SoundCategory, VarInt,
};
use pumpkin_protocol::{
    client::play::{CSetEntityMetadata, Metadata},
    server::play::{SClickContainer, SKeepAlive},
};
use pumpkin_world::{
    cylindrical_chunk_iterator::Cylindrical,
    item::{item_registry::get_item_by_id, ItemStack},
};
use tokio::sync::{Mutex, Notify};

use super::Entity;
use crate::error::PumpkinError;
use crate::{
    client::{
        authentication::GameProfile,
        combat::{self, player_attack_sound, AttackType},
        Client, PlayerConfig,
    },
    server::Server,
    world::World,
};

use super::living::LivingEntity;

/// Represents a Minecraft player entity.
///
/// A `Player` is a special type of entity that represents a human player connected to the server.
pub struct Player {
    /// The underlying living entity object that represents the player.
    pub living_entity: LivingEntity<PlayerInventory>,
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
    /// The ID of the currently open container (if any).
    pub open_container: AtomicCell<Option<u64>>,
    /// The item currently being held by the player.
    pub carried_item: AtomicCell<Option<ItemStack>>,

    /// send `send_abilities_update` when changed
    /// The player's abilities and special powers.
    ///
    /// This field represents the various abilities that the player possesses, such as flight, invulnerability, and other special effects.
    ///
    /// **Note:** When the `abilities` field is updated, the server should send a `send_abilities_update` packet to the client to notify them of the changes.
    pub abilities: Mutex<Abilities>,

    /// The current stage of the block the player is breaking.
    pub current_block_destroy_stage: AtomicU8,
    /// A counter for teleport IDs used to track pending teleports.
    pub teleport_id_count: AtomicI32,
    /// The pending teleport information, including the teleport ID and target location.
    pub awaiting_teleport: Mutex<Option<(VarInt, Vector3<f64>)>>,
    /// The coordinates of the chunk section the player is currently watching.
    pub watched_section: AtomicCell<Cylindrical>,
    /// Did we send a keep alive Packet and wait for the response?
    pub wait_for_keep_alive: AtomicBool,
    /// Whats the keep alive packet payload we send, The client should responde with the same id
    pub keep_alive_id: AtomicI64,
    /// Last time we send a keep alive
    pub last_keep_alive_time: AtomicCell<Instant>,
    /// Amount of ticks since last attack
    pub last_attacked_ticks: AtomicU32,

    /// Tell tasks to stop if we are closing
    cancel_tasks: Notify,

    /// the players op permission level
    permission_lvl: PermissionLvl,
}

impl Player {
    pub async fn new(
        client: Arc<Client>,
        world: Arc<World>,
        entity_id: EntityId,
        gamemode: GameMode,
    ) -> Self {
        let gameprofile = client.gameprofile.lock().await.clone().map_or_else(
            || {
                log::error!("Client {} has no game profile!", client.id);
                GameProfile {
                    id: uuid::Uuid::new_v4(),
                    name: String::new(),
                    properties: vec![],
                    profile_actions: None,
                }
            },
            |profile| profile,
        );
        let config = client.config.lock().await.clone().unwrap_or_default();
        let bounding_box_size = BoundingBoxSize {
            width: 0.6,
            height: 1.8,
        };

        Self {
            living_entity: LivingEntity::new_with_container(
                Entity::new(
                    entity_id,
                    world,
                    EntityType::Player,
                    1.62,
                    AtomicCell::new(BoundingBox::new_default(&bounding_box_size)),
                    AtomicCell::new(bounding_box_size),
                ),
                PlayerInventory::new(),
            ),
            config: Mutex::new(config),
            gameprofile,
            client,
            awaiting_teleport: Mutex::new(None),
            // TODO: Load this from previous instance
            food: AtomicI32::new(20),
            food_saturation: AtomicCell::new(20.0),
            current_block_destroy_stage: AtomicU8::new(0),
            open_container: AtomicCell::new(None),
            carried_item: AtomicCell::new(None),
            teleport_id_count: AtomicI32::new(0),
            abilities: Mutex::new(Abilities::default()),
            gamemode: AtomicCell::new(gamemode),
            // We want this to be an impossible watched section so that `player_chunker::update_position`
            // will mark chunks as watched for a new join rather than a respawn
            // (We left shift by one so we can search around that chunk)
            watched_section: AtomicCell::new(Cylindrical::new(
                Vector2::new(i32::MAX >> 1, i32::MAX >> 1),
                0,
            )),
            wait_for_keep_alive: AtomicBool::new(false),
            keep_alive_id: AtomicI64::new(0),
            last_keep_alive_time: AtomicCell::new(std::time::Instant::now()),
            last_attacked_ticks: AtomicU32::new(0),
            cancel_tasks: Notify::new(),
            // TODO: change this
            permission_lvl: PermissionLvl::Four,
        }
    }

    pub fn inventory(&self) -> &Mutex<PlayerInventory> {
        self.living_entity
            .inventory
            .as_ref()
            .expect("Player always has inventory")
    }

    /// Removes the Player out of the current World
    #[allow(unused_variables)]
    pub async fn remove(&self) {
        let world = self.world();
        self.cancel_tasks.notify_waiters();

        world.remove_player(self).await;

        let cylindrical = self.watched_section.load();

        // Radial chunks are all of the chunks the player is theoretically viewing
        // Giving enough time, all of these chunks will be in memory
        let radial_chunks = cylindrical.all_chunks_within();

        log::debug!(
            "Removing player {} ({}), unwatching {} chunks",
            self.gameprofile.name,
            self.client.id,
            radial_chunks.len()
        );

        // Decrement value of watched chunks
        let chunks_to_clean = world.mark_chunks_as_not_watched(&radial_chunks);

        // Remove chunks with no watchers from the cache
        world.clean_chunks(&chunks_to_clean);
        // Remove left over entries from all possiblily loaded chunks
        world.clean_memory(&radial_chunks);

        log::debug!(
            "Removed player id {} ({}) ({} chunks remain cached)",
            self.gameprofile.name,
            self.client.id,
            self.world().get_cached_chunk_len()
        );

        //self.world().level.list_cached();
    }

    pub async fn attack(&self, victim: &Arc<Self>) {
        let world = self.world();
        let victim_entity = &victim.living_entity.entity;
        let attacker_entity = &self.living_entity.entity;
        let config = &ADVANCED_CONFIG.pvp;

        let inventory = self.inventory().lock().await;
        let item_slot = inventory.held_item();

        let base_damage = 1.0;
        let base_attack_speed = 4.0;

        let mut damage_multiplier = 1.0;
        let mut add_damage = 0.0;
        let mut add_speed = 0.0;

        // get attack damage
        if let Some(item_stack) = item_slot {
            if let Some(item) = get_item_by_id(item_stack.item_id) {
                // TODO: this should be cached in memory
                if let Some(modifiers) = &item.components.attribute_modifiers {
                    for item_mod in &modifiers.modifiers {
                        if item_mod.operation == "add_value" {
                            if item_mod.id == "minecraft:base_attack_damage" {
                                add_damage = item_mod.amount;
                            }
                            if item_mod.id == "minecraft:base_attack_speed" {
                                add_speed = item_mod.amount;
                            }
                        }
                    }
                }
            }
        }
        drop(inventory);

        let attack_speed = base_attack_speed + add_speed;

        let attack_cooldown_progress = self.get_attack_cooldown_progress(0.5, attack_speed);
        self.last_attacked_ticks
            .store(0, std::sync::atomic::Ordering::Relaxed);

        // only reduce attack damage if in cooldown
        // TODO: Enchantments are reduced same way just without the square
        if attack_cooldown_progress < 1.0 {
            damage_multiplier = 0.2 + attack_cooldown_progress.pow(2) * 0.8;
        }
        // modify added damage based on multiplier
        let mut damage = base_damage + add_damage * damage_multiplier;

        let pos = victim_entity.pos.load();

        if (config.protect_creative && victim.gamemode.load() == GameMode::Creative)
            || !victim.living_entity.check_damage(damage as f32)
        {
            world
                .play_sound(
                    sound!("entity.player.attack.nodamage"),
                    SoundCategory::Players,
                    &pos,
                )
                .await;
            return;
        }

        world
            .play_sound(sound!("entity.player.hurt"), SoundCategory::Players, &pos)
            .await;

        let attack_type = AttackType::new(self, attack_cooldown_progress as f32).await;

        player_attack_sound(&pos, world, attack_type).await;

        if matches!(attack_type, AttackType::Critical) {
            damage *= 1.5;
        }

        victim
            .living_entity
            .damage(damage as f32, 34) // PlayerAttack
            .await;

        let mut knockback_strength = 1.0;
        match attack_type {
            AttackType::Knockback => knockback_strength += 1.0,
            AttackType::Sweeping => {
                combat::spawn_sweep_particle(attacker_entity, world, &pos).await;
            }
            _ => {}
        };

        if config.knockback {
            combat::handle_knockback(attacker_entity, victim, victim_entity, knockback_strength)
                .await;
        }

        if config.hurt_animation {
            let entity_id = VarInt(victim_entity.entity_id);
            world
                .broadcast_packet_all(&CHurtAnimation::new(&entity_id, attacker_entity.yaw.load()))
                .await;
        }

        if config.swing {}
    }

    pub async fn await_cancel(&self) {
        self.cancel_tasks.notified().await;
    }

    pub async fn tick(&self) {
        if self
            .client
            .closed
            .load(std::sync::atomic::Ordering::Relaxed)
        {
            return;
        }
        let now = Instant::now();
        self.last_attacked_ticks
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);

        self.living_entity.tick();

        if now.duration_since(self.last_keep_alive_time.load()) >= Duration::from_secs(15) {
            // We never got a response from our last keep alive we send
            if self
                .wait_for_keep_alive
                .load(std::sync::atomic::Ordering::Relaxed)
            {
                self.kick(TextComponent::text("Timeout")).await;
                return;
            }
            self.wait_for_keep_alive
                .store(true, std::sync::atomic::Ordering::Relaxed);
            self.last_keep_alive_time.store(now);
            let id = now.elapsed().as_millis() as i64;
            self.keep_alive_id
                .store(id, std::sync::atomic::Ordering::Relaxed);
            self.client.send_packet(&CKeepAlive::new(id)).await;
        }
    }

    pub fn get_attack_cooldown_progress(&self, base_time: f64, attack_speed: f64) -> f64 {
        #[allow(clippy::cast_precision_loss)]
        let x = f64::from(
            self.last_attacked_ticks
                .load(std::sync::atomic::Ordering::Acquire),
        ) + base_time;

        let progress_per_tick = f64::from(BASIC_CONFIG.tps) / attack_speed;
        let progress = x / progress_per_tick;
        progress.clamp(0.0, 1.0)
    }

    pub const fn entity_id(&self) -> EntityId {
        self.living_entity.entity.entity_id
    }

    pub const fn world(&self) -> &Arc<World> {
        &self.living_entity.entity.world
    }

    /// Updates the current abilities the Player has
    pub async fn send_abilities_update(&self) {
        let mut b = 0i8;
        let abilities = &self.abilities.lock().await;

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
        self.client
            .send_packet(&CPlayerAbilities::new(
                b,
                abilities.fly_speed,
                abilities.walk_speed_fov,
            ))
            .await;
    }

    /// syncs the players permission level with the client
    pub async fn send_permission_lvl_update(&self) {
        self.client
            .send_packet(&CEntityStatus::new(
                self.entity_id(),
                24 + self.permission_lvl as i8,
            ))
            .await;
    }

    /// sets the players permission level and syncs it with the client
    pub async fn set_permission_lvl(&mut self, lvl: PermissionLvl) {
        self.permission_lvl = lvl;
        self.send_permission_lvl_update().await;
    }

    /// get the players permission level
    pub fn permission_lvl(&self) -> PermissionLvl {
        self.permission_lvl
    }

    /// Sends the world time to just the player.
    pub async fn send_time(&self, world: &World) {
        let l_world = world.level_time.lock().await;
        self.client
            .send_packet(&CUpdateTime::new(
                l_world.world_age,
                l_world.time_of_day,
                true,
            ))
            .await;
    }

    /// Yaw and Pitch in degrees
    /// Rarly used, For example when waking up player from bed or first time spawn. Otherwise entity teleport is used
    /// Player should respond with the `SConfirmTeleport` packet
    pub async fn request_teleport(&self, position: Vector3<f64>, yaw: f32, pitch: f32) {
        // this is the ultra special magic code used to create the teleport id
        // This returns the old value
        // This operation wraps around on overflow.
        let i = self
            .teleport_id_count
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let teleport_id = i + 1;
        self.living_entity.set_pos(position);
        let entity = &self.living_entity.entity;
        entity.set_rotation(yaw, pitch);
        *self.awaiting_teleport.lock().await = Some((teleport_id.into(), position));
        self.client
            .send_packet(&CPlayerPosition::new(
                teleport_id.into(),
                position,
                Vector3::new(0.0, 0.0, 0.0),
                yaw,
                pitch,
                // TODO
                &[],
            ))
            .await;
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
            y: entity_pos.y + f64::from(standing_eye_height),
            z: entity_pos.z,
        }) < d * d
    }

    /// Kicks the Client with a reason depending on the connection state
    pub async fn kick<'a>(&self, reason: TextComponent<'a>) {
        if self
            .client
            .closed
            .load(std::sync::atomic::Ordering::Relaxed)
        {
            log::debug!(
                "Tried to kick id {} but connection is closed!",
                self.client.id
            );
            return;
        }

        self.client
            .try_send_packet(&CPlayDisconnect::new(&reason))
            .await
            .unwrap_or_else(|_| self.client.close());
        log::info!(
            "Kicked Player {} ({}) for {}",
            self.gameprofile.name,
            self.client.id,
            reason.to_pretty_console()
        );
        self.client.close();
    }

    pub async fn set_health(&self, health: f32, food: i32, food_saturation: f32) {
        self.living_entity.set_health(health).await;
        self.food.store(food, std::sync::atomic::Ordering::Relaxed);
        self.food_saturation.store(food_saturation);
        self.client
            .send_packet(&CSetHealth::new(health, food.into(), food_saturation))
            .await;
    }

    pub async fn kill(&self) {
        self.living_entity.kill().await;

        self.client
            .send_packet(&CCombatDeath::new(
                self.entity_id().into(),
                TextComponent::text("noob"),
            ))
            .await;
    }

    pub async fn set_gamemode(&self, gamemode: GameMode) {
        // We could send the same gamemode without problems. But why waste bandwidth ?
        assert_ne!(
            self.gamemode.load(),
            gamemode,
            "Setting the same gamemode as already is"
        );
        self.gamemode.store(gamemode);
        // The client is using the same method for setting abilities when receiving the CGameEvent ChangeGameMode packet.
        // So we can just update the abilities without sending them.
        {
            // use another scope so we instantly unlock abilities
            let mut abilities = self.abilities.lock().await;
            match gamemode {
                GameMode::Undefined | GameMode::Survival | GameMode::Adventure => {
                    abilities.flying = false;
                    abilities.allow_flying = false;
                    abilities.creative = false;
                    abilities.invulnerable = false;
                }
                GameMode::Creative => {
                    abilities.allow_flying = true;
                    abilities.creative = true;
                    abilities.invulnerable = true;
                }
                GameMode::Spectator => {
                    abilities.flying = true;
                    abilities.allow_flying = true;
                    abilities.creative = false;
                    abilities.invulnerable = true;
                }
            }
        }
        self.living_entity
            .entity
            .world
            .broadcast_packet_all(&CPlayerInfoUpdate::new(
                0x04,
                &[pumpkin_protocol::client::play::Player {
                    uuid: self.gameprofile.id,
                    actions: vec![PlayerAction::UpdateGameMode((gamemode as i32).into())],
                }],
            ))
            .await;
        #[allow(clippy::cast_precision_loss)]
        self.client
            .send_packet(&CGameEvent::new(
                GameEvent::ChangeGameMode,
                gamemode as i32 as f32,
            ))
            .await;
    }

    /// Send skin layers and used hand to all players
    pub async fn update_client_information(&self) {
        let config = self.config.lock().await;
        let world = self.world();
        world
            .broadcast_packet_all(&CSetEntityMetadata::new(
                self.entity_id().into(),
                Metadata::new(17, 0.into(), config.skin_parts),
            ))
            .await;
        world
            .broadcast_packet_all(&CSetEntityMetadata::new(
                self.entity_id().into(),
                Metadata::new(18, 0.into(), config.main_hand as u8),
            ))
            .await;
    }

    pub async fn send_system_message<'a>(&self, text: &TextComponent<'a>) {
        self.client
            .send_packet(&CSystemChatMessage::new(text, false))
            .await;
    }
}

impl Player {
    pub async fn process_packets(self: &Arc<Self>, server: &Arc<Server>) {
        let mut packets = self.client.client_packets_queue.lock().await;
        while let Some(mut packet) = packets.pop_back() {
            tokio::select! {
                () = self.await_cancel() => {
                    log::debug!("Canceling player packet processing");
                    return;
                },
                packet_result = self.handle_play_packet(server, &mut packet) => {
                    match packet_result {
                        Ok(()) => {}
                        Err(e) => {
                            if e.is_kick() {
                                if let Some(kick_reason) = e.client_kick_reason() {
                                    self.kick(TextComponent::text(&kick_reason)).await;
                                } else {
                                    self.kick(TextComponent::text(&format!(
                                        "Error while reading incoming packet {e}"
                                    )))
                                    .await;
                                }
                            }
                            e.log();
                        }
                    };
                }
            }
        }
    }

    pub async fn handle_play_packet(
        self: &Arc<Self>,
        server: &Arc<Server>,
        packet: &mut RawPacket,
    ) -> Result<(), Box<dyn PumpkinError>> {
        let bytebuf = &mut packet.bytebuf;
        match packet.id.0 {
            SConfirmTeleport::PACKET_ID => {
                self.handle_confirm_teleport(SConfirmTeleport::read(bytebuf)?)
                    .await;
            }
            SChatCommand::PACKET_ID => {
                self.handle_chat_command(server, SChatCommand::read(bytebuf)?)
                    .await;
            }
            SChatMessage::PACKET_ID => {
                self.handle_chat_message(SChatMessage::read(bytebuf)?).await;
            }
            SClientInformationPlay::PACKET_ID => {
                self.handle_client_information(SClientInformationPlay::read(bytebuf)?)
                    .await;
            }
            SClientCommand::PACKET_ID => {
                self.handle_client_status(SClientCommand::read(bytebuf)?)
                    .await;
            }
            SPlayerInput::PACKET_ID => {
                // TODO
            }
            SInteract::PACKET_ID => {
                self.handle_interact(SInteract::read(bytebuf)?).await;
            }
            SKeepAlive::PACKET_ID => {
                self.handle_keep_alive(SKeepAlive::read(bytebuf)?).await;
            }
            SClientTickEnd::PACKET_ID => {
                // TODO
            }
            SPlayerPosition::PACKET_ID => {
                self.handle_position(SPlayerPosition::read(bytebuf)?).await;
            }
            SPlayerPositionRotation::PACKET_ID => {
                self.handle_position_rotation(SPlayerPositionRotation::read(bytebuf)?)
                    .await;
            }
            SPlayerRotation::PACKET_ID => {
                self.handle_rotation(SPlayerRotation::read(bytebuf)?).await;
            }
            SSetPlayerGround::PACKET_ID => {
                self.handle_player_ground(&SSetPlayerGround::read(bytebuf)?);
            }
            SPlayerAbilities::PACKET_ID => {
                self.handle_player_abilities(SPlayerAbilities::read(bytebuf)?)
                    .await;
            }
            SPlayerAction::PACKET_ID => {
                self.handle_player_action(SPlayerAction::read(bytebuf)?, server)
                    .await;
            }
            SPlayerCommand::PACKET_ID => {
                self.handle_player_command(SPlayerCommand::read(bytebuf)?)
                    .await;
            }
            SPlayPingRequest::PACKET_ID => {
                self.handle_play_ping_request(SPlayPingRequest::read(bytebuf)?)
                    .await;
            }
            SClickContainer::PACKET_ID => {
                self.handle_click_container(server, SClickContainer::read(bytebuf)?)
                    .await?;
            }
            SSetHeldItem::PACKET_ID => {
                self.handle_set_held_item(SSetHeldItem::read(bytebuf)?)
                    .await;
            }
            SSetCreativeSlot::PACKET_ID => {
                self.handle_set_creative_slot(SSetCreativeSlot::read(bytebuf)?)
                    .await?;
            }
            SSwingArm::PACKET_ID => {
                self.handle_swing_arm(SSwingArm::read(bytebuf)?).await;
            }
            SUseItemOn::PACKET_ID => {
                self.handle_use_item_on(SUseItemOn::read(bytebuf)?, server)
                    .await?;
            }
            SUseItem::PACKET_ID => self.handle_use_item(&SUseItem::read(bytebuf)?),
            SCommandSuggestion::PACKET_ID => {
                self.handle_command_suggestion(SCommandSuggestion::read(bytebuf)?, server)
                    .await;
            }
            SPCookieResponse::PACKET_ID => {
                self.handle_cookie_response(SPCookieResponse::read(bytebuf)?);
            }
            SCloseContainer::PACKET_ID => {
                self.handle_close_container(server, SCloseContainer::read(bytebuf)?)
                    .await;
            }
            _ => {
                log::warn!("Failed to handle player packet id {}", packet.id.0);
                // TODO: We give an error if all play packets are implemented
                //  return Err(Box::new(DeserializerError::UnknownPacket));
            }
        };
        Ok(())
    }
}

/// Represents a player's abilities and special powers.
///
/// This struct contains information about the player's current abilities, such as flight, invulnerability, and creative mode.
pub struct Abilities {
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

impl Default for Abilities {
    fn default() -> Self {
        Self {
            invulnerable: false,
            flying: false,
            allow_flying: false,
            creative: false,
            fly_speed: 0.4,
            walk_speed_fov: 0.1,
        }
    }
}

/// Represents the player's dominant hand.
#[derive(Debug, FromPrimitive, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Hand {
    /// Usually the player's off-hand.
    Left,
    /// Usually the player's primary hand.
    Right,
}

/// Represents the player's chat mode settings.
#[derive(Debug, FromPrimitive, Clone)]
pub enum ChatMode {
    /// Chat is enabled for the player.
    Enabled,
    /// The player should only see chat messages from commands
    CommandsOnly,
    /// All messages should be hidden
    Hidden,
}

/// the player's permission level
#[derive(Debug, FromPrimitive, ToPrimitive, Clone, Copy)]
#[repr(i8)]
pub enum PermissionLvl {
    /// `normal`: Player can use basic commands.
    Zero = 0,
    /// `moderator`: Player can bypass spawn protection.
    One = 1,
    /// `gamemaster`: Player or executor can use more commands and player can use command blocks.
    Two = 2,
    /// `admin`: Player or executor can use commands related to multiplayer management.
    Three = 3,
    /// `owner`: Player or executor can use all of the commands, including commands related to server management.
    Four = 4,
}
