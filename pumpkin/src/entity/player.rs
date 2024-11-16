use std::{
    collections::{HashMap, VecDeque},
    sync::{
        atomic::{AtomicBool, AtomicI32, AtomicI64, AtomicU32, AtomicU8},
        Arc,
    },
    time::{Duration, Instant},
};

use crossbeam::atomic::AtomicCell;
use itertools::Itertools;
use num_derive::{FromPrimitive, ToPrimitive};
use pumpkin_config::ADVANCED_CONFIG;
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
use pumpkin_protocol::{
    bytebuf::packet_id::Packet,
    client::play::{
        CCombatDeath, CEntityStatus, CGameEvent, CHurtAnimation, CKeepAlive, CPlayDisconnect,
        CPlayerAbilities, CPlayerInfoUpdate, CRespawn, CSetHealth, CSpawnEntity,
        CSyncPlayerPosition, CSystemChatMessage, GameEvent, PlayerAction,
    },
    server::play::{
        SChatCommand, SChatMessage, SClientCommand, SClientInformationPlay, SClientTickEnd,
        SCommandSuggestion, SConfirmTeleport, SInteract, SPlayerAbilities, SPlayerAction,
        SPlayerCommand, SPlayerInput, SPlayerPosition, SPlayerPositionRotation, SPlayerRotation,
        SSetCreativeSlot, SSetHeldItem, SSetPlayerGround, SSwingArm, SUseItem, SUseItemOn,
        SUpdateSign
    },
    RawPacket, ServerPacket, SoundCategory, VarInt,
};
use tokio::sync::{Mutex, Notify};
use tokio::task::JoinHandle;

use pumpkin_protocol::server::play::{SClickContainer, SKeepAlive};
use pumpkin_world::{cylindrical_chunk_iterator::Cylindrical, item::ItemStack};

use super::Entity;
use crate::{
    client::{
        authentication::GameProfile,
        combat::{self, player_attack_sound, AttackType},
        Client, PlayerConfig,
    },
    server::Server,
    world::{player_chunker, World},
};
use crate::{error::PumpkinError, world::player_chunker::get_view_distance};

use super::living::LivingEntity;

pub struct ChunkHandleWrapper {
    handle: Option<JoinHandle<()>>,
    aborted: bool,
}

impl ChunkHandleWrapper {
    #[must_use]
    pub fn new(handle: JoinHandle<()>) -> Self {
        Self {
            handle: Some(handle),
            aborted: false,
        }
    }

    pub fn abort(&mut self) {
        self.aborted = true;
        if let Some(handle) = &self.handle {
            handle.abort();
        } else {
            log::error!("Trying to abort without a handle!");
        }
    }

    pub fn take_handle(&mut self) -> JoinHandle<()> {
        self.handle.take().unwrap()
    }

    #[must_use]
    pub fn aborted(&self) -> bool {
        self.aborted
    }
}

