pub mod api;

pub use api::*;
use std::{any::Any, fs, path::Path};

type PluginData = (
    PluginMetadata<'static>,
    Box<dyn Plugin>,
    Box<dyn events::Hooks>,
    libloading::Library,
    bool,
);

pub struct PluginManager {
    plugins: Vec<PluginData>,
}

impl Default for PluginManager {
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

impl PluginManager {
    #[must_use]
    pub fn new() -> Self {
        PluginManager {
            plugins: vec![],
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
        let hooks_fn = unsafe {
            library
                .get::<fn() -> Box<dyn events::Hooks>>(b"hooks")
                .unwrap()
        };
        let metadata: &PluginMetadata =
            unsafe { &**library.get::<*const PluginMetadata>(b"METADATA").unwrap() };

        let context = Context { metadata };
        let res = plugin_fn().on_load(&context);
        let mut loaded = true;
        if let Err(e) = res {
            log::error!("Error loading plugin: {}", e);
            loaded = false;
        }

        self.plugins.push(
            (metadata.clone(), plugin_fn(), hooks_fn(), library, loaded),
        );
    }

    pub fn list_plugins(&self) -> Vec<(&PluginMetadata, &bool)> {
        return self.plugins.iter().map(|(metadata, _, _, _, loaded)| (metadata, loaded)).collect();
    }

    pub fn emit<T: Any>(&mut self, event_name: &str, event: &T) -> bool {
        let mut blocking_hooks = Vec::new();
        let mut non_blocking_hooks = Vec::new();

        for (metadata, _, hooks, _, loaded) in self.plugins.iter_mut() {
            if !*loaded {
                continue;
            }
            if hooks
                .registered_events()
                .unwrap()
                .iter()
                .any(|e| e.name == event_name)
            {
                let context = Context { metadata };
                if hooks
                    .registered_events()
                    .unwrap()
                    .iter()
                    .any(|e| e.name == event_name && e.blocking)
                {
                    blocking_hooks.push((context, hooks));
                } else {
                    non_blocking_hooks.push((context, hooks));
                }
            }
        }

        blocking_hooks.sort_by(|a, b| {
            b.1.registered_events()
                .unwrap()
                .iter()
                .find(|e| e.name == event_name)
                .unwrap()
                .priority
                .cmp(
                    &a.1.registered_events()
                        .unwrap()
                        .iter()
                        .find(|e| e.name == event_name)
                        .unwrap()
                        .priority,
                )
        });
        non_blocking_hooks.sort_by(|a, b| {
            b.1.registered_events()
                .unwrap()
                .iter()
                .find(|e| e.name == event_name)
                .unwrap()
                .priority
                .cmp(
                    &a.1.registered_events()
                        .unwrap()
                        .iter()
                        .find(|e| e.name == event_name)
                        .unwrap()
                        .priority,
                )
        });

        let event = event as &dyn Any;

        for (context, hooks) in blocking_hooks {
            let r = match event_name {
                "player_join" => {
                    if let Some(event) = event.downcast_ref::<crate::entity::player::Player>() {
                        hooks.on_player_join(&context, event)
                    } else {
                        Ok(false)
                    }
                }
                "player_leave" => {
                    if let Some(event) = event.downcast_ref::<crate::entity::player::Player>() {
                        hooks.on_player_leave(&context, event)
                    } else {
                        Ok(false)
                    }
                }
                _ => Ok(false),
            };
            match r {
                Ok(true) => return true,
                Err(e) => {
                    log::error!("Error in plugin: {}", e);
                }
                _ => {}
            }
        }

        for (context, hooks) in non_blocking_hooks {
            let r = match event_name {
                "player_join" => {
                    if let Some(event) = event.downcast_ref::<crate::entity::player::Player>() {
                        hooks.on_player_join(&context, event)
                    } else {
                        Ok(false)
                    }
                }
                "player_leave" => {
                    if let Some(event) = event.downcast_ref::<crate::entity::player::Player>() {
                        hooks.on_player_leave(&context, event)
                    } else {
                        Ok(false)
                    }
                }
                _ => Ok(false),
            };
            match r {
                Ok(true) => continue,
                Err(e) => {
                    log::error!("Error in plugin: {}", e);
                }
                _ => {}
            }
        }

        false
    }
}
