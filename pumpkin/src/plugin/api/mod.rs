pub mod context;
pub mod events;
pub mod types;

use async_trait::async_trait;
pub use context::*;
pub use events::*;

#[derive(Debug, Clone)]
pub struct PluginMetadata<'s> {
    /// The name of the plugin.
    pub name: &'s str,
    /// The version of the plugin.
    pub version: &'s str,
    /// The authors of the plugin.
    pub authors: &'s str,
    /// A description of the plugin.
    pub description: &'s str,
}

#[async_trait]
pub trait PluginMethods: Send + Sync + 'static {
    /// Called when the plugin is loaded.
    async fn on_load(&mut self, _server: &Context) -> Result<(), String> {
        Ok(())
    }

    /// Called when the plugin is unloaded.
    async fn on_unload(&mut self, _server: &Context) -> Result<(), String> {
        Ok(())
    }
}

pub trait Plugin: PluginMethods + Hooks {}
