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
    /// RCON Logging
    pub logging: RCONLogging,
}

#[serde_inline_default]
#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct RCONLogging {
    /// Whether successful RCON logins should be logged.
    #[serde_inline_default(true)]
    pub log_logged_successfully: bool,
    /// Whether failed RCON login attempts with incorrect passwords should be logged.
    #[serde_inline_default(true)]
    pub log_wrong_password: bool,
    /// Whether all RCON commands, regardless of success or failure, should be logged.
    #[serde_inline_default(true)]
    pub log_commands: bool,
    /// Whether RCON quit commands should be logged.
    #[serde_inline_default(true)]
    pub log_quit: bool,
}

impl Default for RCONLogging {
    fn default() -> Self {
        Self {
            log_logged_successfully: true,
            log_wrong_password: true,
            log_commands: true,
            log_quit: true,
        }
    }
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
            logging: Default::default(),
        }
    }
}
