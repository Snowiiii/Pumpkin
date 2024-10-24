use std::{collections::HashMap, sync::Arc};

pub mod player_chunker;

use crate::{
    client::Client,
    entity::{player::Player, Entity},
};
use num_traits::ToPrimitive;
use pumpkin_config::BasicConfiguration;
use pumpkin_core::math::vector2::Vector2;
use pumpkin_core::math::{position::WorldPosition, vector3::Vector3};
use pumpkin_entity::{entity_type::EntityType, EntityId};
use pumpkin_protocol::client::play::{CBlockUpdate, CWorldEvent};
use pumpkin_protocol::{
    client::play::{
        CChunkData, CGameEvent, CLogin, CPlayerAbilities, CPlayerInfoUpdate, CRemoveEntities,
        CRemovePlayerInfo, CSetEntityMetadata, CSpawnEntity, GameEvent, Metadata, PlayerAction,
    },
    ClientPacket, VarInt,
};
use pumpkin_world::block::BlockId;
use pumpkin_world::chunk::ChunkData;
use pumpkin_world::coordinates::ChunkRelativeBlockCoordinates;
use pumpkin_world::level::Level;
use scoreboard::Scoreboard;
use tokio::sync::{mpsc, RwLock};
use tokio::sync::{mpsc::Receiver, Mutex};

pub mod scoreboard;

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
    pub level: Arc<Mutex<Level>>,
    /// A map of active players within the world, keyed by their unique token.
    pub current_players: Arc<Mutex<HashMap<uuid::Uuid, Arc<Player>>>>,
    pub scoreboard: Mutex<Scoreboard>,
    // TODO: entities
}

impl World {
    #[must_use]
    pub fn load(level: Level) -> Self {
        Self {
            level: Arc::new(Mutex::new(level)),
            current_players: Arc::new(Mutex::new(HashMap::new())),
            scoreboard: Mutex::new(Scoreboard::new()),
        }
    }

    /// Broadcasts a packet to all connected players within the world.
    ///
    /// Sends the specified packet to every player currently logged in to the server.
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
    /// Sends the specified packet to every player currently logged in to the server, excluding the players listed in the `except` parameter.
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

    pub async fn tick(&self) {
        let current_players = self.current_players.lock().await;
        for player in current_players.values() {
            player.tick().await;
        }
    }

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

        // Spawn in initial chunks
        player_chunker::player_join(self, player.clone()).await;
    }

    pub async fn mark_chunks_as_not_watched(&self, chunks: &[Vector2<i32>]) {
        let level = self.level.lock().await;
        level.mark_chunk_as_not_watched_and_clean(chunks).await;
    }

    pub async fn mark_chunks_as_watched(&self, chunks: &[Vector2<i32>]) {
        let level = self.level.lock().await;
        level.mark_chunks_as_newly_watched(chunks);
    }

    pub async fn get_cached_chunk_len(&self) -> usize {
        let level = self.level.lock().await;
        level.loaded_chunk_count()
    }

    fn spawn_world_chunks(&self, client: Arc<Client>, chunks: Vec<Vector2<i32>>) {
        if client.closed.load(std::sync::atomic::Ordering::Relaxed) {
            log::info!("The connection has closed before world chunks were spawned",);
            return;
        }
        let inst = std::time::Instant::now();
        let mut receiver = self.receive_chunks(chunks);

        tokio::spawn(async move {
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

                // TODO: Queue player packs in a queue so we don't need to check if its closed before
                // sending
                if !client.closed.load(std::sync::atomic::Ordering::Relaxed) {
                    client.send_packet(&packet).await;
                }
            }

            log::debug!("chunks sent after {}ms", inst.elapsed().as_millis());
        });
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

    /// Gets a Player by name
    pub fn get_player_by_name(&self, name: &str) -> Option<Arc<Player>> {
        // not sure of blocking lock
        for player in self.current_players.blocking_lock().values() {
            if player.gameprofile.name == name {
                return Some(player.clone());
            }
        }
        None
    }

    /// Gets a Player by UUID
    pub async fn get_player_by_uuid(&self, id: uuid::Uuid) -> Option<Arc<Player>> {
        for player in self.current_players.lock().await.values() {
            if player.gameprofile.id == id {
                return Some(player.clone());
            }
        }
        None
    }

    pub async fn add_player(&self, uuid: uuid::Uuid, player: Arc<Player>) {
        self.current_players.lock().await.insert(uuid, player);
    }

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
    }

    pub async fn remove_entity(&self, entity: &Entity) {
        self.broadcast_packet_all(&CRemoveEntities::new(&[entity.entity_id.into()]))
            .await;
    }
    pub async fn set_block(&self, position: WorldPosition, block_id: BlockId) {
        let (chunk_coordinate, relative_coordinates) = position.chunk_and_chunk_relative_position();

        // Since we divide by 16 remnant can never exceed u8
        let relative = ChunkRelativeBlockCoordinates::from(relative_coordinates);

        let chunk = self.receive_chunk(chunk_coordinate).await;
        chunk.write().await.blocks.set_block(relative, block_id);

        self.broadcast_packet_all(&CBlockUpdate::new(
            &position,
            i32::from(block_id.data).into(),
        ))
        .await;
    }

    // Stream the chunks (don't collect them and then do stuff with them)
    pub fn receive_chunks(&self, chunks: Vec<Vector2<i32>>) -> Receiver<Arc<RwLock<ChunkData>>> {
        let (sender, receive) = mpsc::channel(chunks.len());
        {
            let level = self.level.clone();
            tokio::spawn(async move {
                let level = level.lock().await;
                level.fetch_chunks(&chunks, sender).await;
            });
        }
        receive
    }

    pub async fn receive_chunk(&self, chunk: Vector2<i32>) -> Arc<RwLock<ChunkData>> {
        let mut receiver = self.receive_chunks(vec![chunk]);
        receiver
            .recv()
            .await
            .expect("Channel closed for unknown reason")
    }

    pub async fn break_block(&self, position: WorldPosition) {
        self.set_block(position, BlockId { data: 0 }).await;

        self.broadcast_packet_all(&CWorldEvent::new(2001, &position, 11, false))
            .await;
    }

    pub async fn get_block(&self, position: WorldPosition) -> BlockId {
        let (chunk, relative) = position.chunk_and_chunk_relative_position();
        let relative = ChunkRelativeBlockCoordinates::from(relative);
        let chunk = self.receive_chunk(chunk).await;
        let chunk = chunk.read().await;
        chunk.blocks.get_block(relative)
    }
}
