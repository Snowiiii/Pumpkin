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
use num_derive::FromPrimitive;
use num_traits::{FromPrimitive, ToPrimitive};
use pumpkin_core::{
    math::{boundingbox::BoundingBox, position::WorldPosition, vector2::Vector2, vector3::Vector3},
    text::TextComponent,
    GameMode,
};
use pumpkin_entity::{entity_type::EntityType, EntityId};
use pumpkin_inventory::player::PlayerInventory;
use pumpkin_protocol::{
    bytebuf::DeserializerError,
    client::play::{
        CGameEvent, CKeepAlive, CPlayDisconnect, CPlayerAbilities, CPlayerInfoUpdate, CSetHealth,
        CSyncPlayerPosition, CSystemChatMessage, GameEvent, PlayerAction,
    },
    server::play::{
        SChatCommand, SChatMessage, SClientInformationPlay, SConfirmTeleport, SInteract,
        SPlayerAbilities, SPlayerAction, SPlayerCommand, SPlayerPosition, SPlayerPositionRotation,
        SPlayerRotation, SSetCreativeSlot, SSetHeldItem, SSetPlayerGround, SSwingArm, SUseItem,
        SUseItemOn, ServerboundPlayPackets,
    },
    RawPacket, ServerPacket, VarInt,
};
use tokio::sync::Mutex;
use tokio::task::JoinHandle;

use pumpkin_protocol::server::play::SKeepAlive;
use pumpkin_world::{cylindrical_chunk_iterator::Cylindrical, item::ItemStack};

use super::Entity;
use crate::{
    client::{authentication::GameProfile, Client, PlayerConfig, TaskId},
    server::Server,
    world::World,
};
use crate::{error::PumpkinError, world::player_chunker::get_view_distance};

use super::living::LivingEntity;

