use serde::{Deserialize, Serialize};
use serde_inline_default::serde_inline_default;

#[serde_inline_default]
#[derive(Deserialize, Serialize)]
pub struct LoggingConfig {
    #[serde_inline_default(true)]
    pub enabled: bool,
    #[serde_inline_default(LevelFilter::Info)]
    pub level: LevelFilter,
    #[serde_inline_default(false)]
    pub env: bool,
    #[serde_inline_default(true)]
    pub threads: bool,
    #[serde_inline_default(true)]
    pub color: bool,
    #[serde_inline_default(true)]
    pub timestamp: bool,
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            level: LevelFilter::Info,
            env: false,
            threads: true,
            color: true,
            timestamp: true,
        }
    }
}

#[derive(Deserialize, Serialize, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug, Hash)]
pub enum LevelFilter {
    /// A level lower than all log levels.
    Off,
    /// Corresponds to the `Error` log level.
    Error,
    /// Corresponds to the `Warn` log level.
    Warn,
    /// Corresponds to the `Info` log level.
    Info,
    /// Corresponds to the `Debug` log level.
    Debug,
    /// Corresponds to the `Trace` log level.
    Trace,
}
