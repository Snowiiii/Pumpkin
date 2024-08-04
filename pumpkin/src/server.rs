use std::{
    io::Cursor,
    sync::atomic::{AtomicI32, Ordering},
};

use base64::{engine::general_purpose, Engine};
use mio::{event::Event, Poll};
use pumpkin_protocol::{
    client::{
        config::CPluginMessage,
        play::{CChunkDataUpdateLight, CGameEvent, CLogin},
    },
    BitSet, PacketError, Players, Sample, StatusResponse, VarInt, VarInt32, Version,
};
use pumpkin_world::chunk::TestChunk;
use rsa::{rand_core::OsRng, traits::PublicKeyParts, RsaPrivateKey, RsaPublicKey};
use serde::{Deserialize, Serialize};

use crate::{
    client::Client,
    configuration::{AdvancedConfiguration, BasicConfiguration},
    entity::{
        player::{GameMode, Player},
        Entity, EntityId,
    },
};

pub struct Server {
    pub compression_threshold: Option<u8>,

    pub online_mode: bool,
    pub encryption: bool, // encryptiony is always required when online_mode is disabled
    pub public_key: RsaPublicKey,
    pub private_key: RsaPrivateKey,
    pub public_key_der: Box<[u8]>,

    /// the maximum amount of players that can join the Server
    pub max_players: u32,

    // pub world: World,
    pub status_response: StatusResponse,
    // We cache the json response here so we don't parse it every time someone makes a Status request.
    // Keep in mind that we must parse this again, when the StatusResponse changes which usally happen when a player joins or leaves
    pub status_response_json: String,

    /// Cache the Server brand buffer so we don't have to rebuild them every time a player joins
    pub cached_server_brand: Vec<u8>,

    // todo replace with HashMap <World, Player>
    entity_id: AtomicI32, // todo: place this into every world
    pub difficulty: Difficulty,
}

impl Server {
    pub fn new(config: (BasicConfiguration, AdvancedConfiguration)) -> Self {
        let status_response = Self::build_response(&config.0);
        let status_response_json = serde_json::to_string(&status_response)
            .expect("Failed to parse Status response into JSON");

        let cached_server_brand = Self::build_brand();

        // todo, only create when needed
        let (public_key, private_key) = Self::generate_keys();

        let public_key_der = rsa_der::public_key_to_der(
            &private_key.n().to_bytes_be(),
            &private_key.e().to_bytes_be(),
        )
        .into_boxed_slice();

        Self {
            // 0 is invalid
            entity_id: 2.into(),
            //  world: World::load(""),
            online_mode: config.0.online_mode,
            encryption: config.1.encryption,
            compression_threshold: None, // 256
            public_key,
            cached_server_brand,
            private_key,
            max_players: config.0.max_players,
            status_response,
            status_response_json,
            public_key_der,
            difficulty: config.0.default_difficulty,
        }
    }

    // Returns Tokens to remove
    pub fn poll(&mut self, client: &mut Client, _poll: &Poll, event: &Event) {
        // todo, Poll players in every world
        client.poll(self, event)
    }

    // todo: do this in a world
    pub fn spawn_player(&mut self, client: &mut Client) {
        dbg!("spawning player");
        let entity_id = self.new_entity_id();
        let player = Player::new(Entity { entity_id });
        client.player = Some(player);

        client
            .send_packet(CLogin::new(
                entity_id,
                self.difficulty == Difficulty::Hard,
                vec!["minecraft:overworld".into()],
                self.max_players as VarInt,
                8, //  view distance todo
                8, // sim view dinstance todo
                false,
                false,
                false,
                0,
                "minecraft:overworld".into(),
                0, // seed
                GameMode::Spectator.to_byte() as u8,
                GameMode::Spectator.to_byte(),
                false,
                false,
                false, // deth loc
                None,
                None,
                0,
                false,
            ))
            .unwrap_or_else(|e| client.kick(&e.to_string()));
        // teleport
        client.teleport(10.0, 500.0, 10.0, 10.0, 10.0);

        // Start waiting for level chunks
        client
            .send_packet(CGameEvent::new(13, 0.0))
            .unwrap_or_else(|e| client.kick(&e.to_string()));

        // Server::spawn_test_chunk(client);
    }

    // todo: do this in a world
    fn _spawn_test_chunk(client: &mut Client) {
        let test_chunk = TestChunk::new();
        client
            .send_packet(CChunkDataUpdateLight::new(
                10,
                10,
                test_chunk.heightmap,
                Vec::new(),
                Vec::new(),
                BitSet(0, Vec::new()),
                BitSet(0, Vec::new()),
                BitSet(0, Vec::new()),
                Vec::new(),
                Vec::new(),
            ))
            .unwrap_or_else(|e| client.kick(&e.to_string()))
    }

    // move to world
    pub fn new_entity_id(&self) -> EntityId {
        self.entity_id.fetch_add(1, Ordering::SeqCst)
    }

    pub fn build_brand() -> Vec<u8> {
        let brand = "pumpkin";
        let mut buf = vec![];
        let _ = VarInt32(brand.len() as i32).encode(&mut buf);
        buf.extend_from_slice(brand.as_bytes());
        buf
    }

    pub fn send_brand(&self, client: &mut Client) -> Result<(), PacketError> {
        // send server brand
        client.send_packet(CPluginMessage::new(
            "minecraft:brand",
            &self.cached_server_brand,
        ))
    }

    pub fn build_response(config: &BasicConfiguration) -> StatusResponse {
        let path = concat!(env!("CARGO_MANIFEST_DIR"), "/icon.png");
        let mut max_players = config.max_players;

        let online = 0; // get online players

        //increasing player counter
        if max_players == 0 {
            max_players = online + 1;
        }

        StatusResponse {
            version: Version {
                name: "1.21".into(),
                protocol: 767,
            },
            players: Players {
                max: max_players,
                online: 0,
                sample: vec![Sample {
                    name: "".into(),
                    id: "".into(),
                }],
            },
            description: config.motd.clone(),
            favicon: Self::load_icon(path),
        }
    }

    pub fn load_icon(path: &str) -> String {
        let mut icon = match image::open(path).map_err(|e| panic!("error loading icon: {}", e)) {
            Ok(icon) => icon,
            Err(_) => return "".into(),
        };
        icon = icon.resize_exact(64, 64, image::imageops::FilterType::Triangle);
        let mut image = Vec::new();
        icon.write_to(&mut Cursor::new(&mut image), image::ImageFormat::Png)
            .unwrap();
        let mut result = "data:image/png;base64,".to_owned();
        general_purpose::STANDARD.encode_string(image, &mut result);
        result
    }

    pub fn generate_keys() -> (RsaPublicKey, RsaPrivateKey) {
        let priv_key = RsaPrivateKey::new(&mut OsRng, 1024).expect("failed to generate a key");
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
