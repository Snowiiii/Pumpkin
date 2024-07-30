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
    pub server_address: String,
    pub server_port: u16,
    pub seed: String,
    pub max_plyers: u32,
    pub view_distances: u8,
    pub simulation_distance: u8,
    pub resource_pack: String,
    pub resource_pack_sha1: String,
    pub default_difficulty: Difficulty,
    pub allow_nether: bool,
    pub hardcore: bool,
    pub online_mode: bool,
    pub spawn_protection: u32,
    pub motd: String,
}

impl Default for BasicConfiguration {
    fn default() -> Self {
        Self {
            server_address: "127.0.0.1".to_string(),
            server_port: 25565,
            seed: "".to_string(),
            max_plyers: 0,
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
