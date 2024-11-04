use std::{
    collections::{hash_map::Entry, HashMap, VecDeque},
    sync::Arc,
};

pub mod player_chunker;

use crate::entity::{
    player::{ChunkHandleWrapper, Player},
    Entity,
};
use num_traits::ToPrimitive;
use pumpkin_config::BasicConfiguration;
use pumpkin_core::math::vector2::Vector2;
use pumpkin_core::math::{position::WorldPosition, vector3::Vector3};
use pumpkin_core::text::{color::NamedColor, TextComponent};
use pumpkin_entity::{entity_type::EntityType, EntityId};
use pumpkin_protocol::{
    client::play::{CBlockUpdate, CSoundEffect, CWorldEvent},
    SoundCategory,
};
use pumpkin_protocol::{
    client::play::{
        CChunkData, CGameEvent, CLogin, CPlayerAbilities, CPlayerInfoUpdate, CRemoveEntities,
        CRemovePlayerInfo, CSetEntityMetadata, CSpawnEntity, GameEvent, Metadata, PlayerAction,
    },
    ClientPacket, VarInt,
};
use pumpkin_world::chunk::ChunkData;
use pumpkin_world::coordinates::ChunkRelativeBlockCoordinates;
use pumpkin_world::level::Level;
use rand::{thread_rng, Rng};
use scoreboard::Scoreboard;
use tokio::sync::{mpsc::Receiver, Mutex};
use tokio::{
    sync::{mpsc, RwLock},
    task::JoinHandle,
};
use worldborder::Worldborder;

pub mod scoreboard;
pub mod worldborder;

type ChunkReceiver = (
    Vec<(Vector2<i32>, JoinHandle<()>)>,
    Receiver<Arc<RwLock<ChunkData>>>,
);

/// Represents a Minecraft world, containing entities, players, and the underlying level data.
///
/// Each dimension (Overworld, Nether, End) typically has its own `World`.
///
/// **Key Responsibilities:**
///
/// - Manages the `Level` instance for handling chunk-related operations.
/// - Stores and tracks active `Player` entities within the world.
/// - Provides a central hub for interacting with the world's entities and environment.
pub struct World {
    /// The underlying level, responsible for chunk management and terrain generation.
    pub level: Arc<Level>,
    /// A map of active players within the world, keyed by their unique UUID.
    pub current_players: Arc<Mutex<HashMap<uuid::Uuid, Arc<Player>>>>,
    /// The world's scoreboard, used for tracking scores, objectives, and display information.
    pub scoreboard: Mutex<Scoreboard>,
    /// The world's worldborder, defining the playable area and controlling its expansion or contraction.
    pub worldborder: Mutex<Worldborder>,
    // TODO: entities
}

impl World {
    #[must_use]
    pub fn load(level: Level) -> Self {
        Self {
            level: Arc::new(level),
            current_players: Arc::new(Mutex::new(HashMap::new())),
            scoreboard: Mutex::new(Scoreboard::new()),
            worldborder: Mutex::new(Worldborder::new(0.0, 0.0, 29_999_984.0, 0, 0, 0)),
        }
    }

    /// Broadcasts a packet to all connected players within the world.
    ///
    /// Sends the specified packet to every player currently logged in to the world.
    ///
    /// **Note:** This function acquires a lock on the `current_players` map, ensuring thread safety.
    pub async fn broadcast_packet_all<P>(&self, packet: &P)
    where
        P: ClientPacket,
    {
        let current_players = self.current_players.lock().await;
        for player in current_players.values() {
            player.client.send_packet(packet).await;
        }
    }

    /// Broadcasts a packet to all connected players within the world, excluding the specified players.
    ///
    /// Sends the specified packet to every player currently logged in to the world, excluding the players listed in the `except` parameter.
    ///
    /// **Note:** This function acquires a lock on the `current_players` map, ensuring thread safety.
    pub async fn broadcast_packet_expect<P>(&self, except: &[uuid::Uuid], packet: &P)
    where
        P: ClientPacket,
    {
        let current_players = self.current_players.lock().await;
        for (_, player) in current_players.iter().filter(|c| !except.contains(c.0)) {
            player.client.send_packet(packet).await;
        }
    }

