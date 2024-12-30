use connection_cache::{CachedBranding, CachedStatus};
use crossbeam::atomic::AtomicCell;
use key_store::KeyStore;
use pumpkin_config::BASIC_CONFIG;
use pumpkin_core::math::boundingbox::{BoundingBox, BoundingBoxSize};
use pumpkin_core::math::position::WorldPosition;
use pumpkin_core::math::vector2::Vector2;
use pumpkin_core::GameMode;
use pumpkin_entity::entity_type::EntityType;
use pumpkin_entity::EntityId;
use pumpkin_inventory::drag_handler::DragHandler;
use pumpkin_inventory::{Container, OpenContainer};
use pumpkin_protocol::client::login::CEncryptionRequest;
use pumpkin_protocol::{client::config::CPluginMessage, ClientPacket};
use pumpkin_registry::{DimensionType, Registry};
use pumpkin_world::block::block_registry::Block;
use pumpkin_world::dimension::Dimension;
use pumpkin_world::entity::entity_registry::get_entity_by_id;
use rand::prelude::SliceRandom;
use std::collections::HashMap;
use std::sync::atomic::AtomicU32;
use std::{
    sync::{
        atomic::{AtomicI32, Ordering},
        Arc,
    },
    time::Duration,
};
use tokio::sync::{Mutex, RwLock};
use uuid::Uuid;

use crate::block::block_manager::BlockManager;
use crate::block::default_block_manager;
use crate::entity::living::LivingEntity;
use crate::entity::Entity;
use crate::net::EncryptionError;
use crate::world::custom_bossbar::CustomBossbars;
use crate::{
    command::{default_dispatcher, dispatcher::CommandDispatcher},
    entity::player::Player,
    net::Client,
    world::World,
};

mod connection_cache;
mod key_store;
pub mod ticker;

pub const CURRENT_MC_VERSION: &str = "1.21.4";

/// Represents a Minecraft server instance.
pub struct Server {
    /// Handles cryptographic keys for secure communication.
    key_store: KeyStore,
    /// Manages server status information.
    server_listing: Mutex<CachedStatus>,
    /// Saves server branding information.
    server_branding: CachedBranding,
    /// Saves and Dispatches commands to appropriate handlers.
    pub command_dispatcher: RwLock<CommandDispatcher>,
    /// Saves and calls blocks blocks
    pub block_manager: Arc<BlockManager>,
    /// Manages multiple worlds within the server.
    pub worlds: Vec<Arc<World>>,
    // All the dimensions that exists on the server,
    pub dimensions: Vec<DimensionType>,
    /// Caches game registries for efficient access.
    pub cached_registry: Vec<Registry>,
    /// Tracks open containers used for item interactions.
    // TODO: should have per player open_containers
    pub open_containers: RwLock<HashMap<u64, OpenContainer>>,
    pub drag_handler: DragHandler,
    /// Assigns unique IDs to entities.
    entity_id: AtomicI32,
    /// Assigns unique IDs to containers.
    container_id: AtomicU32,
    /// Manages authentication with a authentication server, if enabled.
    pub auth_client: Option<reqwest::Client>,
    /// The server's custom bossbars
    pub bossbars: Mutex<CustomBossbars>,
}

impl Server {
    #[allow(clippy::new_without_default)]
    #[must_use]
    pub fn new() -> Self {
        let auth_client = BASIC_CONFIG.online_mode.then(|| {
            reqwest::Client::builder()
                .timeout(Duration::from_millis(5000))
                .build()
                .expect("Failed to to make reqwest client")
        });

        // First register default command, after that plugins can put in their own
        let command_dispatcher = RwLock::new(default_dispatcher());

        let world = World::load(
            Dimension::OverWorld.into_level(
                // TODO: load form config
                "./world".parse().unwrap(),
            ),
            DimensionType::Overworld,
        );

        // Spawn chunks are never unloaded
        for x in -1..=1 {
            for z in -1..=1 {
                world.level.mark_chunk_as_newly_watched(Vector2::new(x, z));
            }
        }

        Self {
            cached_registry: Registry::get_synced(),
            open_containers: RwLock::new(HashMap::new()),
            drag_handler: DragHandler::new(),
            // 0 is invalid
            entity_id: 2.into(),
            container_id: 0.into(),
            worlds: vec![Arc::new(world)],
            dimensions: vec![
                DimensionType::Overworld,
                DimensionType::OverworldCaves,
                DimensionType::TheNether,
                DimensionType::TheEnd,
            ],
            command_dispatcher,
            block_manager: default_block_manager(),
            auth_client,
            key_store: KeyStore::new(),
            server_listing: Mutex::new(CachedStatus::new()),
            server_branding: CachedBranding::new(),
            bossbars: Mutex::new(CustomBossbars::new()),
        }
    }

