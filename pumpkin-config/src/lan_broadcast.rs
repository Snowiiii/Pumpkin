use serde::{Deserialize, Serialize};
use serde_inline_default::serde_inline_default;

#[serde_inline_default]
#[derive(Deserialize, Serialize, Default)]
pub struct LANBroadcastConfig {
    #[serde_inline_default(false)]
    pub enabled: bool,
    // We use an extra motd because this only supports one line
    #[serde_inline_default("A Blazing fast Pumpkin Server!".to_string())]
    pub motd: String,
    // No idea why you would want to change this
    #[serde_inline_default(None)]
    pub port: Option<u16>,
}
