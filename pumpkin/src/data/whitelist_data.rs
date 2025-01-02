use std::{path::Path, sync::LazyLock};

use pumpkin_config::player_profile;
use serde::{Deserialize, Serialize};

use super::{LoadJSONConfiguration, ReloadJSONConfiguration, SaveJSONConfiguration};

pub static WHITELIST_CONFIG: LazyLock<tokio::sync::RwLock<WhitelistConfig>> =
    LazyLock::new(|| tokio::sync::RwLock::new(WhitelistConfig::load()));

#[derive(Deserialize, Serialize, Default)]
#[serde(transparent)]
pub struct WhitelistConfig {
    pub whitelist: Vec<player_profile::PlayerProfile>,
}

impl LoadJSONConfiguration for WhitelistConfig {
    fn get_path() -> &'static Path {
        Path::new("whitelist.json")
    }
    fn validate(&self) {
        // TODO: Validate the whitelist configuration
    }
}

impl SaveJSONConfiguration for WhitelistConfig {}

impl ReloadJSONConfiguration for WhitelistConfig {}
