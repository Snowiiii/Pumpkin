use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
pub struct LoggingConfig {
    pub enabled: bool,
    pub level: LevelFilter,
    pub env: bool,
    pub threads: bool,
    pub color: bool,
    pub timestamp: bool,
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            level: Default::default(),
            env: false,
            threads: true,
            color: true,
            timestamp: true,
        }
    }
}

#[derive(
    Deserialize, Serialize, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug, Hash,
)]
pub enum LevelFilter {
    Off,
    Error,
    Warn,
    #[default]
    Info,
    Debug,
    Trace,
}
