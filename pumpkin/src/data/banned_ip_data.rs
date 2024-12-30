use std::{net::IpAddr, path::Path, sync::LazyLock};

use serde::{Deserialize, Serialize};

use super::{banlist_serializer::BannedIpEntry, LoadJSONConfiguration, SaveJSONConfiguration};

pub static BANNED_IP_LIST: LazyLock<tokio::sync::RwLock<BannedIpList>> =
    LazyLock::new(|| tokio::sync::RwLock::new(BannedIpList::load()));

#[derive(Deserialize, Serialize, Default)]
#[serde(transparent)]
pub struct BannedIpList {
    pub banned_ips: Vec<BannedIpEntry>,
}

impl BannedIpList {
    #[must_use]
    pub fn is_banned(&self, ip: &IpAddr) -> bool {
        self.banned_ips.iter().any(|entry| &entry.ip == ip)
    }
}

impl LoadJSONConfiguration for BannedIpList {
    fn get_path() -> &'static Path {
        Path::new("banned-ips.json")
    }
    fn validate(&self) {
        // TODO: Validate the list
    }
}

impl SaveJSONConfiguration for BannedIpList {}
