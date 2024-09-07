use std::{
    collections::HashMap,
    ops::DerefMut,
    sync::{Arc, Mutex, MutexGuard},
};

pub mod player_chunker;

use mio::Token;
use num_traits::ToPrimitive;
use pumpkin_config::BasicConfiguration;
use pumpkin_core::math::vector2::Vector2;
use pumpkin_entity::{entity_type::EntityType, EntityId};
use pumpkin_protocol::{
    client::play::{
        CChunkData, CGameEvent, CLogin, CPlayerAbilities, CPlayerInfoUpdate, CRemoveEntities,
        CRemovePlayerInfo, CSetEntityMetadata, CSpawnEntity, Metadata, PlayerAction,
    },
    uuid::UUID,
    ClientPacket, VarInt,
};
use pumpkin_world::level::Level;
use tokio::sync::mpsc;

use crate::{client::Client, entity::player::Player};

pub struct World {
    pub level: Arc<Mutex<Level>>,
    pub current_players: HashMap<Token, Arc<Mutex<Player>>>,
    // entities, players...
}

impl World {
    pub fn load(level: Level) -> Self {
        Self {
            level: Arc::new(Mutex::new(level)),
            current_players: HashMap::new(),
        }
    }

    /// Sends a Packet to all Players, Expect some players. Because we can't lock them twice
    pub fn broadcast_packet<P>(&self, except: &[Token], packet: &P)
    where
        P: ClientPacket,
    {
        for (_, player) in self
            .current_players
            .iter()
            .filter(|c| !except.contains(c.0))
        {
            let mut player = player.lock().unwrap();
            player.client.send_packet(packet);
        }
    }

    pub async fn spawn_player(
        &mut self,
        base_config: &BasicConfiguration,
        player: Arc<Mutex<Player>>,
    ) {
        let mut player = player.lock().unwrap();
        let player = player.deref_mut();
        // This code follows the vanilla packet order
        let entity_id = player.entity_id();
        let gamemode = player.gamemode;
        log::debug!("spawning player, entity id {}", entity_id);

        // login packet for our new player
        player.client.send_packet(&CLogin::new(
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
            false,
        ));
        dbg!("sending abilities");
        // player abilities
        // TODO: this is for debug purpose, remove later
        player
            .client
            .send_packet(&CPlayerAbilities::new(0x02, 0.4, 0.1));

        // teleport
        let x = 10.0;
        let y = 120.0;
        let z = 10.0;
        let yaw = 10.0;
        let pitch = 10.0;
        player.teleport(x, y, z, 10.0, 10.0);
        let gameprofile = &player.gameprofile;
        // first send info update to our new player, So he can see his Skin
        // also send his info to everyone else
        player.client.send_packet(&CPlayerInfoUpdate::new(
            0x01 | 0x08,
            &[pumpkin_protocol::client::play::Player {
                uuid: gameprofile.id,
                actions: vec![
                    PlayerAction::AddPlayer {
                        name: gameprofile.name.clone(),
                        properties: gameprofile.properties.clone(),
                    },
                    PlayerAction::UpdateListed(true),
                ],
            }],
        ));
        self.broadcast_packet(
            &[player.client.token],
            &CPlayerInfoUpdate::new(
                0x01 | 0x08,
                &[pumpkin_protocol::client::play::Player {
                    uuid: gameprofile.id,
                    actions: vec![
                        PlayerAction::AddPlayer {
                            name: gameprofile.name.clone(),
                            properties: gameprofile.properties.clone(),
                        },
                        PlayerAction::UpdateListed(true),
                    ],
                }],
            ),
        );

        // here we send all the infos of already joined players
        let mut entries = Vec::new();
        for (_, playerr) in self
            .current_players
            .iter()
            .filter(|(c, _)| **c != player.client.token)
        {
            let playerr = playerr.as_ref().lock().unwrap();
            let gameprofile = &playerr.gameprofile;
            entries.push(pumpkin_protocol::client::play::Player {
                uuid: gameprofile.id,
                actions: vec![
                    PlayerAction::AddPlayer {
                        name: gameprofile.name.clone(),
                        properties: gameprofile.properties.clone(),
                    },
                    PlayerAction::UpdateListed(true),
                ],
            })
        }
        player
            .client
            .send_packet(&CPlayerInfoUpdate::new(0x01 | 0x08, &entries));

        let gameprofile = &player.gameprofile;

        // spawn player for every client
        self.broadcast_packet(
            &[player.client.token],
            // TODO: add velo
            &CSpawnEntity::new(
                entity_id.into(),
                UUID(gameprofile.id),
                (EntityType::Player as i32).into(),
                x,
                y,
                z,
                pitch,
                yaw,
                yaw,
                0.into(),
                0.0,
                0.0,
                0.0,
            ),
        );
        // spawn players for our client
        let token = player.client.token;
        for (_, existing_player) in self.current_players.iter().filter(|c| c.0 != &token) {
            let existing_player = existing_player.as_ref().lock().unwrap();
            let entity = &existing_player.entity;
            let gameprofile = &existing_player.gameprofile;
            player.client.send_packet(&CSpawnEntity::new(
                existing_player.entity_id().into(),
                UUID(gameprofile.id),
                (EntityType::Player as i32).into(),
                entity.pos.x,
                entity.pos.y,
                entity.pos.z,
                entity.yaw,
                entity.pitch,
                entity.head_yaw,
                0.into(),
                0.0,
                0.0,
                0.0,
            ))
        }
        // entity meta data
        // set skin parts
        if let Some(config) = player.client.config.as_ref() {
            let packet = CSetEntityMetadata::new(
                entity_id.into(),
                Metadata::new(17, VarInt(0), config.skin_parts),
            );
            player.client.send_packet(&packet);
            self.broadcast_packet(&[player.client.token], &packet)
        }

        // Start waiting for level chunks
        player.client.send_packet(&CGameEvent::new(13, 0.0));

        // Spawn in inital chunks
        player_chunker::player_join(self, player).await;
    }

