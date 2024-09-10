use base64::{engine::general_purpose, Engine};
use bikeshed_key_store::BikeShedKeyStore;
use image::GenericImageView;
use mio::Token;
use parking_lot::{Mutex, RwLock};
use pumpkin_config::{BasicConfiguration, BASIC_CONFIG};
use pumpkin_core::GameMode;
use pumpkin_entity::EntityId;
use pumpkin_plugin::PluginLoader;
use pumpkin_protocol::client::login::CEncryptionRequest;
use pumpkin_protocol::{
    client::config::CPluginMessage, ClientPacket, Players, Sample, StatusResponse, VarInt, Version,
    CURRENT_MC_PROTOCOL,
};
use pumpkin_world::dimension::Dimension;
use std::collections::HashMap;
use std::{
    io::Cursor,
    path::Path,
    sync::{
        atomic::{AtomicI32, Ordering},
        Arc,
    },
    time::Duration,
};

use pumpkin_inventory::drag_handler::DragHandler;
use pumpkin_inventory::{Container, OpenContainer};
use pumpkin_registry::Registry;

use crate::client::EncryptionError;
use crate::{
    client::Client,
    commands::{default_dispatcher, dispatcher::CommandDispatcher},
    entity::player::Player,
    world::World,
};

mod bikeshed_key_store;
pub const CURRENT_MC_VERSION: &str = "1.21.1";

pub struct Server {
    key_store: BikeShedKeyStore,
    pub plugin_loader: PluginLoader,

    pub command_dispatcher: Arc<CommandDispatcher<'static>>,

    pub worlds: Vec<Arc<World>>,
    pub status_response: StatusResponse,
    // We cache the json response here so we don't parse it every time someone makes a Status request.
    // Keep in mind that we must parse this again, when the StatusResponse changes which usally happen when a player joins or leaves
    pub status_response_json: String,

    /// Cache the Server brand buffer so we don't have to rebuild them every time a player joins
    pub cached_server_brand: Vec<u8>,

    /// Cache the registry so we don't have to parse it every time a player joins
    pub cached_registry: Vec<Registry>,

    pub open_containers: RwLock<HashMap<u64, OpenContainer>>,
    pub drag_handler: DragHandler,
    entity_id: AtomicI32,

    /// Used for Authentication, None is Online mode is disabled
    pub auth_client: Option<reqwest::Client>,
}

impl Server {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        let status_response = Self::build_response(&BASIC_CONFIG);
        let status_response_json = serde_json::to_string(&status_response)
            .expect("Failed to parse Status response into JSON");
        let cached_server_brand = Self::build_brand();

        // TODO: only create when needed
        let key_store = BikeShedKeyStore::new();
        let auth_client = if BASIC_CONFIG.online_mode {
            Some(
                reqwest::Client::builder()
                    .timeout(Duration::from_millis(5000))
                    .build()
                    .expect("Failed to to make reqwest client"),
            )
        } else {
            None
        };

        // First register default command, after that plugins can put in their own
        let command_dispatcher = default_dispatcher();
        log::info!("Loading Plugins");
        let plugin_loader = PluginLoader::load();

        let world = World::load(Dimension::OverWorld.into_level(
            // TODO: load form config
            "./world".parse().unwrap(),
        ));
        Self {
            plugin_loader,
            cached_registry: Registry::get_static(),
            open_containers: RwLock::new(HashMap::new()),
            drag_handler: DragHandler::new(),
            // 0 is invalid
            entity_id: 2.into(),
            worlds: vec![Arc::new(world)],
            command_dispatcher: Arc::new(command_dispatcher),
            auth_client,
            key_store,
            status_response,
            status_response_json,
            cached_server_brand,
        }
    }

    pub async fn add_player(&self, token: Token, client: Client) -> (Arc<Player>, Arc<World>) {
        let entity_id = self.new_entity_id();
        let gamemode = match BASIC_CONFIG.default_gamemode {
            GameMode::Undefined => GameMode::Survival,
            game_mode => game_mode,
        };
        // Basicly the default world
        // TODO: select default from config
        let world = self.worlds[0].clone();

        let player = Arc::new(Player::new(client, world.clone(), entity_id, gamemode));
        world.add_player(token, player.clone());
        (player, world)
    }

    pub fn try_get_container(
        &self,
        player_id: EntityId,
        container_id: u64,
    ) -> Option<Arc<Mutex<Box<dyn Container>>>> {
        let open_containers = self.open_containers.read();
        open_containers
            .get(&container_id)?
            .try_open(player_id)
            .cloned()
    }

    /// Sends a Packet to all Players in all worlds
    pub fn broadcast_packet_all<P>(&self, packet: &P)
    where
        P: ClientPacket,
    {
        for world in &self.worlds {
            world.broadcast_packet_all(packet)
        }
    }

    /// Searches every world for a player by name
    pub fn get_player_by_name(&self, name: &str) -> Option<Arc<Player>> {
        for world in self.worlds.iter() {
            if let Some(player) = world.get_player_by_name(name) {
                return Some(player);
            }
        }
        None
    }

    /// Generates a new entity id
    /// This should be global
    pub fn new_entity_id(&self) -> EntityId {
        self.entity_id.fetch_add(1, Ordering::SeqCst)
    }

    pub fn encryption_request<'a>(
        &'a self,
        verification_token: &'a [u8; 4],
        should_authenticate: bool,
    ) -> CEncryptionRequest<'_> {
        self.key_store
            .encryption_request("", verification_token, should_authenticate)
    }

    pub fn decrypt(&self, data: &[u8]) -> Result<Vec<u8>, EncryptionError> {
        self.key_store.decrypt(data)
    }

    pub fn digest_secret(&self, secret: &[u8]) -> String {
        self.key_store.get_digest(secret)
    }

    pub fn build_brand() -> Vec<u8> {
        let brand = "Pumpkin";
        let mut buf = vec![];
        let _ = VarInt(brand.len() as i32).encode(&mut buf);
        buf.extend_from_slice(brand.as_bytes());
        buf
    }

    pub fn send_brand(&self, client: &Client) {
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
            version: Some(Version {
                name: CURRENT_MC_VERSION.into(),
                protocol: CURRENT_MC_PROTOCOL,
            }),
            players: Some(Players {
                max: config.max_players,
                online: 0,
                sample: vec![Sample {
                    name: "".into(),
                    id: "".into(),
                }],
            }),
            description: config.motd.clone(),
            favicon: icon,
            // TODO
            enforece_secure_chat: false,
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
}
