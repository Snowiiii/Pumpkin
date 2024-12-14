use serde::{Deserialize, Serialize};
use std::net::{Ipv4Addr, SocketAddr};
#[cfg(feature = "schemars")]
use schemars::JsonSchema;

#[derive(Deserialize, Serialize, Clone)]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
#[serde(default)]
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
    /// RCON Logging
    pub logging: RCONLogging,
}

impl Default for RCONConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            address: SocketAddr::new(Ipv4Addr::new(0, 0, 0, 0).into(), 25575),
            password: "".to_string(),
            max_connections: 0,
            logging: Default::default(),
        }
    }
}

#[derive(Deserialize, Serialize, Clone, Debug)]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
#[serde(default)]
pub struct RCONLogging {
    /// Whether successful RCON logins should be logged.
    pub log_logged_successfully: bool,
    /// Whether failed RCON login attempts with incorrect passwords should be logged.
    pub log_wrong_password: bool,
    /// Whether all RCON commands, regardless of success or failure, should be logged.
    pub log_commands: bool,
    /// Whether RCON quit commands should be logged.
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
