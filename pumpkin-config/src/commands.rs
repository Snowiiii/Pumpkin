use serde::{Deserialize, Serialize};
use serde_inline_default::serde_inline_default;

#[derive(Deserialize, Serialize)]
#[serde_inline_default]
pub struct CommandsConfig {
    /// Are commands from the Console accepted ?
    #[serde_inline_default(true)]
    pub use_console: bool,
    /// Should be commands from players be logged in console?
    #[serde_inline_default(true)]
    pub log_console: bool,
    pub disabled: DisabledCommands,
}

impl Default for CommandsConfig {
    fn default() -> Self {
        Self {
            use_console: true,
            log_console: true,
            disabled: DisabledCommands::default(),
        }
    }
}

#[derive(Deserialize, Serialize, Default)]
pub struct DisabledCommands {
    pub pumpkin: bool,
    pub say: bool,
    pub gamemode: bool,
    pub stop: bool,
    pub help: bool,
    pub echest: bool,
    pub craft: bool,
    pub kill: bool,
    pub kick: bool,
    pub worldborder: bool,
    pub teleport: bool,
    pub give: bool,
    pub list: bool,
    pub clear: bool,
    pub setblock: bool,
    pub transfer: bool,
}
