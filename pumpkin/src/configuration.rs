use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::server::Difficulty;

#[derive(Deserialize, Serialize)]
pub struct AdvancedConfiguration {
    pub liquid_physics: bool,
    pub encryption: bool,
}

impl Default for AdvancedConfiguration {
    fn default() -> Self {
        Self {
            liquid_physics: true,
            encryption: true,
        }
    }
}

#[derive(Deserialize, Serialize)]
pub struct BasicConfiguration {
    #[serde(default)]
    pub server_address: String,
    #[serde(default)]
    pub server_port: u16,
    #[serde(default)]
    pub seed: String,
    #[serde(default)]
    pub max_players: u32,
    #[serde(default)]
    pub view_distances: u8,
    #[serde(default)]
    pub simulation_distance: u8,
    #[serde(default)]
    pub resource_pack: String,
    #[serde(default)]
    pub resource_pack_sha1: String,
    pub default_difficulty: Difficulty,
    #[serde(default)]
    pub allow_nether: bool,
    #[serde(default)]
    pub hardcore: bool,
    #[serde(default)]
    pub online_mode: bool,
    #[serde(default)]
    pub spawn_protection: u32,
    #[serde(default)]
    pub motd: String,
}

impl Default for BasicConfiguration {
    fn default() -> Self {
        Self {
            server_address: "127.0.0.1".to_string(),
            server_port: 25565,
            seed: "".to_string(),
            max_players: 0, //if max_players is zero it increases every join
            view_distances: 10,
            simulation_distance: 10,
            resource_pack: "".to_string(),
            resource_pack_sha1: "".to_string(),
            default_difficulty: Difficulty::Normal,
            allow_nether: true,
            hardcore: false,
            online_mode: true,
            spawn_protection: 16,
            motd: "A Blazing fast Pumpkin Server!".to_string(),
        }
    }
}

impl AdvancedConfiguration {
    pub fn load<P: AsRef<Path>>(path: P) -> AdvancedConfiguration {
        if path.as_ref().exists() {
            let toml = std::fs::read_to_string(path).expect("Couldn't read configuration");
            let configuration: AdvancedConfiguration =
                toml::from_str(toml.as_str()).expect("Couldn't parse");
            configuration
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
            let configuration: BasicConfiguration =
                toml::from_str(toml.as_str()).expect("Couldn't parse");
            configuration
        } else {
            let config = BasicConfiguration::default();
            let toml = toml::to_string(&config).expect("Couldn't create toml!");

            std::fs::write(path, toml).expect("Couldn't save configuration");
            config
        }
    }
}
