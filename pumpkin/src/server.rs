use std::{
    collections::HashMap,
    io::Cursor,
    path::Path,
    sync::{
        atomic::{AtomicI32, Ordering},
        Arc, Mutex, MutexGuard,
    },
    time::Duration,
};

use base64::{engine::general_purpose, Engine};
use image::GenericImageView;
use mio::Token;
use num_traits::ToPrimitive;
use pumpkin_entity::{entity_type::EntityType, EntityId};
use pumpkin_plugin::PluginLoader;
use pumpkin_protocol::{
    client::{
        config::CPluginMessage,
        play::{
            CCenterChunk, CChunkData, CGameEvent, CLogin, CPlayerAbilities, CPlayerInfoUpdate,
            CRemoveEntities, CRemovePlayerInfo, CSetEntityMetadata, CSpawnEntity, Metadata,
            PlayerAction,
        },
    },
    uuid::UUID,
    ClientPacket, Players, Sample, StatusResponse, VarInt, Version, CURRENT_MC_PROTOCOL,
};
use pumpkin_world::{dimension::Dimension, radial_chunk_iterator::RadialIterator, World};

use pumpkin_registry::Registry;
use rsa::{traits::PublicKeyParts, RsaPrivateKey, RsaPublicKey};
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;

use crate::{
    client::Client,
    config::{AdvancedConfiguration, BasicConfiguration},
    entity::player::{GameMode, Player},
};

pub const CURRENT_MC_VERSION: &str = "1.21.1";

pub struct Server {
    pub public_key: RsaPublicKey,
    pub private_key: RsaPrivateKey,
    pub public_key_der: Box<[u8]>,

    pub plugin_loader: PluginLoader,

    pub world: Arc<tokio::sync::Mutex<World>>,
    pub status_response: StatusResponse,
    // We cache the json response here so we don't parse it every time someone makes a Status request.
    // Keep in mind that we must parse this again, when the StatusResponse changes which usally happen when a player joins or leaves
    pub status_response_json: String,

    /// Cache the Server brand buffer so we don't have to rebuild them every time a player joins
    pub cached_server_brand: Vec<u8>,

    /// Cache the registry so we don't have to parse it every time a player joins
    pub cached_registry: Vec<Registry>,

    // TODO: place this into every world
    pub current_players: HashMap<Arc<Token>, Arc<Mutex<Player>>>,

    entity_id: AtomicI32,
    pub base_config: BasicConfiguration,
    pub advanced_config: AdvancedConfiguration,

    /// Used for Authentication, None is Online mode is disabled
    pub auth_client: Option<reqwest::Client>,
}

impl Server {
    pub fn new(config: (BasicConfiguration, AdvancedConfiguration)) -> Self {
        let status_response = Self::build_response(&config.0);
        let status_response_json = serde_json::to_string(&status_response)
            .expect("Failed to parse Status response into JSON");
        let cached_server_brand = Self::build_brand();

        // TODO: only create when needed
        log::debug!("Creating encryption keys...");
        let (public_key, private_key) = Self::generate_keys();

        let public_key_der = rsa_der::public_key_to_der(
            &private_key.n().to_bytes_be(),
            &private_key.e().to_bytes_be(),
        )
        .into_boxed_slice();
        let auth_client = if config.0.online_mode {
            Some(
                reqwest::Client::builder()
                    .timeout(Duration::from_millis(5000))
                    .build()
                    .expect("Failed to to make reqwest client"),
            )
        } else {
            None
        };

        log::info!("Loading Plugins");
        let plugin_loader = PluginLoader::load();

        let world = World::load(Dimension::OverWorld.into_level(
            // TODO: load form config
            "./world".parse().unwrap(),
        ));

        Self {
            plugin_loader,
            cached_registry: Registry::get_static(),
            // 0 is invalid
            entity_id: 2.into(),
            world: Arc::new(tokio::sync::Mutex::new(world)),
            public_key,
            cached_server_brand,
            private_key,
            status_response,
            status_response_json,
            public_key_der,
            current_players: HashMap::new(),
            base_config: config.0,
            auth_client,
            advanced_config: config.1,
        }
    }

