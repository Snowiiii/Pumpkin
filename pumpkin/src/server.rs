use std::{
    cell::RefCell,
    collections::HashMap,
    io::Cursor,
    rc::Rc,
    sync::atomic::{AtomicI32, Ordering},
};

use base64::{engine::general_purpose, Engine};
use mio::{event::Event, Poll, Token};
use num_traits::ToPrimitive;
use pumpkin_entity::{entity_type::EntityType, EntityId};
use pumpkin_protocol::{
    client::{
        config::CPluginMessage,
        play::{
            CChunkDataUpdateLight, CGameEvent, CLogin, CPlayerAbilities, CPlayerInfoUpdate,
            CSpawnEntity, PlayerAction,
        },
    },
    BitSet, ClientPacket, Players, Sample, StatusResponse, VarInt, Version, CURRENT_MC_PROTOCOL,
};
use pumpkin_world::chunk::TestChunk;
use rsa::{rand_core::OsRng, traits::PublicKeyParts, RsaPrivateKey, RsaPublicKey};
use serde::{Deserialize, Serialize};

use crate::{
    client::Client,
    configuration::{AdvancedConfiguration, BasicConfiguration},
    entity::player::{GameMode, Player},
};

pub const CURRENT_MC_VERSION: &str = "1.21";

pub struct Server {
    pub compression_threshold: Option<u8>,

    pub public_key: RsaPublicKey,
    pub private_key: RsaPrivateKey,
    pub public_key_der: Box<[u8]>,

    // pub world: World,
    pub status_response: StatusResponse,
    // We cache the json response here so we don't parse it every time someone makes a Status request.
    // Keep in mind that we must parse this again, when the StatusResponse changes which usally happen when a player joins or leaves
    pub status_response_json: String,

    /// Cache the Server brand buffer so we don't have to rebuild them every time a player joins
    pub cached_server_brand: Vec<u8>,

    pub current_clients: HashMap<Rc<Token>, Rc<RefCell<Client>>>,

    // todo replace with HashMap <World, Player>
    entity_id: AtomicI32, // todo: place this into every world
    pub base_config: BasicConfiguration,
    pub advanced_config: AdvancedConfiguration,

    /// Used for Authentication, None is Online mode is disabled
    pub auth_client: Option<reqwest::blocking::Client>,
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
        let auth_client = if config.0.online_mode {
            Some(reqwest::blocking::Client::new())
        } else {
            None
        };

