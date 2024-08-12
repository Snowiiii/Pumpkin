use std::path::Path;

use auth_config::Authentication;
use resource_pack::ResourcePack;
use serde::{Deserialize, Serialize};

use crate::{entity::player::GameMode, server::Difficulty};

pub mod auth_config;
pub mod resource_pack;

/// Current Config version of the Base Config
const CURRENT_BASE_VERSION: &str = "1.0.0";

#[derive(Deserialize, Serialize)]
/// The idea is that Pumpkin should very customizable, You can Enable or Disable Features depning on your needs.
/// This also allows you get some Performance or Resource boosts.
/// Important: The Configuration should match Vanilla by default
pub struct AdvancedConfiguration {
    pub commands: Commands,
    pub authentication: Authentication,
    pub packet_compression: Compression,
    pub resource_pack: ResourcePack,
    pub rcon: RCONConfig,
}

#[derive(Deserialize, Serialize, Clone)]
pub struct RCONConfig {
    pub enabled: bool,
    pub ip: String,
    pub port: u16,
    pub password: String,
}

impl Default for RCONConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            ip: "0.0.0.0".to_string(),
            port: 25575,
            password: "".to_string(),
        }
    }
}

#[derive(Deserialize, Serialize)]
pub struct Commands {
    /// Are commands from the Console accepted ?
    pub use_console: bool,
    // TODO: commands...
}

#[derive(Deserialize, Serialize)]
// Packet compression
pub struct Compression {
    // Is compression enabled ?
    pub enabled: bool,
    // The compression threshold used when compression is enabled
    pub compression_threshold: u32,
    // A value between 0..9
    // 1 = Optimize for the best speed of encoding.
    // 9 = Optimize for the size of data being encoded.
    pub compression_level: u32,
}

impl Default for Compression {
    fn default() -> Self {
        Self {
            enabled: true,
            compression_threshold: 256,
            compression_level: 4,
        }
    }
}

impl Default for Commands {
    fn default() -> Self {
        Self { use_console: true }
    }
}

/// Important: The Configuration should match Vanilla by default
impl Default for AdvancedConfiguration {
    fn default() -> Self {
        Self {
            authentication: Authentication::default(),
            commands: Commands::default(),
            packet_compression: Compression::default(),
            resource_pack: ResourcePack::default(),
            rcon: RCONConfig::default(),
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct BasicConfiguration {
    /// A version identifier for the configuration format.
    pub config_version: String,
    /// The address to bind the server to.
    pub server_address: String,
    /// The port to listen on.
    pub server_port: u16,
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
            server_address: "0.0.0.0".to_string(),
            server_port: 25565,
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

impl AdvancedConfiguration {
    pub fn load<P: AsRef<Path>>(path: P) -> AdvancedConfiguration {
        if path.as_ref().exists() {
            let toml = std::fs::read_to_string(path).expect("Couldn't read configuration");
            let config: AdvancedConfiguration =
                toml::from_str(toml.as_str()).expect("Couldn't parse features.toml, Proberbly old config, Replacing with a new one or just delete it");
            config.validate();
            config
        } else {
            let config = AdvancedConfiguration::default();
            let toml = toml::to_string(&config).expect("Couldn't create toml!");
            std::fs::write(path, toml).expect("Couldn't save configuration");
            config.validate();
            config
        }
    }
    pub fn validate(&self) {
        self.resource_pack.validate()
    }
}

impl BasicConfiguration {
    pub fn load<P: AsRef<Path>>(path: P) -> BasicConfiguration {
        if path.as_ref().exists() {
            let toml = std::fs::read_to_string(path).expect("Couldn't read configuration");
            let config: BasicConfiguration = toml::from_str(toml.as_str()).expect("Couldn't parse configuration.toml, Proberbly old config, Replacing with a new one or just delete it");
            config.validate();
            config
        } else {
            let config = BasicConfiguration::default();
            let toml = toml::to_string(&config).expect("Couldn't create toml!");
            std::fs::write(path, toml).expect("Couldn't save configuration");
            config.validate();
            config
        }
    }

    pub fn validate(&self) {
        assert_eq!(
            self.config_version, CURRENT_BASE_VERSION,
            "Config version does not match used Config version. Please update your config"
        );
        assert!(self.view_distance >= 2, "View distance must be atleast 2");
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
