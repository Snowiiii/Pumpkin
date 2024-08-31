use log::warn;
use serde::{de::DeserializeOwned, Deserialize, Serialize};

use std::{
    fs,
    net::{Ipv4Addr, SocketAddr},
    path::Path,
    sync::LazyLock,
};

use crate::{entity::player::GameMode, server::Difficulty};

pub mod auth;
pub mod proxy;
pub mod resource_pack;

pub use auth::AuthenticationConfig;
pub use commands::CommandsConfig;
pub use compression::CompressionConfig;
pub use pvp::PVPConfig;
pub use rcon::RCONConfig;

mod commands;
mod compression;
mod pvp;
mod rcon;

use proxy::ProxyConfig;
use resource_pack::ResourcePackConfig;

/// Current Config version of the Base Config
const CURRENT_BASE_VERSION: &str = "1.0.0";

pub static ADVANCED_CONFIG: LazyLock<AdvancedConfiguration> =
    LazyLock::new(AdvancedConfiguration::load);

pub static BASIC_CONFIG: LazyLock<BasicConfiguration> = LazyLock::new(BasicConfiguration::load);

/// The idea is that Pumpkin should very customizable.
/// You can Enable or Disable Features depending on your needs.
///
/// This also allows you get some Performance or Resource boosts.
/// Important: The Configuration should match Vanilla by default
#[derive(Deserialize, Serialize, Default)]
pub struct AdvancedConfiguration {
    pub proxy: ProxyConfig,
    pub authentication: AuthenticationConfig,
    pub packet_compression: CompressionConfig,
    pub resource_pack: ResourcePackConfig,
    pub commands: CommandsConfig,
    pub rcon: RCONConfig,
    pub pvp: PVPConfig,
}

#[derive(Serialize, Deserialize)]
pub struct BasicConfiguration {
    /// A version identifier for the configuration format.
    pub config_version: String,
    /// The address to bind the server to.
    pub server_address: SocketAddr,
    /// The seed for world generation.
    pub seed: String,
    /// The maximum number of players allowed on the server.
    pub max_players: u32,
    /// The maximum view distance for players.
    pub view_distance: u8,
    /// The maximum simulated view distance.
    pub simulation_distance: u8,
    /// The default game difficulty.
    pub default_difficulty: Difficulty,
    /// Whether the Nether dimension is enabled.
    pub allow_nether: bool,
    /// Whether the server is in hardcore mode.
    pub hardcore: bool,
    /// Whether online mode is enabled. Requires valid Minecraft accounts.
    pub online_mode: bool,
    /// Whether packet encryption is enabled. Required when online mode is enabled.
    pub encryption: bool,
    /// The server's description displayed on the status screen.
    pub motd: String,
    /// The default game mode for players.
    pub default_gamemode: GameMode,
}

impl Default for BasicConfiguration {
    fn default() -> Self {
        Self {
            config_version: CURRENT_BASE_VERSION.to_string(),
            server_address: SocketAddr::new(Ipv4Addr::new(0, 0, 0, 0).into(), 25565),
            seed: "".to_string(),
            max_players: 100000,
            view_distance: 10,
            simulation_distance: 10,
            default_difficulty: Difficulty::Normal,
            allow_nether: true,
            hardcore: false,
            online_mode: true,
            encryption: true,
            motd: "A Blazing fast Pumpkin Server!".to_string(),
            default_gamemode: GameMode::Survival,
        }
    }
}

trait LoadConfiguration {
    fn load() -> Self
    where
        Self: Sized + Default + Serialize + DeserializeOwned,
    {
        let path = Self::get_path();

        let config = if path.exists() {
            let file_content = fs::read_to_string(path)
                .unwrap_or_else(|_| panic!("Couldn't read configuration file at {:?}", path));

            toml::from_str(&file_content).unwrap_or_else(|err| {
                panic!(
                    "Couldn't parse config at {:?}. Reason: {}",
                    path,
                    err.message()
                )
            })
        } else {
            let content = Self::default();

            if let Err(err) = fs::write(path, toml::to_string(&content).unwrap()) {
                warn!(
                    "Couldn't write default config to {:?}. Reason: {}",
                    path, err
                );
            }

            content
        };

        config.validate();
        config
    }

    fn get_path() -> &'static Path;

    fn validate(&self);
}

impl LoadConfiguration for AdvancedConfiguration {
    fn get_path() -> &'static Path {
        Path::new("features.toml")
    }

    fn validate(&self) {
        self.resource_pack.validate()
    }
}

impl LoadConfiguration for BasicConfiguration {
    fn get_path() -> &'static Path {
        Path::new("configuration.toml")
    }

    fn validate(&self) {
        assert_eq!(
            self.config_version, CURRENT_BASE_VERSION,
            "Config version does not match used Config version. Please update your config"
        );
        assert!(self.view_distance >= 2, "View distance must be at least 2");
        assert!(
            self.view_distance <= 32,
            "View distance must be less than 32"
        );
        if self.online_mode {
            assert!(
                self.encryption,
                "When Online Mode is enabled, Encryption must be enabled"
            )
        }
    }
}