    /// Adds a new player to the server.
    ///
    /// This function takes an `Arc<Client>` representing the connected client and performs the following actions:
    ///
    /// 1. Generates a new entity ID for the player.
    /// 2. Determines the player's gamemode (defaulting to Survival if not specified in configuration).
    /// 3. **(TODO: Select default from config)** Selects the world for the player (currently uses the first world).
    /// 4. Creates a new `Player` instance using the provided information.
    /// 5. Adds the player to the chosen world.
    /// 6. **(TODO: Config if we want increase online)** Optionally updates server listing information based on player's configuration.
    ///
    /// # Arguments
    ///
    /// * `client`: An `Arc<Client>` representing the connected client.
    ///
    /// # Returns
    ///
    /// A tuple containing:
    ///
    /// - `Arc<Player>`: A reference to the newly created player object.
    /// - `Arc<World>`: A reference to the world the player was added to.
    ///
    /// # Note
    ///
    /// You still have to spawn the Player in the World to make then to let them Join and make them Visible
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
        // TODO: Config if we want decrease online
        self.server_listing.lock().await.remove_player();
    }

    pub async fn save(&self) {
        for world in &self.worlds {
            world.save().await;
        }
    }

    /// Adds a new living entity to the server.
    ///
    /// # Returns
    ///
    /// A tuple containing:
    ///
    /// - `Arc<LivingEntity>`: A reference to the newly created living entity.
    /// - `Arc<World>`: A reference to the world that the living entity was added to.
    /// - `Uuid`: The uuid of the newly created living entity to be used to send to the client.
    pub async fn add_living_entity(
        &self,
        entity_type: EntityType,
    ) -> (Arc<LivingEntity>, Arc<World>, Uuid) {
        let entity_id = self.new_entity_id();
        // TODO: select current
        let world = &self.worlds[0];

        // TODO: this should be resolved to a integer using a macro when calling this function
        let bounding_box_size: BoundingBoxSize;
        if let Some(entity) = get_entity_by_id(entity_type.clone() as u16) {
            bounding_box_size = BoundingBoxSize {
                width: f64::from(entity.dimension[0]),
                height: f64::from(entity.dimension[1]),
            };
        } else {
            bounding_box_size = BoundingBoxSize {
                width: 0.6,
                height: 1.8,
            };
        }

        // TODO: standing eye height should be per mob
        let new_uuid = uuid::Uuid::new_v4();
        let mob = Arc::new(LivingEntity::new(Entity::new(
            entity_id,
            new_uuid,
            world.clone(),
            entity_type,
            1.62,
            AtomicCell::new(BoundingBox::new_default(&bounding_box_size)),
            AtomicCell::new(bounding_box_size),
        )));

        world.add_living_entity(new_uuid, mob.clone()).await;

        (mob, world.clone(), new_uuid)
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

    /// Returns the first id with a matching location and block type. If this is used with unique
    /// blocks, the output will return a random result.
    pub async fn get_container_id(&self, location: WorldPosition, block: Block) -> Option<u32> {
        let open_containers = self.open_containers.read().await;
        // TODO: do better than brute force
        for (id, container) in open_containers.iter() {
            if container.is_location(location) {
                if let Some(container_block) = container.get_block() {
                    if container_block.id == block.id {
                        log::debug!("Found container id: {}", id);
                        return Some(*id as u32);
                    }
                }
            }
        }

        drop(open_containers);

        None
    }

    pub async fn get_all_container_ids(
        &self,
        location: WorldPosition,
        block: Block,
    ) -> Option<Vec<u32>> {
        let open_containers = self.open_containers.read().await;
        let mut matching_container_ids: Vec<u32> = vec![];
        // TODO: do better than brute force
        for (id, container) in open_containers.iter() {
            if container.is_location(location) {
                if let Some(container_block) = container.get_block() {
                    if container_block.id == block.id {
                        log::debug!("Found matching container id: {}", id);
                        matching_container_ids.push(*id as u32);
                    }
                }
            }
        }

        drop(open_containers);

        Some(matching_container_ids)
    }

    /// Broadcasts a packet to all players in all worlds.
    ///
    /// This function sends the specified packet to every connected player in every world managed by the server.
    ///
    /// # Arguments
    ///
    /// * `packet`: A reference to the packet to be broadcast. The packet must implement the `ClientPacket` trait.
    pub async fn broadcast_packet_all<P>(&self, packet: &P)
    where
        P: ClientPacket,
    {
        for world in &self.worlds {
            world.broadcast_packet_all(packet).await;
        }
    }

    /// Searches for a player by their username across all worlds.
    ///
    /// This function iterates through each world managed by the server and attempts to find a player with the specified username.
    /// If a player is found in any world, it returns an `Arc<Player>` reference to that player. Otherwise, it returns `None`.
    ///
    /// # Arguments
    ///
    /// * `name`: The username of the player to search for.
    ///
    /// # Returns
    ///
    /// An `Option<Arc<Player>>` containing the player if found, or `None` if not found.
    pub async fn get_player_by_name(&self, name: &str) -> Option<Arc<Player>> {
        for world in &self.worlds {
            if let Some(player) = world.get_player_by_name(name).await {
                return Some(player);
            }
        }
        None
    }

    /// Returns all players from all worlds.
    pub async fn get_all_players(&self) -> Vec<Arc<Player>> {
        let mut players = Vec::<Arc<Player>>::new();

        for world in &self.worlds {
            for (_, player) in world.current_players.lock().await.iter() {
                players.push(player.clone());
            }
        }

        players
    }

    /// Returns a random player from any of the worlds or None if all worlds are empty.
    pub async fn get_random_player(&self) -> Option<Arc<Player>> {
        let players = self.get_all_players().await;

        players.choose(&mut rand::thread_rng()).map(Arc::<_>::clone)
    }

    /// Searches for a player by their UUID across all worlds.
    ///
    /// This function iterates through each world managed by the server and attempts to find a player with the specified UUID.
    /// If a player is found in any world, it returns an `Arc<Player>` reference to that player. Otherwise, it returns `None`.
    ///
    /// # Arguments
    ///
    /// * `id`: The UUID of the player to search for.
    ///
    /// # Returns
    ///
    /// An `Option<Arc<Player>>` containing the player if found, or `None` if not found.
    pub async fn get_player_by_uuid(&self, id: uuid::Uuid) -> Option<Arc<Player>> {
        for world in &self.worlds {
            if let Some(player) = world.get_player_by_uuid(id).await {
                return Some(player);
            }
        }
        None
    }

    /// Counts the total number of players across all worlds.
    ///
    /// This function iterates through each world and sums up the number of players currently connected to that world.
    ///
    /// # Returns
    ///
    /// The total number of players connected to the server.
    pub async fn get_player_count(&self) -> usize {
        let mut count = 0;
        for world in &self.worlds {
            count += world.current_players.lock().await.len();
        }
        count
    }

    /// Similar to [`Server::get_player_count`] >= n, but may be more efficient since it stops it's iteration through all worlds as soon as n players were found.
    pub async fn has_n_players(&self, n: usize) -> bool {
        let mut count = 0;
        for world in &self.worlds {
            count += world.current_players.lock().await.len();
            if count >= n {
                return true;
            }
        }
        false
    }

    /// Generates a new entity id
    /// This should be global
    pub fn new_entity_id(&self) -> EntityId {
        self.entity_id.fetch_add(1, Ordering::SeqCst)
    }

    /// Generates a new container id
    pub fn new_container_id(&self) -> u32 {
        self.container_id.fetch_add(1, Ordering::SeqCst)
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
    ) -> CEncryptionRequest<'a> {
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