        Self {
            // 0 is invalid
            entity_id: 2.into(),
            //  world: World::load(""),
            compression_threshold: None, // 256
            public_key,
            cached_server_brand,
            private_key,
            status_response,
            status_response_json,
            public_key_der,
            current_clients: HashMap::new(),
            base_config: config.0,
            auth_client,
            advanced_config: config.1,
        }
    }

    // Returns Tokens to remove
    pub fn poll(&mut self, client: &mut Client, _poll: &Poll, event: &Event) {
        // todo, Poll players in every world
        client.poll(self, event)
    }

    pub fn add_client(&mut self, token: Rc<Token>, client: Rc<RefCell<Client>>) {
        self.current_clients.insert(token, client);
    }

    pub fn remove_client(&mut self, token: &Token) {
        self.current_clients.remove(token);
    }

    // here is where the magic happens
    // todo: do this in a world
    pub fn spawn_player(&mut self, client: &mut Client) {
        // This code follows the vanilla packet order
        let entity_id = self.new_entity_id();
        log::debug!("spawning player, entity id {}", entity_id);
        let player = Player::new(entity_id);
        client.player = Some(player);

        // login packet for our new player
        client.send_packet(CLogin::new(
            entity_id,
            self.base_config.hardcore,
            vec!["minecraft:overworld".into()],
            self.base_config.max_players.into(),
            self.base_config.view_distance.into(), //  view distance todo
            self.base_config.simulation_distance.into(), // sim view dinstance todo
            false,
            false,
            false,
            0.into(),
            "minecraft:overworld".into(),
            0, // seed
            match self.base_config.default_gamemode {
                GameMode::Undefined => GameMode::Survival,
                game_mode => game_mode,
            }
            .to_u8()
            .unwrap(),
            self.base_config.default_gamemode.to_i8().unwrap(),
            false,
            false,
            false, // deth loc
            None,
            None,
            0.into(),
            false,
        ));
        // player abilities
        client.send_packet(CPlayerAbilities::new(0x02, 00.2, 0.1));

        // teleport
        let x = 10.0;
        let y = 500.0;
        let z = 10.0;
        client.teleport(x, y, z, 10.0, 10.0);
        let gameprofile = client.gameprofile.as_ref().unwrap();
        // first send info update to our new player, So he can see his Skin
        // TODO: send more actions, (chat. list, ping)
        client.send_packet(CPlayerInfoUpdate::new(
            0x01,
            &[pumpkin_protocol::client::play::Player {
                uuid: gameprofile.id,
                actions: vec![PlayerAction::AddPlayer {
                    name: gameprofile.name.clone(),
                    properties: gameprofile.properties.clone(),
                }],
            }],
        ));
        let gameprofile = client.gameprofile.as_ref().unwrap();
        // send his info to everyone else
        self.broadcast_packet(
            client,
            CPlayerInfoUpdate::new(
                0x01,
                &[pumpkin_protocol::client::play::Player {
                    uuid: gameprofile.id,
                    actions: vec![PlayerAction::AddPlayer {
                        name: gameprofile.name.clone(),
                        properties: gameprofile.properties.clone(),
                    }],
                }],
            ),
        );

        // here we send all the infos of already joined players
        let mut entries = Vec::new();
        for (_, client) in self.current_clients.iter().filter(|c| c.0 != &client.token) {
            let client = client.borrow();
            if client.is_player() {
                let gameprofile = client.gameprofile.as_ref().unwrap();
                entries.push(pumpkin_protocol::client::play::Player {
                    uuid: gameprofile.id,
                    actions: vec![PlayerAction::AddPlayer {
                        name: gameprofile.name.clone(),
                        properties: gameprofile.properties.clone(),
                    }],
                })
            }
        }
        client.send_packet(CPlayerInfoUpdate::new(0x01, &entries));

        // Start waiting for level chunks
        client.send_packet(CGameEvent::new(13, 0.0));

        let gameprofile = client.gameprofile.as_ref().unwrap();

        // spawn player for every client
        self.broadcast_packet(
            client,
            CSpawnEntity::new(
                entity_id.into(),
                gameprofile.id,
                EntityType::Player.to_i32().unwrap().into(),
                x,
                y,
                z,
                0,
                0,
                0,
                0.into(),
                0,
                0,
                0,
            ),
        );
        // spawn players for our client
        let token = client.token.clone();
        for (_, existing_client) in self.current_clients.iter().filter(|c| c.0 != &token) {
            let existing_client = existing_client.borrow();
            if let Some(player) = &existing_client.player {
                let entity = &player.entity;
                let gameprofile = existing_client.gameprofile.as_ref().unwrap();
                client.send_packet(CSpawnEntity::new(
                    player.entity_id().into(),
                    gameprofile.id,
                    EntityType::Player.to_i32().unwrap().into(),
                    entity.x,
                    entity.y,
                    entity.z,
                    entity.yaw as u8,
                    entity.pitch as u8,
                    0,
                    0.into(),
                    0,
                    0,
                    0,
                ))
            }
        }

        // Server::spawn_test_chunk(client);
    }

    /// Sends a Packet to all Players
    pub fn broadcast_packet<P>(&mut self, from: &Client, packet: P)
    where
        P: ClientPacket,
        P: Clone,
    {
        for (_, client) in self.current_clients.iter().filter(|c| c.0 != &from.token) {
            // Check if client is a player
            let mut client = client.borrow_mut();
            if client.is_player() {
                // we need to clone, Because we send a new packet to every client
                client.send_packet(packet.clone());
            }
        }
    }

    // todo: do this in a world
    fn _spawn_test_chunk(client: &mut Client) {
        let test_chunk = TestChunk::new();
        client.send_packet(CChunkDataUpdateLight::new(
            10,
            10,
            test_chunk.heightmap,
            Vec::new(),
            Vec::new(),
            BitSet(0.into(), Vec::new()),
            BitSet(0.into(), Vec::new()),
            BitSet(0.into(), Vec::new()),
            Vec::new(),
            Vec::new(),
        ));
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
        client.send_packet(CPluginMessage::new(
            "minecraft:brand",
            &self.cached_server_brand,
        ));
    }

    pub fn build_response(config: &BasicConfiguration) -> StatusResponse {
        let path = concat!(env!("CARGO_MANIFEST_DIR"), "/icon.png");

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