    async fn spawn_world_chunks(
        &self,
        client: &mut Client,
        chunks: Vec<Vector2<i32>>,
        distance: i32,
    ) {
        let inst = std::time::Instant::now();
        let (sender, mut chunk_receiver) = mpsc::channel(distance as usize);

        let level = self.level.clone();
        let closed = client.closed;
        let chunks = Arc::new(chunks);
        tokio::task::spawn_blocking(move || {
            level.lock().unwrap().fetch_chunks(&chunks, sender, closed)
        });

        while let Some(chunk_data) = chunk_receiver.recv().await {
            if closed {
                return;
            }
            // dbg!(chunk_pos);
            let chunk_data = match chunk_data {
                Ok(d) => d,
                Err(_) => continue,
            };
            #[cfg(debug_assertions)]
            if chunk_data.position == (0, 0).into() {
                use pumpkin_protocol::bytebuf::ByteBuffer;
                let mut test = ByteBuffer::empty();
                CChunkData(&chunk_data).write(&mut test);
                let len = test.buf().len();
                log::debug!(
                    "Chunk packet size: {}B {}KB {}MB",
                    len,
                    len / 1024,
                    len / (1024 * 1024)
                );
            }
            client.send_packet(&CChunkData(&chunk_data));
        }
        dbg!("DONE CHUNKS", inst.elapsed());
    }

    /// TODO: This definitly should be in world
    pub fn get_by_entityid(&self, from: &Player, id: EntityId) -> Option<MutexGuard<Player>> {
        for (_, player) in self
            .current_players
            .iter()
            .filter(|c| c.0 != &from.client.token)
        {
            let player = player.lock().unwrap();
            if player.entity_id() == id {
                return Some(player);
            }
        }
        None
    }

    pub fn add_player(&mut self, token: Token, player: Arc<Mutex<Player>>) {
        self.current_players.insert(token, player);
    }

    pub fn remove_player(&mut self, player: &Player) {
        self.current_players.remove(&player.client.token).unwrap();
        // despawn the player
        // todo: put this into the entitiy struct
        let id = player.entity_id();
        let uuid = player.gameprofile.id;
        self.broadcast_packet(
            &[player.client.token],
            &CRemovePlayerInfo::new(1.into(), &[UUID(uuid)]),
        );
        self.broadcast_packet(&[player.client.token], &CRemoveEntities::new(&[id.into()]))
    }
}
