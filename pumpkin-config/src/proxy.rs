use serde::{Deserialize, Serialize};
use serde_inline_default::serde_inline_default;

#[serde_inline_default]
#[derive(Deserialize, Serialize, Default)]
#[serde(default)]
pub struct ProxyConfig {
    #[serde_inline_default(false)]
    pub enabled: bool,
    pub velocity: VelocityConfig,
    pub bungeecord: BungeeCordConfig,
}

#[serde_inline_default]
#[derive(Deserialize, Serialize, Default)]
#[serde(default)]
pub struct BungeeCordConfig {
    #[serde_inline_default(false)]
    pub enabled: bool,
}

#[serde_inline_default]
#[derive(Deserialize, Serialize)]
#[serde(default)]
pub struct VelocityConfig {
    #[serde_inline_default(false)]
    pub enabled: bool,
    #[serde_inline_default("".to_string())]
    pub secret: String,
}

impl Default for VelocityConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            secret: "".into(),
        }
    }
}
