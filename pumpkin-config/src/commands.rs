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
