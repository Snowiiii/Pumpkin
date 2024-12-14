#[cfg(feature = "schemars")]
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Default)]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
#[serde(default)]
pub struct ProxyConfig {
    pub enabled: bool,
    pub velocity: VelocityConfig,
    pub bungeecord: BungeeCordConfig,
}
#[derive(Deserialize, Serialize, Default)]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
#[serde(default)]
pub struct BungeeCordConfig {
    pub enabled: bool,
}

#[derive(Deserialize, Serialize, Default)]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
#[serde(default)]
pub struct VelocityConfig {
    pub enabled: bool,
    pub secret: String,
}
