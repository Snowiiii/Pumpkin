use std::{
    io::Cursor,
    sync::atomic::{AtomicI32, Ordering},
};

use base64::{engine::general_purpose, Engine};
use mio::{event::Event, Poll};
use rsa::{rand_core::OsRng, traits::PublicKeyParts, RsaPrivateKey, RsaPublicKey};
use serde::{Deserialize, Serialize};

use crate::{
    client::Client,
    configuration::{AdvancedConfiguration, BasicConfiguration},
    entity::{
        player::{GameMode, Player},
        Entity, EntityId,
    },
    protocol::{
        client::{
            config::{CPluginMessage},
            play::CLogin,
        },
        Players, Sample, StatusResponse, VarInt, VarInt32, Version,
    },
    world::world::World,
};

pub struct Server {
    pub compression_threshold: Option<u8>,

    pub online_mode: bool,
    pub encryption: bool, // encryptiony is always required when online_mode is disabled
    pub public_key: RsaPublicKey,
    pub private_key: RsaPrivateKey,
    pub public_key_der: Box<[u8]>,

    pub max_players: u32,

    pub world: World,

    pub status_response: StatusResponse,

    // todo replace with HashMap <World, Player>
    entity_id: AtomicI32, // todo: place this into every world
    pub difficulty: Difficulty,
}

impl Server {
    pub fn new(config: (BasicConfiguration, AdvancedConfiguration)) -> Self {
        let max_players = 20;
        let config_clone = &config;
        let status_response = Self::default_response(config_clone);

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
            world: World::new(),
            online_mode: config.0.online_mode,
            encryption: config.1.encryption,
            compression_threshold: None, // 256
            public_key,
            private_key,
            max_players,
            status_response,
            public_key_der,
            difficulty: config.0.default_difficulty,
        }
    }

    // Returns Tokens to remove
    pub fn poll(
        &mut self,
        client: &mut Client,
        poll: &Poll,
        event: &Event,
    ) -> anyhow::Result<bool> {
        let _ = poll;
        // todo, Poll players in every world
        client.poll(self, event)
    }

    pub fn spawn_player(&mut self, client: &mut Client) {
        let player = Player {
            entity: Entity {
                entity_id: self.new_entity_id(),
            },
        };

        client.send_packet(CLogin::new(
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

        client.player = Some(player);
    }

    // move to world
    pub fn new_entity_id(&self) -> EntityId {
        self.entity_id.fetch_add(1, Ordering::SeqCst)
    }

    pub fn send_brand(client: &mut Client) {
        // send server brand
        let brand = "pumpkin";
        let mut buf = vec![];
        let _ = VarInt32(brand.len() as i32).encode(&mut buf);
        buf.extend_from_slice(brand.as_bytes());
        client.send_packet(CPluginMessage::new(
            "minecraft:brand".to_string(),
            buf.as_slice(),
        ))
    }

    pub fn default_response(
        config: &(BasicConfiguration, AdvancedConfiguration),
    ) -> StatusResponse {
        let path = concat!(env!("CARGO_MANIFEST_DIR"), "/icon.png");

        StatusResponse {
            version: Version {
                name: "1.21".into(),
                protocol: 767,
            },
            players: Players {
                max: config.0.max_plyers,
                online: 0,
                sample: vec![Sample {
                    name: "".into(),
                    id: "".into(),
                }],
            },
            description: config.0.motd.clone(),
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
