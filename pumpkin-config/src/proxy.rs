use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Default)]
pub struct ProxyConfig {
    pub enabled: bool,
    pub velocity: VelocityConfig,
    pub bungeecord: BungeeCordConfig,
}
#[derive(Deserialize, Serialize, Default)]
pub struct BungeeCordConfig {
    pub enabled: bool,
}

#[derive(Deserialize, Serialize, Default)]
pub struct VelocityConfig {
    pub enabled: bool,
    pub secret: String,
}
