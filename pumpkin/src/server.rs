use std::{
    io::Cursor,
    path::Path,
    sync::{
        atomic::{AtomicI32, Ordering},
        Arc, Mutex,
    },
    time::Duration,
};

use base64::{engine::general_purpose, Engine};
use image::GenericImageView;
use mio::Token;
use pumpkin_entity::EntityId;
use pumpkin_plugin::PluginLoader;
use pumpkin_protocol::{
    client::config::CPluginMessage, ClientPacket, Players, Sample, StatusResponse, VarInt, Version,
    CURRENT_MC_PROTOCOL,
};
use pumpkin_world::dimension::Dimension;

use pumpkin_registry::Registry;
use rsa::{traits::PublicKeyParts, RsaPrivateKey, RsaPublicKey};
use serde::{Deserialize, Serialize};

use crate::{
    client::Client,
    config::{AdvancedConfiguration, BasicConfiguration},
    entity::player::{GameMode, Player},
    world::World,
};

pub const CURRENT_MC_VERSION: &str = "1.21.1";

pub struct Server {
    pub public_key: RsaPublicKey,
    pub private_key: RsaPrivateKey,
    pub public_key_der: Box<[u8]>,

    pub plugin_loader: PluginLoader,

    pub worlds: Vec<Arc<tokio::sync::Mutex<World>>>,
    pub status_response: StatusResponse,
    // We cache the json response here so we don't parse it every time someone makes a Status request.
    // Keep in mind that we must parse this again, when the StatusResponse changes which usally happen when a player joins or leaves
    pub status_response_json: String,

    /// Cache the Server brand buffer so we don't have to rebuild them every time a player joins
    pub cached_server_brand: Vec<u8>,

    /// Cache the registry so we don't have to parse it every time a player joins
    pub cached_registry: Vec<Registry>,

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
            worlds: vec![Arc::new(tokio::sync::Mutex::new(world))],
            public_key,
            cached_server_brand,
            private_key,
            status_response,
            status_response_json,
            public_key_der,
            base_config: config.0,
            auth_client,
            advanced_config: config.1,
        }
    }

    pub async fn add_player(
        &mut self,
        token: Arc<Token>,
        client: Client,
    ) -> (Arc<Mutex<Player>>, Arc<tokio::sync::Mutex<World>>) {
        let entity_id = self.new_entity_id();
        let gamemode = match self.base_config.default_gamemode {
            GameMode::Undefined => GameMode::Survival,
            game_mode => game_mode,
        };
        // Basicly the default world
        // TODO: select default from config
        let world = self.worlds[0].clone();
        let player = Arc::new(Mutex::new(Player::new(
            client,
            world.clone(),
            entity_id,
            gamemode,
        )));
        world.lock().await.add_player(token, player.clone());
        (player, world)
    }

    /// Sends a Packet to all Players in all worlds
    pub fn broadcast_packet_all<P>(&self, expect: &[&Arc<Token>], packet: &P)
    where
        P: ClientPacket,
    {
        for world in &self.worlds {
            world.blocking_lock().broadcast_packet(expect, packet)
        }
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