    pub async fn play_sound(
        &self,
        sound_id: u16,
        category: SoundCategory,
        posistion: &Vector3<f64>,
    ) {
        let seed = thread_rng().gen::<f64>();
        self.broadcast_packet_all(&CSoundEffect::new(
            VarInt(i32::from(sound_id)),
            category,
            posistion.x,
            posistion.y,
            posistion.z,
            1.0,
            1.0,
            seed,
        ))
        .await;
    }

    pub async fn tick(&self) {
        let current_players = self.current_players.lock().await;
        for player in current_players.values() {
            player.tick().await;
        }
    }

    #[expect(clippy::too_many_lines)]
    pub async fn spawn_player(&self, base_config: &BasicConfiguration, player: Arc<Player>) {
        // This code follows the vanilla packet order
        let entity_id = player.entity_id();
        let gamemode = player.gamemode.load();
        log::debug!(
            "spawning player {}, entity id {}",
            player.gameprofile.name,
            entity_id
        );

        // login packet for our new player
        player
            .client
            .send_packet(&CLogin::new(
                entity_id,
                base_config.hardcore,
                &["minecraft:overworld"],
                base_config.max_players.into(),
                base_config.view_distance.into(), //  TODO: view distance
                base_config.simulation_distance.into(), // TODO: sim view dinstance
                false,
                false,
                false,
                0.into(),
                "minecraft:overworld",
                0, // seed
                gamemode.to_u8().unwrap(),
                base_config.default_gamemode.to_i8().unwrap(),
                false,
                false,
                None,
                0.into(),
                0.into(),
                false,
            ))
            .await;

        // player abilities
        // TODO: this is for debug purpose, remove later
        log::debug!("Sending player abilities to {}", player.gameprofile.name);
        player
            .client
            .send_packet(&CPlayerAbilities::new(0x02, 0.4, 0.1))
            .await;

        // teleport
        let position = Vector3::new(10.0, 120.0, 10.0);
        let yaw = 10.0;
        let pitch = 10.0;

        log::debug!("Sending player teleport to {}", player.gameprofile.name);
        player.teleport(position, yaw, pitch).await;

        let pos = player.living_entity.entity.pos.load();
        player.last_position.store(pos);

        let gameprofile = &player.gameprofile;
        // first send info update to our new player, So he can see his Skin
        // also send his info to everyone else
        log::debug!("Broadcasting player info for {}", player.gameprofile.name);
        self.broadcast_packet_all(&CPlayerInfoUpdate::new(
            0x01 | 0x08,
            &[pumpkin_protocol::client::play::Player {
                uuid: gameprofile.id,
                actions: vec![
                    PlayerAction::AddPlayer {
                        name: &gameprofile.name,
                        properties: &gameprofile.properties,
                    },
                    PlayerAction::UpdateListed(true),
                ],
            }],
        ))
        .await;

        // here we send all the infos of already joined players
        let mut entries = Vec::new();
        {
            let current_players = self.current_players.lock().await;
            for (_, playerr) in current_players
                .iter()
                .filter(|(c, _)| **c != player.gameprofile.id)
            {
                let gameprofile = &playerr.gameprofile;
                entries.push(pumpkin_protocol::client::play::Player {
                    uuid: gameprofile.id,
                    actions: vec![
                        PlayerAction::AddPlayer {
                            name: &gameprofile.name,
                            properties: &gameprofile.properties,
                        },
                        PlayerAction::UpdateListed(true),
                    ],
                });
            }
            log::debug!("Sending player info to {}", player.gameprofile.name);
            player
                .client
                .send_packet(&CPlayerInfoUpdate::new(0x01 | 0x08, &entries))
                .await;
        }

        let gameprofile = &player.gameprofile;

        log::debug!("Broadcasting player spawn for {}", player.gameprofile.name);
        // spawn player for every client
        self.broadcast_packet_expect(
            &[player.gameprofile.id],
            // TODO: add velo
            &CSpawnEntity::new(
                entity_id.into(),
                gameprofile.id,
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
        // spawn players for our client
        let id = player.gameprofile.id;
        for (_, existing_player) in self
            .current_players
            .lock()
            .await
            .iter()
            .filter(|c| c.0 != &id)
        {
            let entity = &existing_player.living_entity.entity;
            let pos = entity.pos.load();
            let gameprofile = &existing_player.gameprofile;
            log::debug!("Sending player entities to {}", player.gameprofile.name);
            player
                .client
                .send_packet(&CSpawnEntity::new(
                    existing_player.entity_id().into(),
                    gameprofile.id,
                    (EntityType::Player as i32).into(),
                    pos.x,
                    pos.y,
                    pos.z,
                    entity.yaw.load(),
                    entity.pitch.load(),
                    entity.head_yaw.load(),
                    0.into(),
                    0.0,
                    0.0,
                    0.0,
                ))
                .await;
        }
        // entity meta data
        // set skin parts
        if let Some(config) = player.client.config.lock().await.as_ref() {
            let packet = CSetEntityMetadata::new(
                entity_id.into(),
                Metadata::new(17, VarInt(0), config.skin_parts),
            );
            log::debug!("Broadcasting skin for {}", player.gameprofile.name);
            self.broadcast_packet_all(&packet).await;
        }

        // Start waiting for level chunks, Sets the "Loading Terrain" screen
        log::debug!("Sending waiting chunks to {}", player.gameprofile.name);
        player
            .client
            .send_packet(&CGameEvent::new(GameEvent::StartWaitingChunks, 0.0))
            .await;

        self.worldborder
            .lock()
            .await
            .init_client(&player.client)
            .await;

        // Spawn in initial chunks
        player_chunker::player_join(self, player.clone()).await;
    }

    pub fn mark_chunks_as_not_watched(&self, chunks: &[Vector2<i32>]) -> Vec<Vector2<i32>> {
        self.level.mark_chunks_as_not_watched(chunks)
    }

    pub fn clean_chunks(&self, chunks: &[Vector2<i32>]) {
        self.level.clean_chunks(chunks);
    }

    pub fn clean_memory(&self, chunks_to_check: &[Vector2<i32>]) {
        self.level.clean_memory(chunks_to_check);
    }

    pub fn get_cached_chunk_len(&self) -> usize {
        self.level.loaded_chunk_count()
    }

    #[expect(clippy::too_many_lines)]
    fn spawn_world_chunks(&self, player: Arc<Player>, chunks: &[Vector2<i32>]) {
        if player
            .client
            .closed
            .load(std::sync::atomic::Ordering::Relaxed)
        {
            log::info!("The connection has closed before world chunks were spawned",);
            return;
        }
        #[cfg(debug_assertions)]
        let inst = std::time::Instant::now();
        // Unique id of this chunk batch for later removal
        let id = uuid::Uuid::new_v4();

        let (pending, mut receiver) = self.receive_chunks(chunks);
        {
            let mut pending_chunks = player.pending_chunks.lock();

            for chunk in chunks {
                if pending_chunks.contains_key(chunk) {
                    log::debug!(
                        "Client id {} is requesting chunk {:?} but its already pending!",
                        player.client.id,
                        chunk
                    );
                }
            }

            for (chunk, handle) in pending {
                let entry = pending_chunks.entry(chunk);
                let wrapper = ChunkHandleWrapper::new(handle);
                match entry {
                    Entry::Occupied(mut entry) => {
                        entry.get_mut().push_back(wrapper);
                    }
                    Entry::Vacant(entry) => {
                        let mut queue = VecDeque::new();
                        queue.push_back(wrapper);
                        entry.insert(queue);
                    }
                };
            }
        }
        let pending_chunks = player.pending_chunks.clone();
        let level = self.level.clone();
        let retained_player = player.clone();
        let batch_id = id;

        let handle = tokio::spawn(async move {
            while let Some(chunk_data) = receiver.recv().await {
                let chunk_data = chunk_data.read().await;
                let packet = CChunkData(&chunk_data);
                #[cfg(debug_assertions)]
                if chunk_data.position == (0, 0).into() {
                    use pumpkin_protocol::bytebuf::ByteBuffer;
                    let mut test = ByteBuffer::empty();
                    packet.write(&mut test);
                    let len = test.buf().len();
                    log::debug!(
                        "Chunk packet size: {}B {}KB {}MB",
                        len,
                        len / 1024,
                        len / (1024 * 1024)
                    );
                }

                {
                    let mut pending_chunks = pending_chunks.lock();
                    let handlers = pending_chunks
                        .get_mut(&chunk_data.position)
                        .expect("All chunks should be pending");
                    let handler = handlers
                        .pop_front()
                        .expect("All chunks should have a handler");

                    if handlers.is_empty() {
                        pending_chunks.remove(&chunk_data.position);
                    }

                    // Chunk loading task was canceled after it was completed
                    if handler.aborted() {
                        // We never increment the watch value
                        if level.should_pop_chunk(&chunk_data.position) {
                            level.clean_chunks(&[chunk_data.position]);
                        }
                        // If ignored, dont send the packet
                        let loaded_chunks = level.loaded_chunk_count();
                        log::debug!(
                            "Aborted chunk {:?} (post-process) {} cached",
                            chunk_data.position,
                            loaded_chunks
                        );

                        // We dont want to mark this chunk as watched or send it to the client
                        continue;
                    }

                    // This must be locked with pending
                    level.mark_chunk_as_newly_watched(chunk_data.position);
                };

                if !player
                    .client
                    .closed
                    .load(std::sync::atomic::Ordering::Relaxed)
                {
                    player.client.send_packet(&packet).await;
                }
            }

            {
                let mut batch = player.pending_chunk_batch.lock();
                batch.remove(&batch_id);
            }
            #[cfg(debug_assertions)]
            log::debug!(
                "chunks sent after {}ms (batch {})",
                inst.elapsed().as_millis(),
                batch_id
            );
        });

        {
            let mut batch_handles = retained_player.pending_chunk_batch.lock();
            batch_handles.insert(id, handle);
        }
    }

    /// Gets a Player by entity id
    pub async fn get_player_by_entityid(&self, id: EntityId) -> Option<Arc<Player>> {
        for player in self.current_players.lock().await.values() {
            if player.entity_id() == id {
                return Some(player.clone());
            }
        }
        None
    }

    /// Gets a Player by username
    pub async fn get_player_by_name(&self, name: &str) -> Option<Arc<Player>> {
        for player in self.current_players.lock().await.values() {
            if player.gameprofile.name == name {
                return Some(player.clone());
            }
        }
        None
    }

    /// Retrieves a player by their unique UUID.
    ///
    /// This function searches the world's active player list for a player with the specified UUID.
    /// If found, it returns an `Arc<Player>` reference to the player. Otherwise, it returns `None`.
    ///
    /// # Arguments
    ///
    /// * `id`: The UUID of the player to retrieve.
    ///
    /// # Returns
    ///
    /// An `Option<Arc<Player>>` containing the player if found, or `None` if not.
    pub async fn get_player_by_uuid(&self, id: uuid::Uuid) -> Option<Arc<Player>> {
        return self.current_players.lock().await.get(&id).cloned();
    }

    /// Adds a player to the world and broadcasts a join message if enabled.
    ///
    /// This function takes a player's UUID and an `Arc<Player>` reference.
    /// It inserts the player into the world's `current_players` map using the UUID as the key.
    /// Additionally, it may broadcasts a join message to all connected players in the world.
    ///
    /// # Arguments
    ///
    /// * `uuid`: The unique UUID of the player to add.
    /// * `player`: An `Arc<Player>` reference to the player object.
    pub async fn add_player(&self, uuid: uuid::Uuid, player: Arc<Player>) {
        let mut current_players = self.current_players.lock().await;
        current_players.insert(uuid, player.clone());

        // Handle join message
        // TODO: Config
        let msg_txt = format!("{} joined the game.", player.gameprofile.name.as_str());
        let msg_comp = TextComponent::text(msg_txt.as_str()).color_named(NamedColor::Yellow);
        for player in current_players.values() {
            player.send_system_message(&msg_comp).await;
        }
        log::info!("{}", msg_comp.to_pretty_console());
    }

    /// Removes a player from the world and broadcasts a disconnect message if enabled.
    ///
    /// This function removes a player from the world based on their `Player` reference.
    /// It performs the following actions:
    ///
    /// 1. Removes the player from the `current_players` map using their UUID.
    /// 2. Broadcasts a `CRemovePlayerInfo` packet to all connected players to inform them about the player leaving.
    /// 3. Removes the player's entity from the world using its entity ID.
    /// 4. Optionally sends a disconnect message to all other players notifying them about the player leaving.
    ///
    /// # Arguments
    ///
    /// * `player`: A reference to the `Player` object to be removed.
    ///
    /// # Notes
    ///
    /// - This function assumes `broadcast_packet_expect` and `remove_entity` are defined elsewhere.
    /// - The disconnect message sending is currently optional. Consider making it a configurable option.
    pub async fn remove_player(&self, player: &Player) {
        self.current_players
            .lock()
            .await
            .remove(&player.gameprofile.id)
            .unwrap();
        let uuid = player.gameprofile.id;
        self.broadcast_packet_expect(
            &[player.gameprofile.id],
            &CRemovePlayerInfo::new(1.into(), &[uuid]),
        )
        .await;
        self.remove_entity(&player.living_entity.entity).await;

        // Send disconnect message / quit message to players in the same world
        // TODO: Config
        let disconn_msg_txt = format!("{} left the game.", player.gameprofile.name.as_str());
        let disconn_msg_cmp =
            TextComponent::text(disconn_msg_txt.as_str()).color_named(NamedColor::Yellow);
        for player in self.current_players.lock().await.values() {
            player.send_system_message(&disconn_msg_cmp).await;
        }
        log::info!("{}", disconn_msg_cmp.to_pretty_console());
    }

    pub async fn remove_entity(&self, entity: &Entity) {
        self.broadcast_packet_all(&CRemoveEntities::new(&[entity.entity_id.into()]))
            .await;
    }
    pub async fn set_block(&self, position: WorldPosition, block_id: u16) {
        let (chunk_coordinate, relative_coordinates) = position.chunk_and_chunk_relative_position();

        // Since we divide by 16 remnant can never exceed u8
        let relative = ChunkRelativeBlockCoordinates::from(relative_coordinates);

        let chunk = self.receive_chunk(chunk_coordinate).await;
        chunk.write().await.blocks.set_block(relative, block_id);

        self.broadcast_packet_all(&CBlockUpdate::new(&position, i32::from(block_id).into()))
            .await;
    }

    // Stream the chunks (don't collect them and then do stuff with them)
    pub fn receive_chunks(&self, chunks: &[Vector2<i32>]) -> ChunkReceiver {
        let (sender, receive) = mpsc::channel(chunks.len());
        let pending_chunks = self.level.fetch_chunks(chunks, sender);
        (pending_chunks, receive)
    }

    pub async fn receive_chunk(&self, chunk: Vector2<i32>) -> Arc<RwLock<ChunkData>> {
        let (_, mut receiver) = self.receive_chunks(&[chunk]);
        receiver
            .recv()
            .await
            .expect("Channel closed for unknown reason")
    }

    pub async fn break_block(&self, position: WorldPosition) {
        self.set_block(position, 0).await;

        self.broadcast_packet_all(&CWorldEvent::new(2001, &position, 11, false))
            .await;
    }

    pub async fn get_block(&self, position: WorldPosition) -> u16 {
        let (chunk, relative) = position.chunk_and_chunk_relative_position();
        let relative = ChunkRelativeBlockCoordinates::from(relative);
        let chunk = self.receive_chunk(chunk).await;
        let chunk = chunk.read().await;
        chunk.blocks.get_block(relative)
    }
}
