use pumpkin_core::PermissionLvl;
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
    /// The op permission level of everyone that is not in the ops file
    pub default_op_level: PermissionLvl,
}

impl Default for CommandsConfig {
    fn default() -> Self {
        Self {
            use_console: true,
            log_console: true,
            default_op_level: PermissionLvl::Zero,
        }
    }
}
