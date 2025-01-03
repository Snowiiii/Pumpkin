use std::{path::Path, sync::LazyLock};

use serde::{Deserialize, Serialize};

use crate::net::GameProfile;

use super::{banlist_serializer::BannedPlayerEntry, LoadJSONConfiguration, SaveJSONConfiguration};

pub static BANNED_PLAYER_LIST: LazyLock<tokio::sync::RwLock<BannedPlayerList>> =
    LazyLock::new(|| tokio::sync::RwLock::new(BannedPlayerList::load()));

#[derive(Deserialize, Serialize, Default)]
#[serde(transparent)]
pub struct BannedPlayerList {
    pub banned_players: Vec<BannedPlayerEntry>,
}

impl BannedPlayerList {
    #[must_use]
    pub fn is_banned(&self, profile: &GameProfile) -> bool {
        self.banned_players
            .iter()
            .any(|entry| entry.name == profile.name && entry.uuid == profile.id)
    }
}

impl LoadJSONConfiguration for BannedPlayerList {
    fn get_path() -> &'static Path {
        Path::new("banned-players.json")
    }
    fn validate(&self) {
        // TODO: Validate the list
    }
}

impl SaveJSONConfiguration for BannedPlayerList {}
