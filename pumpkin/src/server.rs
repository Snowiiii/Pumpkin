use std::{
    collections::HashMap,
    io::Cursor,
    rc::Rc,
    sync::{
        atomic::{AtomicI32, Ordering},
        Arc, Mutex,
    },
};

use base64::{engine::general_purpose, Engine};
use mio::{net::TcpStream, Token};
use rsa::{rand_core::OsRng, traits::PublicKeyParts, RsaPrivateKey, RsaPublicKey};

use crate::{
    client::Client,
    entity::{
        player::{GameMode, Player},
        Entity, EntityId,
    },
    protocol::{client::play::CLogin, Players, Sample, StatusResponse, VarInt, Version},
};

pub struct Server {
    pub compression_threshold: Option<u8>,

    pub online_mode: bool,
    pub encryption: bool, // encryptiony is always required when online_mode is disabled
    pub public_key: RsaPublicKey,
    pub private_key: RsaPrivateKey,
    pub public_key_der: Box<[u8]>,

    pub max_players: u32,

    pub status_response: StatusResponse,
    pub connections: HashMap<Token, Client>,

    // todo replace with HashMap <World, Player>
    entity_id: AtomicI32, // todo: place this into every world
    pub players: Vec<Player>,
    pub difficulty: Difficulty,
}

impl Default for Server {
    fn default() -> Self {
        Self::new()
    }
}

impl Server {
    pub fn new() -> Self {
        let max_players = 20;
        let status_response = Self::default_response(max_players);

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
            online_mode: true,
            encryption: true,
            compression_threshold: None, // 256
            public_key,
            private_key,
            max_players,
            status_response,
            public_key_der,
            connections: HashMap::new(),
            players: Vec::new(),
            difficulty: Difficulty::Normal,
        }
    }

    pub fn new_client(&mut self, rc: Arc<Mutex<Server>>, connection: TcpStream, token: Token) {
        self.connections
            .insert(token, Client::new(rc, Rc::new(token), connection));
    }

    pub fn spawn_player(&mut self, token: &Token) {
        let mut player = Player {
            entity: Entity::new(self.new_entity_id()),
            client: self.connections.remove(token).unwrap(),
        };
        player.send_packet(CLogin::new(
            player.entity_id(),
            self.difficulty == Difficulty::Hard,
            1,
            vec!["minecraft:overworld".into()],
            self.max_players as VarInt,
            8, //  view distance todo
            8, // sim view dinstance todo
            false,
            false,
            false,
            1,
            "minecraft:overworld".into(),
            0, // seed
            GameMode::Survival,
            GameMode::Undefined,
            false,
            false,
            false, // deth loc
            None,
            None,
            0,
            false,
        ));

        self.players.push(player);
    }

    // move to world
    pub fn new_entity_id(&self) -> EntityId {
        self.entity_id.fetch_add(1, Ordering::SeqCst)
    }

    pub fn default_response(max_players: u32) -> StatusResponse {
        let path = concat!(env!("CARGO_MANIFEST_DIR"), "/icon.png");

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
            description: "Pumpkin Server".into(),
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

#[derive(PartialEq)]
pub enum Difficulty {
    Easy,
    Normal,
    Hard,
}
