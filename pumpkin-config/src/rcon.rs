use std::net::{Ipv4Addr, SocketAddr};

use serde::{Deserialize, Serialize};
use serde_inline_default::serde_inline_default;

#[serde_inline_default]
#[derive(Deserialize, Serialize, Clone)]
pub struct RCONConfig {
    /// Is RCON Enabled?
    #[serde_inline_default(false)]
    pub enabled: bool,
    /// The network address and port where the RCON server will listen for connections.
    #[serde(default = "default_rcon_address")]
    pub address: SocketAddr,
    /// The password required for RCON authentication.
    #[serde(default)]
    pub password: String,
    /// The maximum number of concurrent RCON connections allowed.
    /// If 0 there is no limit
    #[serde(default)]
    pub max_connections: u32,
}

fn default_rcon_address() -> SocketAddr {
    SocketAddr::new(Ipv4Addr::new(0, 0, 0, 0).into(), 25575)
}

impl Default for RCONConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            address: default_rcon_address(),
            password: "".to_string(),
            max_connections: 0,
        }
    }
}
