#[cfg(feature = "schemars")]
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
#[serde(default)]
pub struct CommandsConfig {
    /// Are commands from the Console accepted ?
    pub use_console: bool,
    /// Should be commands from players be logged in console?
    pub log_console: bool, // TODO: commands...
}

impl Default for CommandsConfig {
    fn default() -> Self {
        Self {
            use_console: true,
            log_console: true,
        }
    }
}