pub type PlayerPendingChunks = Arc<parking_lot::Mutex<HashMap<Vector2<i32>, VecDeque<TaskId>>>>;

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
            abilities: Mutex::new(Abilities::default()),
            gamemode: AtomicCell::new(gamemode),
            watched_section: AtomicCell::new(Vector3::new(0, 0, 0)),
            last_position: AtomicCell::new(Vector3::new(0.0, 0.0, 0.0)),
            wait_for_keep_alive: AtomicBool::new(false),
            keep_alive_id: AtomicI64::new(0),
            last_keep_alive_time: AtomicCell::new(std::time::Instant::now()),
            last_attacked_ticks: AtomicU32::new(0),
            pending_chunks: Arc::new(parking_lot::Mutex::new(HashMap::new())),
            pending_chunk_batch: parking_lot::Mutex::new(HashMap::new()),
        }
    }

    /// Removes the Player out of the current World
    #[allow(unused_variables)]
    pub async fn remove(&self) {
        let world = &self.living_entity.entity.world;

        world.remove_player(self).await;

        let watched = self.watched_section.load();
        let view_distance = i32::from(get_view_distance(self).await);
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

        let watched_chunks = {
            let pending_chunks = self.pending_chunks.lock();

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

            // Return chunks to stop watching and what to wait for
            watched_chunks
        };

        // Wait for tasks to finish
        self.client.await_expensive_tasks("Removing player").await;

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
    pub async fn send_abilties_update(&mut self) {
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
        let entity = &self.living_entity.entity;
        entity.set_pos(position.x, position.y, position.z);
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
        // Cancel tasks ASAP
        self.client.cancel_expensive_tasks("Kicking player");

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

    pub async fn set_gamemode(&self, gamemode: GameMode) {
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
            ))
            .await;
        self.client
            .send_packet(&CGameEvent::new(
                GameEvent::ChangeGameMode,
                gamemode.to_f32().unwrap(),
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
            //#[cfg(debug_assertions)]
            //let inst = std::time::Instant::now();
            tokio::select! {
                () = self.client.await_cancel_notify() => {
                    log::debug!("Canceling player packet processing");
                    return;
                },
                packet_result = self.handle_play_packet(server, &mut packet) => {
                    //#[cfg(debug_assertions)]
                    //log::debug!("Handled play packet in {:?}", inst.elapsed());
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

    #[expect(clippy::too_many_lines)]
    pub async fn handle_play_packet(
        self: &Arc<Self>,
        server: &Arc<Server>,
        packet: &mut RawPacket,
    ) -> Result<(), Box<dyn PumpkinError>> {
        let bytebuf = &mut packet.bytebuf;
        if let Some(packet) = ServerboundPlayPackets::from_i32(packet.id.0) {
            #[expect(clippy::match_same_arms)]
            match packet {
                ServerboundPlayPackets::TeleportConfirm => {
                    self.handle_confirm_teleport(SConfirmTeleport::read(bytebuf)?)
                        .await;
                }
                ServerboundPlayPackets::QueryBlockNbt => {}
                ServerboundPlayPackets::SelectBundleItem => {}
                ServerboundPlayPackets::SetDifficulty => {}
                ServerboundPlayPackets::ChatAck => {}
                ServerboundPlayPackets::ChatCommandUnsigned => {
                    self.handle_chat_command(server, SChatCommand::read(bytebuf)?)
                        .await;
                }
                ServerboundPlayPackets::ChatCommand => {}
                ServerboundPlayPackets::ChatMessage => {
                    self.handle_chat_message(SChatMessage::read(bytebuf)?).await;
                }
                ServerboundPlayPackets::ChatSessionUpdate => {}
                ServerboundPlayPackets::ChunkBatchAck => {}
                ServerboundPlayPackets::ClientStatus => {}
                ServerboundPlayPackets::ClientTickEnd => {}
                ServerboundPlayPackets::ClientSettings => {
                    self.handle_client_information(SClientInformationPlay::read(bytebuf)?)
                        .await;
                }
                ServerboundPlayPackets::TabComplete => {}
                ServerboundPlayPackets::ConfigurationAck => {}
                ServerboundPlayPackets::ClickWindowButton => {}
                ServerboundPlayPackets::ClickWindow => {}
                ServerboundPlayPackets::CloseWindow => {}
                ServerboundPlayPackets::SlotStateChange => {}
                ServerboundPlayPackets::CookieResponse => {}
                ServerboundPlayPackets::PluginMessage => {}
                ServerboundPlayPackets::DebugSampleSubscription => {}
                ServerboundPlayPackets::EditBook => {}
                ServerboundPlayPackets::QueryEntityNbt => {}
                ServerboundPlayPackets::InteractEntity => {
                    self.handle_interact(server, SInteract::read(bytebuf)?)
                        .await;
                }
                ServerboundPlayPackets::GenerateStructure => {}
                ServerboundPlayPackets::KeepAlive => {
                    self.handle_keep_alive(SKeepAlive::read(bytebuf)?).await;
                }
                ServerboundPlayPackets::LockDifficulty => {}
                ServerboundPlayPackets::PlayerPosition => {
                    self.handle_position(SPlayerPosition::read(bytebuf)?).await;
                }
                ServerboundPlayPackets::PlayerPositionAndRotation => {
                    self.handle_position_rotation(SPlayerPositionRotation::read(bytebuf)?)
                        .await;
                }
                ServerboundPlayPackets::PlayerRotation => {
                    self.handle_rotation(SPlayerRotation::read(bytebuf)?).await;
                }
                ServerboundPlayPackets::PlayerFlying => {
                    self.handle_player_ground(&SSetPlayerGround::read(bytebuf)?);
                }
                ServerboundPlayPackets::VehicleMove => {}
                ServerboundPlayPackets::SteerBoat => {}
                ServerboundPlayPackets::PickItem => {}
                ServerboundPlayPackets::DebugPing => {}
                ServerboundPlayPackets::CraftRecipeRequest => {}
                ServerboundPlayPackets::PlayerAbilities => {
                    self.handle_player_abilities(SPlayerAbilities::read(bytebuf)?)
                        .await;
                }
                ServerboundPlayPackets::PlayerDigging => {
                    self.handle_player_action(SPlayerAction::read(bytebuf)?)
                        .await;
                }
                ServerboundPlayPackets::EntityAction => {
                    self.handle_player_command(SPlayerCommand::read(bytebuf)?)
                        .await;
                }
                ServerboundPlayPackets::PlayerInput => {}
                ServerboundPlayPackets::Pong => {}
                ServerboundPlayPackets::SetRecipeBookState => {}
                ServerboundPlayPackets::SetDisplayedRecipe => {}
                ServerboundPlayPackets::NameItem => {}
                ServerboundPlayPackets::ResourcePackStatus => {}
                ServerboundPlayPackets::AdvancementTab => {}
                ServerboundPlayPackets::SelectTrade => {}
                ServerboundPlayPackets::SetBeaconEffect => {}
                ServerboundPlayPackets::HeldItemChange => {
                    self.handle_set_held_item(SSetHeldItem::read(bytebuf)?)
                        .await;
                }
                ServerboundPlayPackets::UpdateCommandBlock => {}
                ServerboundPlayPackets::UpdateCommandBlockMinecart => {}
                ServerboundPlayPackets::CreativeInventoryAction => {
                    self.handle_set_creative_slot(SSetCreativeSlot::read(bytebuf)?)
                        .await?;
                }
                ServerboundPlayPackets::UpdateJigsawBlock => {}
                ServerboundPlayPackets::UpdateStructureBlock => {}
                ServerboundPlayPackets::UpdateSign => {}
                ServerboundPlayPackets::Animation => {
                    self.handle_swing_arm(SSwingArm::read(bytebuf)?).await;
                }
                ServerboundPlayPackets::Spectate => {}
                ServerboundPlayPackets::PlayerBlockPlacement => {
                    self.handle_use_item_on(SUseItemOn::read(bytebuf)?).await;
                }
                ServerboundPlayPackets::UseItem => self.handle_use_item(&SUseItem::read(bytebuf)?),
            };
        } else {
            log::error!("Failed to handle player packet id {:#04x}", packet.id.0);
            return Err(Box::new(DeserializerError::UnknownPacket));
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