    pub fn add_player(&mut self, token: Arc<Token>, client: Client) -> Arc<Mutex<Player>> {
        let entity_id = self.new_entity_id();
        let gamemode = match self.base_config.default_gamemode {
            GameMode::Undefined => GameMode::Survival,
            game_mode => game_mode,
        };
        let player = Arc::new(Mutex::new(Player::new(client, entity_id, gamemode)));
        self.current_players.insert(token, player.clone());
        player
    }

    pub fn remove_player(&mut self, token: &Token) {
        let player = self.current_players.remove(token).unwrap();
        let player = player.as_ref().lock().unwrap();
        // despawn the player
        // todo: put this into the entitiy struct
        let id = player.entity_id();
        let uuid = player.gameprofile.id;
        self.broadcast_packet_except(
            &[&player.client.token],
            &CRemovePlayerInfo::new(1.into(), &[UUID(uuid)]),
        );
        self.broadcast_packet_except(&[&player.client.token], &CRemoveEntities::new(&[id.into()]))
    }

    // here is where the magic happens
    // TODO: do this in a world
    pub async fn spawn_player(&mut self, player: &mut Player) {
        // This code follows the vanilla packet order
        let entity_id = player.entity_id();
        let gamemode = player.gamemode;
        log::debug!("spawning player, entity id {}", entity_id);

        // login packet for our new player
        player.client.send_packet(&CLogin::new(
            entity_id,
            self.base_config.hardcore,
            &["minecraft:overworld"],
            self.base_config.max_players.into(),
            self.base_config.view_distance.into(), //  TODO: view distance
            self.base_config.simulation_distance.into(), // TODO: sim view dinstance
            false,
            false,
            false,
            0.into(),
            "minecraft:overworld",
            0, // seed
            gamemode.to_u8().unwrap(),
            self.base_config.default_gamemode.to_i8().unwrap(),
            false,
            false,
            None,
            0.into(),
            false,
        ));
        dbg!("sending abilities");
        // player abilities
        player
            .client
            .send_packet(&CPlayerAbilities::new(0x02, 0.1, 0.1));

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
        self.broadcast_packet(
            player,
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
            .filter(|c| c.0 != &player.client.token)
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

        // Start waiting for level chunks
        player.client.send_packet(&CGameEvent::new(13, 0.0));

        let gameprofile = &player.gameprofile;

        // spawn player for every client
        self.broadcast_packet_except(
            &[&player.client.token],
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
        let token = player.client.token.clone();
        for (_, existing_player) in self.current_players.iter().filter(|c| c.0 != &token) {
            let existing_player = existing_player.as_ref().lock().unwrap();
            let entity = &existing_player.entity;
            let gameprofile = &existing_player.gameprofile;
            player.client.send_packet(&CSpawnEntity::new(
                existing_player.entity_id().into(),
                UUID(gameprofile.id),
                EntityType::Player.to_i32().unwrap().into(),
                entity.x,
                entity.y,
                entity.z,
                entity.yaw,
                entity.pitch,
                entity.pitch,
                0.into(),
                0.0,
                0.0,
                0.0,
            ))
        }
        // entity meta data
        if let Some(config) = player.client.config.as_ref() {
            self.broadcast_packet(
                player,
                &CSetEntityMetadata::new(
                    entity_id.into(),
                    Metadata::new(17, VarInt(0), config.skin_parts),
                ),
            )
        }

        self.spawn_test_chunk(player, self.base_config.view_distance as u32)
            .await;
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

    /// Sends a Packet to all Players
    pub fn broadcast_packet<P>(&self, from: &mut Player, packet: &P)
    where
        P: ClientPacket,
    {
        // we can't borrow twice at same time
        from.client.send_packet(packet);
        for (_, player) in self
            .current_players
            .iter()
            .filter(|c| c.0 != &from.client.token)
        {
            let mut player = player.lock().unwrap();
            player.client.send_packet(packet);
        }
    }

    /// Sends a packet to all players except those specified in `from`
    pub fn broadcast_packet_except<P>(&self, from: &[&Token], packet: &P)
    where
        P: ClientPacket,
    {
        for (_, player) in self
            .current_players
            .iter()
            .filter(|c| !from.contains(&c.0.as_ref()))
        {
            let mut player = player.lock().unwrap();
            player.client.send_packet(packet);
        }
    }

    // TODO: do this in a world
    async fn spawn_test_chunk(&self, player: &mut Player, distance: u32) {
        let inst = std::time::Instant::now();
        let (sender, mut chunk_receiver) = mpsc::channel(distance as usize);
        let world = self.world.clone();

        let chunks: Vec<_> = RadialIterator::new(distance).collect();
        tokio::spawn(async move {
            world.lock().await.level.fetch_chunks(&chunks, sender);
        });

        player.client.send_packet(&CCenterChunk {
            chunk_x: 0.into(),
            chunk_z: 0.into(),
        });

        while let Some(chunk_data) = chunk_receiver.recv().await {
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
            player.client.send_packet(&CChunkData(&chunk_data));
        }
        let t = inst.elapsed();
        dbg!("DONE", t);
    }

    // move to world
    pub fn new_entity_id(&self) -> EntityId {
        self.entity_id.fetch_add(1, Ordering::SeqCst)
    }

    pub fn build_brand() -> Vec<u8> {
        let brand = "Pumpkin";
        let mut buf = vec![];
        let _ = VarInt(brand.len() as i32).encode(&mut buf);
        buf.extend_from_slice(brand.as_bytes());
        buf
    }

    pub fn send_brand(&self, client: &mut Client) {
        // send server brand
        client.send_packet(&CPluginMessage::new(
            "minecraft:brand",
            &self.cached_server_brand,
        ));
    }

    pub fn build_response(config: &BasicConfiguration) -> StatusResponse {
        let icon_path = concat!(env!("CARGO_MANIFEST_DIR"), "/icon.png");
        let icon = if Path::new(icon_path).exists() {
            Some(Self::load_icon(icon_path))
        } else {
            None
        };

        StatusResponse {
            version: Version {
                name: CURRENT_MC_VERSION.into(),
                protocol: CURRENT_MC_PROTOCOL,
            },
            players: Players {
                max: config.max_players,
                online: 0,
                sample: vec![Sample {
                    name: "".into(),
                    id: "".into(),
                }],
            },
            description: config.motd.clone(),
            favicon: icon,
        }
    }

    pub fn load_icon(path: &str) -> String {
        let icon = match image::open(path).map_err(|e| panic!("error loading icon: {}", e)) {
            Ok(icon) => icon,
            Err(_) => return "".into(),
        };
        let dimension = icon.dimensions();
        assert!(dimension.0 == 64, "Icon width must be 64");
        assert!(dimension.1 == 64, "Icon height must be 64");
        let mut image = Vec::with_capacity(64 * 64 * 4);
        icon.write_to(&mut Cursor::new(&mut image), image::ImageFormat::Png)
            .unwrap();
        let mut result = "data:image/png;base64,".to_owned();
        general_purpose::STANDARD.encode_string(image, &mut result);
        result
    }

    pub fn generate_keys() -> (RsaPublicKey, RsaPrivateKey) {
        let mut rng = rand::thread_rng();

        let priv_key = RsaPrivateKey::new(&mut rng, 1024).expect("failed to generate a key");
        let pub_key = RsaPublicKey::from(&priv_key);
        (pub_key, priv_key)
    }
}

#[derive(PartialEq, Serialize, Deserialize)]
pub enum Difficulty {
    Peaceful,
    Easy,
    Normal,
    Hard,
}
