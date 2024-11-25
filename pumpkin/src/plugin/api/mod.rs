pub mod context;
pub mod events;

pub use context::*;
pub use events::*;

#[derive(Debug, Clone)]
pub struct PluginMetadata<'s> {
    /// The name of the plugin.
    pub name: &'s str,
    /// The unique identifier of the plugin.
    pub id: &'s str,
    /// The version of the plugin.
    pub version: &'s str,
    /// The authors of the plugin.
    pub authors: &'s [&'s str],
    /// A description of the plugin.
    pub description: Option<&'s str>,
}

pub trait Plugin: Send + Sync + 'static {
    /// Called when the plugin is loaded.
    fn on_load(&mut self, _server: &dyn PluginContext) -> Result<(), String> {
        Ok(())
    }

    /// Called when the plugin is unloaded.
    fn on_unload(&mut self, _server: &dyn PluginContext) -> Result<(), String> {
        Ok(())
    }
}

#[macro_export]
macro_rules! plugin_metadata {
    ($name:expr, $id:expr, $version:expr, $authors:expr, $description:expr) => {
        #[no_mangle]
        pub static METADATA: pumpkin::plugin::PluginMetadata = pumpkin::plugin::PluginMetadata {
            name: $name,
            id: $id,
            version: $version,
            authors: $authors,
            description: Some($description),
        };
    };
}
