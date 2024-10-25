use connection_cache::{CachedBranding, CachedStatus};
use key_store::KeyStore;
use pumpkin_config::BASIC_CONFIG;
use pumpkin_core::GameMode;
use pumpkin_entity::EntityId;
use pumpkin_inventory::drag_handler::DragHandler;
use pumpkin_inventory::{Container, OpenContainer};
use pumpkin_protocol::client::login::CEncryptionRequest;
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
use tokio::sync::Mutex;
use tokio::sync::RwLock;

use crate::client::EncryptionError;
use crate::{
    client::Client,
    commands::{default_dispatcher, dispatcher::CommandDispatcher},
    entity::player::Player,
    world::World,
};

mod connection_cache;
mod key_store;
pub mod ticker;

pub const CURRENT_MC_VERSION: &str = "1.21.3";

pub struct Server {
    key_store: KeyStore,
    server_listing: Mutex<CachedStatus>,
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
    #[must_use]
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
            command_dispatcher,
            auth_client,
            key_store: KeyStore::new(),
            server_listing: Mutex::new(CachedStatus::new()),
            server_branding: CachedBranding::new(),
        }
    }

    pub async fn add_player(&self, client: Arc<Client>) -> (Arc<Player>, Arc<World>) {
        let entity_id = self.new_entity_id();
        let gamemode = match BASIC_CONFIG.default_gamemode {
            GameMode::Undefined => GameMode::Survival,
            game_mode => game_mode,
        };
        // Basically the default world
        // TODO: select default from config
        let world = &self.worlds[0];

        let player = Arc::new(Player::new(client, world.clone(), entity_id, gamemode).await);
        world
            .add_player(player.gameprofile.id, player.clone())
            .await;
        // TODO: Config if we want increase online
        if let Some(config) = player.client.config.lock().await.as_ref() {
            // TODO: Config so we can also just ignore this hehe
            if config.server_listing {
                self.server_listing.lock().await.add_player();
            }
        }
        (player, world.clone())
    }

    pub async fn remove_player(&self) {
        // TODO: Config if we want increase online
        self.server_listing.lock().await.remove_player();
    }

    pub async fn try_get_container(
        &self,
        player_id: EntityId,
        container_id: u64,
    ) -> Option<Arc<Mutex<Box<dyn Container>>>> {
        let open_containers = self.open_containers.read().await;
        open_containers
            .get(&container_id)?
            .try_open(player_id)
            .cloned()
    }

    /// Sends a Packet to all Players in all worlds
    pub async fn broadcast_packet_all<P>(&self, packet: &P)
    where
        P: ClientPacket,
    {
        for world in &self.worlds {
            world.broadcast_packet_all(packet).await;
        }
    }

    /// Searches every world for a player by username
    pub async fn get_player_by_name(&self, name: &str) -> Option<Arc<Player>> {
        for world in &self.worlds {
            if let Some(player) = world.get_player_by_name(name).await {
                return Some(player);
            }
        }
        None
    }

    /// Searches every world for a player by UUID
    pub async fn get_player_by_uuid(&self, id: uuid::Uuid) -> Option<Arc<Player>> {
        for world in &self.worlds {
            if let Some(player) = world.get_player_by_uuid(id).await {
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

    pub fn get_status(&self) -> &Mutex<CachedStatus> {
        &self.server_listing
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

    async fn tick(&self) {
        for world in &self.worlds {
            world.tick().await;
        }
    }
}