pub type PlayerPendingChunks =
    Arc<parking_lot::Mutex<HashMap<Vector2<i32>, VecDeque<ChunkHandleWrapper>>>>;

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
    pub abilities: Mutex<Abilities>,

    /// The current stage of the block the player is breaking.
    pub current_block_destroy_stage: AtomicU8,
    /// A counter for teleport IDs used to track pending teleports.
    pub teleport_id_count: AtomicI32,
    /// The pending teleport information, including the teleport ID and target location.
    pub awaiting_teleport: Mutex<Option<(VarInt, Vector3<f64>)>>,
    /// The coordinates of the chunk section the player is currently watching.
    pub watched_section: AtomicCell<Vector3<i32>>,
    /// Did we send a keep alive Packet and wait for the response?
    pub wait_for_keep_alive: AtomicBool,
    /// Whats the keep alive packet payload we send, The client should responde with the same id
    pub keep_alive_id: AtomicI64,
    /// Last time we send a keep alive
    pub last_keep_alive_time: AtomicCell<Instant>,
    /// Amount of ticks since last attack
    pub last_attacked_ticks: AtomicU32,

    //TODO: Is there a way to consolidate these two?
    //Need to lookup by chunk, but also would be need to contain all the stuff
    //In a PendingBatch struct. Is there a cheap way to map multiple keys to a single element?
    //
    /// Individual chunk tasks that this client is waiting for
    pub pending_chunks: PlayerPendingChunks,
    /// Chunk batches that this client is waiting for
    pub pending_chunk_batch: parking_lot::Mutex<HashMap<uuid::Uuid, JoinHandle<()>>>,

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
                log::error!("No gameprofile?. Impossible");
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
            living_entity: LivingEntity::new(Entity::new(
                entity_id,
                world,
                EntityType::Player,
                1.62,
                AtomicCell::new(BoundingBox::new_default(&bounding_box_size)),
                AtomicCell::new(bounding_box_size),
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
            abilities: Mutex::new(Abilities::default()),
            gamemode: AtomicCell::new(gamemode),
            watched_section: AtomicCell::new(Vector3::new(0, 0, 0)),
            wait_for_keep_alive: AtomicBool::new(false),
            keep_alive_id: AtomicI64::new(0),
            last_keep_alive_time: AtomicCell::new(std::time::Instant::now()),
            last_attacked_ticks: AtomicU32::new(0),
            pending_chunks: Arc::new(parking_lot::Mutex::new(HashMap::new())),
            pending_chunk_batch: parking_lot::Mutex::new(HashMap::new()),
            cancel_tasks: Notify::new(),
            // TODO: change this
            permission_lvl: PermissionLvl::Four,
        }
    }

    /// Removes the Player out of the current World
    #[allow(unused_variables)]
    pub async fn remove(&self) {
        let world = &self.living_entity.entity.world;
        // Abort pending chunks here too because we might clean up before chunk tasks are done
        self.abort_chunks("closed");

        self.cancel_tasks.notify_waiters();

        world.remove_player(self).await;

        let watched = self.watched_section.load();
        let view_distance = get_view_distance(self).await;
        let cylindrical = Cylindrical::new(Vector2::new(watched.x, watched.z), view_distance);

        // NOTE: This all must be synchronous to make sense! The chunks are handled asynhrously.
        // Non-async code is atomic to async code

        // Radial chunks are all of the chunks the player is theoretically viewing
        // Giving enough time, all of these chunks will be in memory
        let radial_chunks = cylindrical.all_chunks_within();

        log::debug!(
            "Removing player {} ({}), unwatching {} chunks",
            self.gameprofile.name,
            self.client.id,
            radial_chunks.len()
        );

        let (watched_chunks, to_await) = {
            let mut pending_chunks = self.pending_chunks.lock();

            // Don't try to clean chunks that dont exist yet
            // If they are still pending, we never sent the client the chunk,
            // And the watcher value is not set
            //
            // The chunk may or may not be in the cache at this point
            let watched_chunks = radial_chunks
                .iter()
                .filter(|chunk| !pending_chunks.contains_key(chunk))
                .copied()
                .collect::<Vec<_>>();

            // Mark all pending chunks to be cancelled
            // Cant use abort chunk because we use the lock for more
            pending_chunks.iter_mut().for_each(|(chunk, handles)| {
                handles.iter_mut().enumerate().for_each(|(count, handle)| {
                    if !handle.aborted() {
                        log::debug!("Aborting chunk {:?} ({}) (disconnect)", chunk, count);
                        handle.abort();
                    }
                });
            });

            let to_await = pending_chunks
                .iter_mut()
                .map(|(chunk, pending)| {
                    (
                        *chunk,
                        pending
                            .iter_mut()
                            .map(ChunkHandleWrapper::take_handle)
                            .collect_vec(),
                    )
                })
                .collect_vec();

            // Return chunks to stop watching and what to wait for
            (watched_chunks, to_await)
        };

        // Wait for individual chunks to finish after we cancel them
        for (chunk, awaitables) in to_await {
            for (count, handle) in awaitables.into_iter().enumerate() {
                #[cfg(debug_assertions)]
                log::debug!("Waiting for chunk {:?} ({})", chunk, count);
                let _ = handle.await;
            }
        }

        // Allow the batch jobs to properly cull stragglers before we do our clean up
        log::debug!("Collecting chunk batches...");
        let batches = {
            let mut chunk_batches = self.pending_chunk_batch.lock();
            let keys = chunk_batches.keys().copied().collect_vec();
            let handles = keys
                .iter()
                .filter_map(|batch_id| {
                    #[cfg(debug_assertions)]
                    log::debug!("Batch id: {}", batch_id);
                    chunk_batches.remove(batch_id)
                })
                .collect_vec();
            assert!(chunk_batches.is_empty());
            handles
        };

        log::debug!("Awaiting chunk batches ({})...", batches.len());

        for (count, batch) in batches.into_iter().enumerate() {
            #[cfg(debug_assertions)]
            log::debug!("Awaiting batch {}", count);
            let _ = batch.await;
            #[cfg(debug_assertions)]
            log::debug!("Done awaiting batch {}", count);
        }
        log::debug!("Done waiting for chunk batches");

        // Decrement value of watched chunks
        let chunks_to_clean = world.mark_chunks_as_not_watched(&watched_chunks);

        // Remove chunks with no watchers from the cache
        world.clean_chunks(&chunks_to_clean);

        // Remove left over entries from all possiblily loaded chunks
        world.clean_memory(&radial_chunks);

        log::debug!(
            "Removed player id {} ({}) ({} chunks remain cached)",
            self.gameprofile.name,
            self.client.id,
            self.living_entity.entity.world.get_cached_chunk_len()
        );

        //self.living_entity.entity.world.level.list_cached();
    }

    pub async fn attack(&self, victim: &Arc<Self>) {
        let world = &self.living_entity.entity.world;
        let victim_entity = &victim.living_entity.entity;
        let attacker_entity = &self.living_entity.entity;
        let config = &ADVANCED_CONFIG.pvp;

        let pos = victim_entity.pos.load();

        let attack_cooldown_progress = self.get_attack_cooldown_progress(0.5);
        self.last_attacked_ticks
            .store(0, std::sync::atomic::Ordering::Relaxed);

        // TODO: attack damage attribute and deal damage
        let mut damage = 1.0;
        if (config.protect_creative && victim.gamemode.load() == GameMode::Creative)
            || !victim.living_entity.check_damage(damage)
        {
            world
                .play_sound(
                    sound!("minecraft:entity.player.attack.nodamage"),
                    SoundCategory::Players,
                    &pos,
                )
                .await;
            return;
        }

        world
            .play_sound(
                sound!("minecraft:entity.player.hurt"),
                SoundCategory::Players,
                &pos,
            )
            .await;

        let attack_type = AttackType::new(self, attack_cooldown_progress).await;

        player_attack_sound(&pos, world, attack_type).await;

        if matches!(attack_type, AttackType::Critical) {
            damage *= 1.5;
        }

        victim.living_entity.damage(damage).await;

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

    pub fn get_attack_cooldown_progress(&self, base_time: f32) -> f32 {
        #[allow(clippy::cast_precision_loss)]
        let x = self
            .last_attacked_ticks
            .load(std::sync::atomic::Ordering::Acquire) as f32
            + base_time;
        // TODO attack speed attribute
        let attack_speed = 4.0;
        let progress_per_tick = 1.0 / attack_speed * 20.0;

        let progress = x / progress_per_tick;
        progress.clamp(0.0, 1.0)
    }

    pub const fn entity_id(&self) -> EntityId {
        self.living_entity.entity.entity_id
    }

    /// Updates the current abilities the Player has
    pub async fn send_abilties_update(&self) {
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

    pub async fn respawn(self: &Arc<Self>, alive: bool) {
        let last_pos = self.living_entity.last_pos.load();
        let death_location = WorldPosition(Vector3::new(
            last_pos.x.round() as i32,
            last_pos.y.round() as i32,
            last_pos.z.round() as i32,
        ));

        let data_kept = u8::from(alive);

        self.client
            .send_packet(&CRespawn::new(
                0.into(),
                "minecraft:overworld",
                0, // seed
                self.gamemode.load() as u8,
                self.gamemode.load() as i8,
                false,
                false,
                Some(("minecraft:overworld", death_location)),
                0.into(),
                0.into(),
                data_kept,
            ))
            .await;

        log::debug!("Sending player abilities to {}", self.gameprofile.name);
        self.send_abilties_update().await;

        self.send_permission_lvl_update().await;

        let world = &self.living_entity.entity.world;

        // teleport
        let x = 10.0;
        let z = 10.0;
        let top = world.get_top_block(Vector2::new(x as i32, z as i32)).await;
        let position = Vector3::new(x, f64::from(top), z);
        let yaw = 10.0;
        let pitch = 10.0;

        log::debug!("Sending player teleport to {}", self.gameprofile.name);
        self.teleport(position, yaw, pitch).await;

        self.living_entity.last_pos.store(position);

        // TODO: difficulty, exp bar, status effect

        let world = &self.living_entity.entity.world;
        world
            .worldborder
            .lock()
            .await
            .init_client(&self.client)
            .await;

        // TODO: world spawn (compass stuff)

        self.client
            .send_packet(&CGameEvent::new(GameEvent::StartWaitingChunks, 0.0))
            .await;

        let entity = &self.living_entity.entity;
        world
            .broadcast_packet_except(
                &[self.gameprofile.id],
                // TODO: add velo
                &CSpawnEntity::new(
                    entity.entity_id.into(),
                    self.gameprofile.id,
                    (EntityType::Player as i32).into(),
                    position.x,
                    position.y,
                    position.z,
                    pitch,
                    yaw,
                    yaw,
                    0.into(),
                    0.0,
                    0.0,
                    0.0,
                ),
            )
            .await;

        player_chunker::player_join(world, self.clone()).await;

        // update commands

        self.set_health(20.0, 20, 20.0).await;
    }

    /// yaw and pitch in degrees
    pub async fn teleport(&self, position: Vector3<f64>, yaw: f32, pitch: f32) {
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
        self.living_entity
            .set_pos(position.x, position.y, position.z);
        let entity = &self.living_entity.entity;
        entity.set_rotation(yaw, pitch);
        *self.awaiting_teleport.lock().await = Some((teleport_id.into(), position));
        self.client
            .send_packet(&CSyncPlayerPosition::new(
                teleport_id.into(),
                position,
                Vector3::new(0.0, 0.0, 0.0),
                yaw,
                pitch,
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
        let current_gamemode = self.gamemode.load();
        assert!(
            current_gamemode != gamemode,
            "Setting the same gamemode as already is"
        );
        self.gamemode.store(gamemode);
        self.abilities.lock().await.flying = false;
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

    pub async fn send_system_message<'a>(&self, text: &TextComponent<'a>) {
        self.client
            .send_packet(&CSystemChatMessage::new(text, false))
            .await;
    }

    pub fn abort_chunks(&self, reason: &str) {
        let mut pending_chunks = self.pending_chunks.lock();
        pending_chunks.iter_mut().for_each(|(chunk, handles)| {
            handles.iter_mut().enumerate().for_each(|(count, handle)| {
                if !handle.aborted() {
                    log::debug!("Aborting chunk {:?} ({}) ({})", chunk, count, reason);
                    handle.abort();
                }
            });
        });
    }
}

impl Player {
    pub async fn process_packets(self: &Arc<Self>, server: &Arc<Server>) {
        let mut packets = self.client.client_packets_queue.lock().await;
        while let Some(mut packet) = packets.pop_back() {
            #[cfg(debug_assertions)]
            let inst = std::time::Instant::now();
            tokio::select! {
                () = self.await_cancel() => {
                    log::debug!("Canceling player packet processing");
                    return;
                },
                packet_result = self.handle_play_packet(server, &mut packet) => {
                    #[cfg(debug_assertions)]
                    log::debug!("Handled play packet in {:?}", inst.elapsed());
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
                self.handle_player_action(SPlayerAction::read(bytebuf)?)
                    .await;
            }
            SPlayerCommand::PACKET_ID => {
                self.handle_player_command(SPlayerCommand::read(bytebuf)?)
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
                self.handle_use_item_on(SUseItemOn::read(bytebuf)?).await;
            }
            SUseItem::PACKET_ID => self.handle_use_item(&SUseItem::read(bytebuf)?),
            SUpdateSign::PACKET_ID => self.handle_update_sign(&SUpdateSign::read(bytebuf)?),
            SCommandSuggestion::PACKET_ID => {
                self.handle_command_suggestion(SCommandSuggestion::read(bytebuf)?, server)
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

/// the player's permission level
#[derive(FromPrimitive, ToPrimitive, Clone, Copy)]
#[repr(i8)]
pub enum PermissionLvl {
    Zero = 0,
    One = 1,
    Two = 2,
    Three = 3,
    Four = 4,
}
