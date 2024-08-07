use std::path::Path;

use auth_config::Authentication;
use serde::{Deserialize, Serialize};

use crate::{entity::player::GameMode, server::Difficulty};

pub mod auth_config;

/// Current Config version of the Base Config
const CURRENT_BASE_VERSION: &str = "1.0.0";

#[derive(Deserialize, Serialize)]
/// The idea is that Pumpkin should very customizable, You can Enable or Disable Features depning on your needs.
/// This also allows you get some Performance or Resource boosts.
/// Important: The Configuration should match Vanilla by default
pub struct AdvancedConfiguration {
    pub commands: Commands,
    pub authentication: Authentication,
}

#[derive(Deserialize, Serialize)]
pub struct Commands {
    /// Are commands from the Console accepted ?
    pub use_console: bool,
    // todo commands...
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
    /// The path to the resource pack.
    pub resource_pack: String,
    /// The SHA1 hash of the resource pack.
    pub resource_pack_sha1: String,
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
            server_address: "127.0.0.1".to_string(),
            server_port: 25565,
            seed: "".to_string(),
            max_players: 100000,
            view_distance: 10,
            simulation_distance: 10,
            resource_pack: "".to_string(),
            resource_pack_sha1: "".to_string(),
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
            toml::from_str(toml.as_str()).expect("Couldn't parse, Proberbly old config")
        } else {
            let config = AdvancedConfiguration::default();
            let toml = toml::to_string(&config).expect("Couldn't create toml!");
            std::fs::write(path, toml).expect("Couldn't save configuration");
            config
        }
    }
}

impl BasicConfiguration {
    pub fn load<P: AsRef<Path>>(path: P) -> BasicConfiguration {
        if path.as_ref().exists() {
            let toml = std::fs::read_to_string(path).expect("Couldn't read configuration");
            toml::from_str(toml.as_str()).expect("Couldn't parse")
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
        assert_eq!(
            !self.resource_pack.is_empty(),
            !self.resource_pack_sha1.is_empty(),
            "Resource Pack path or Sha1 hash is missing"
        );
    }
}
