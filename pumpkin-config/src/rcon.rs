use std::net::{Ipv4Addr, SocketAddr};

use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Clone)]
pub struct RCONConfig {
    pub enabled: bool,
    pub address: SocketAddr,
    pub password: String,
}

impl Default for RCONConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            address: SocketAddr::new(Ipv4Addr::new(0, 0, 0, 0).into(), 25575),
            password: "".to_string(),
        }
    }
}
