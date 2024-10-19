use connection_cache::{CachedBranding, CachedStatus};
use key_store::KeyStore;
use parking_lot::{Mutex, RwLock};
use pumpkin_config::BASIC_CONFIG;
use pumpkin_core::text::TextComponent;
use pumpkin_core::GameMode;
use pumpkin_entity::EntityId;
use pumpkin_inventory::drag_handler::DragHandler;
use pumpkin_inventory::{Container, OpenContainer};
use pumpkin_protocol::client::login::CEncryptionRequest;
use pumpkin_protocol::client::status::CStatusResponse;
use pumpkin_protocol::{client::config::CPluginMessage, ClientPacket};
use pumpkin_registry::Registry;
use pumpkin_world::dimension::Dimension;
use std::collections::HashMap;
use std::{
    sync::{
        atomic::{AtomicI32, Ordering},
        Arc,
    },
    time::Duration,
};

use crate::client::EncryptionError;
use crate::{
    client::Client,
    commands::{default_dispatcher, dispatcher::CommandDispatcher},
    entity::player::Player,
    world::World,
};

mod connection_cache;
mod key_store;
pub const CURRENT_MC_VERSION: &str = "1.21.1";

pub struct Server {
    key_store: KeyStore,
    server_listing: CachedStatus,
    server_branding: CachedBranding,

    pub command_dispatcher: Arc<CommandDispatcher<'static>>,
    pub worlds: Vec<Arc<World>>,

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
        // TODO: only create when needed

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

        let world = World::load(Dimension::OverWorld.into_level(
            // TODO: load form config
            "./world".parse().unwrap(),
        ));
        Self {
            cached_registry: Registry::get_static(),
            open_containers: RwLock::new(HashMap::new()),
            drag_handler: DragHandler::new(),
            // 0 is invalid
            entity_id: 2.into(),
            worlds: vec![Arc::new(world)],
            command_dispatcher: Arc::new(command_dispatcher),
            auth_client,
            key_store: KeyStore::new(),
            server_listing: CachedStatus::new(),
            server_branding: CachedBranding::new(),
        }
    }

    pub async fn add_player(&self, id: usize, client: Arc<Client>) -> (Arc<Player>, Arc<World>) {
        let entity_id = self.new_entity_id();
        let gamemode = match BASIC_CONFIG.default_gamemode {
            GameMode::Undefined => GameMode::Survival,
            game_mode => game_mode,
        };
        // Basically the default world
        // TODO: select default from config
        let world = &self.worlds[0];

        let player = Arc::new(Player::new(client, world.clone(), entity_id, gamemode));
        world.add_player(id, player.clone());
        (player, world.clone())
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

    /// Sends a message to all players in every world
    pub fn broadcast_message(&self, content: &TextComponent) {
        self.worlds
            .iter()
            .for_each(|w| w.broadcast_message(content));
    }

    /// Get all online player names
    pub fn get_online_player_names(&self) -> Vec<String> {
        self.worlds
            .iter()
            .flat_map(|world| world.get_player_names())
            .collect::<Vec<_>>()
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

    pub fn get_branding(&self) -> CPluginMessage<'_> {
        self.server_branding.get_branding()
    }

    pub fn get_status(&self) -> CStatusResponse<'_> {
        self.server_listing.get_status()
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
}
