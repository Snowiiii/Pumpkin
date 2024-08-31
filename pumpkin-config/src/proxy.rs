use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Default)]
pub struct ProxyConfig {
    pub enabled: bool,
    pub velocity: VelocityConfig,
}

#[derive(Deserialize, Serialize)]
pub struct VelocityConfig {
    pub enabled: bool,
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
