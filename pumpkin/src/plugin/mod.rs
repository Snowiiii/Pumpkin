pub mod api;

pub use api::*;
use std::{collections::HashMap, fs, path::Path};

use crate::{entity::player::Player, world::World};

pub struct PluginManager<'s> {
    plugins: HashMap<String, (PluginMetadata<'s>, Box<dyn Plugin>, Box<dyn Hooks>, libloading::Library)>,
}

impl Default for PluginManager<'_> {
    fn default() -> Self {
        Self::new()
    }
}

struct PluginLogger {
    plugin_name: String,
}

impl Logger for PluginLogger {
    fn info(&self, message: &str) {
        log::info!("[{}] {}", self.plugin_name, message);
    }

    fn warn(&self, message: &str) {
        log::warn!("[{}] {}", self.plugin_name, message);
    }

    fn error(&self, message: &str) {
        log::error!("[{}] {}", self.plugin_name, message);
    }
}

struct Context<'a> {
    metadata: &'a PluginMetadata<'a>,
}
impl PluginContext for Context<'_> {
    fn get_logger(&self) -> Box<dyn Logger> {
        Box::new(PluginLogger {
            plugin_name: self.metadata.name.to_string(),
        })
    }
}

struct Event<'a> {
    player: &'a Player,
    world: &'a World,
}

impl PlayerConnectionEvent for Event<'_> {
    fn get_player(&self) -> &Player {
        self.player
    }

    fn get_world(&self) -> &World {
        self.world
    }
}

impl PluginManager<'_> {
    #[must_use]
    pub fn new() -> Self {
        PluginManager {
            plugins: HashMap::new(),
        }
    }

    pub fn load_plugins(&mut self) -> Result<(), String> {
        const PLUGIN_DIR: &str = "./plugins";

        let dir_entires = fs::read_dir(PLUGIN_DIR);

        for entry in dir_entires.unwrap() {
            if !entry.as_ref().unwrap().path().is_file() {
                continue;
            }
            self.try_load_plugin(entry.unwrap().path().as_path());
        }

        Ok(())
    }

    fn try_load_plugin(&mut self, path: &Path) {
        let library = unsafe { libloading::Library::new(path).unwrap() };

        let plugin_fn = unsafe { library.get::<fn() -> Box<dyn Plugin>>(b"plugin").unwrap() };
        let hooks_fn = unsafe { library.get::<fn() -> Box<dyn Hooks>>(b"hooks").unwrap() };
        let metadata: &PluginMetadata =
            unsafe { &**library.get::<*const PluginMetadata>(b"METADATA").unwrap() };

        let context = Context { metadata };
        let _ = plugin_fn().on_load(&context);

        self.plugins.insert(
            metadata.name.to_string(),
            (metadata.clone(), plugin_fn(), hooks_fn(), library),
        );
    }

    #[must_use]
    pub fn get_plugin(
        &self,
        name: &str,
    ) -> Option<&(PluginMetadata, Box<dyn Plugin>, Box<dyn Hooks>, libloading::Library)> {
        self.plugins.get(name)
    }

    pub fn list_plugins(&self) {
        for (metadata, _, _, _) in self.plugins.values() {
            println!(
                "{}: {} v{} by {}",
                metadata.id,
                metadata.name,
                metadata.version,
                metadata.authors.join(", ")
            );
        }
    }

    pub fn emit_player_join(&mut self, player: &Player, world: &World) {
        for (metadata, _, hooks, _) in self.plugins.values_mut() {
            if hooks.registered_events().unwrap().contains(&"player_join") {
                let context = Context { metadata };
                let event = Event { player, world };
                let _ = hooks.as_mut().on_player_join(&context, &event);
            }
        }
    }
}
