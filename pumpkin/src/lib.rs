use std::sync::LazyLock;

use plugin::PluginManager;
use pumpkin_core::text::TextComponent;
use tokio::sync::Mutex;

pub mod block;
pub mod command;
pub mod entity;
pub mod error;
pub mod net;
pub mod plugin;
pub mod server;
pub mod world;

const GIT_VERSION: &str = env!("GIT_VERSION");

pub static PLUGIN_MANAGER: LazyLock<Mutex<PluginManager>> =
    LazyLock::new(|| Mutex::new(PluginManager::new()));
