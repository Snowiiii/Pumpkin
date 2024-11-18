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
    pub enabled: EnabledCommands,
}

impl Default for CommandsConfig {
    fn default() -> Self {
        Self {
            use_console: true,
            log_console: true,
            enabled: EnabledCommands::default(),
        }
    }
}

#[derive(Deserialize, Serialize)]
#[serde(default)]
pub struct EnabledCommands {
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

impl Default for EnabledCommands {
    fn default() -> Self {
        Self {
            pumpkin: true,
            say: true,
            gamemode: true,
            stop: true,
            help: true,
            echest: true,
            craft: true,
            kill: true,
            kick: true,
            worldborder: true,
            teleport: true,
            give: true,
            list: true,
            clear: true,
            setblock: true,
            transfer: true,
        }
    }
}
