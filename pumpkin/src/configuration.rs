use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::server::Difficulty;

/// Current Config version of the Base Config
const CURRENT_BASE_VERSION: &str = "1.0.0";

#[derive(Deserialize, Serialize)]
pub struct AdvancedConfiguration {
    pub liquid_physics: bool,
}

impl Default for AdvancedConfiguration {
    fn default() -> Self {
        Self {
            liquid_physics: true,
        }
    }
}

#[derive(Deserialize, Serialize)]
pub struct BasicConfiguration {
    /// DO NOT CHANGE
    /// Generated to see if config may have been updated with an new Update
    pub config_version: String,
    /// The Server Address to bind
    pub server_address: String,
    /// The Server Port to bind
    /// Minecraft java editon uses 25565 by default
    pub server_port: u16,
    /// The World sent to the Client
    pub seed: String,
    /// The Maximum amout of player which can join the Server
    pub max_players: u32,
    /// The Maximum view distance
    pub view_distances: u8,
    /// The Maximum simulated view distance
    pub simulation_distance: u8,
    /// Path for resource pack
    pub resource_pack: String,
    /// Sha1 hash of resource pack, when present
    pub resource_pack_sha1: String,
    /// default difficulty
    pub default_difficulty: Difficulty,
    /// allow nether dimension
    pub allow_nether: bool,
    /// is the server hardcore mode?
    pub hardcore: bool,
    /// Online Mode, Require a valid Minecraft account to join the Server, Also adds support for Skins and Capes
    /// IMPORTANT: Be carefull when turning this off, Everyone could join your server and use any Nickname or UUID they want
    pub online_mode: bool,
    /// Enable encryption for Packets send & received.
    /// IMPORTANT: When Online mode is enabled, encryption MUST be enabled also
    pub encryption: bool,
    /// When enabled, Client can't use a diffrent ip to login as they use for the Authentication Server
    /// Usally preventing proxy connections
    pub prevent_proxy_connections: bool,
    /// The description send when Client performs a Status request, (e.g Multiplayer Screen)
    pub motd: String,
}

impl Default for BasicConfiguration {
    fn default() -> Self {
        Self {
            config_version: CURRENT_BASE_VERSION.to_string(),
            server_address: "127.0.0.1".to_string(),
            server_port: 25565,
            seed: "".to_string(),
            max_players: 0,
            view_distances: 10,
            simulation_distance: 10,
            resource_pack: "".to_string(),
            resource_pack_sha1: "".to_string(),
            default_difficulty: Difficulty::Normal,
            allow_nether: true,
            hardcore: false,
            online_mode: true,
            encryption: true,
            prevent_proxy_connections: true,
            motd: "A Blazing fast Pumpkin Server!".to_string(),
        }
    }
}

impl AdvancedConfiguration {
    pub fn load<P: AsRef<Path>>(path: P) -> AdvancedConfiguration {
        if path.as_ref().exists() {
            let toml = std::fs::read_to_string(path).expect("Couldn't read configuration");
            toml::from_str(toml.as_str()).expect("Couldn't parse")
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
        assert!(self.view_distances >= 2, "View distance must be atleast 2");
        assert!(
            self.view_distances <= 32,
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
