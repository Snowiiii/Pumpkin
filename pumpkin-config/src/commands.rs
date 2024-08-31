use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
pub struct CommandsConfig {
    /// Are commands from the Console accepted ?
    pub use_console: bool,
    // TODO: commands...
}

impl Default for CommandsConfig {
    fn default() -> Self {
        Self { use_console: true }
    }
}
