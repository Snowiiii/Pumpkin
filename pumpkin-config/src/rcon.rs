use std::net::{Ipv4Addr, SocketAddr};

use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Clone)]
pub struct RCONConfig {
    /// Is RCON Enabled?
    pub enabled: bool,
    /// The network address and port where the RCON server will listen for connections.
    pub address: SocketAddr,
    /// The password required for RCON authentication.
    pub password: String,
    /// The maximum number of concurrent RCON connections allowed.
    /// If 0 there is no limit
    pub max_connections: u32,
}

impl Default for RCONConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            address: SocketAddr::new(Ipv4Addr::new(0, 0, 0, 0).into(), 25575),
            password: "".to_string(),
            max_connections: 0,
        }
    }
}
