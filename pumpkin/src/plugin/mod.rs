use pumpkin_api::{Plugin, PluginMetadata};
use std::{collections::HashMap, fs, path::Path};

pub struct PluginManager<'s> {
    plugins: HashMap<String, (PluginMetadata<'s>, Box<dyn Plugin>, libloading::Library)>,
}

impl Default for PluginManager<'_> {
    fn default() -> Self {
        Self::new()
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
        struct Logger {
            plugin_name: String,
        }

        impl pumpkin_api::Logger for Logger {
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
        impl pumpkin_api::PluginContext for Context<'_> {
            fn get_logger(&self) -> Box<dyn pumpkin_api::Logger> {
                Box::new(Logger {
                    plugin_name: self.metadata.name.to_string(),
                })
            }
        }

        let library = unsafe { libloading::Library::new(path).unwrap() };

        let plugin_fn = unsafe { library.get::<fn() -> Box<dyn Plugin>>(b"plugin").unwrap() };
        let metadata: &PluginMetadata =
            unsafe { &**library.get::<*const PluginMetadata>(b"METADATA").unwrap() };

        let context = Context { metadata };
        let _ = plugin_fn().on_load(&context);

        self.plugins.insert(
            metadata.name.to_string(),
            (metadata.clone(), plugin_fn(), library),
        );
    }

    #[must_use]
    pub fn get_plugin(
        &self,
        name: &str,
    ) -> Option<&(PluginMetadata, Box<dyn Plugin>, libloading::Library)> {
        self.plugins.get(name)
    }

    pub fn list_plugins(&self) {
        for (metadata, _, _) in self.plugins.values() {
            println!(
                "{}: {} v{} by {}",
                metadata.id,
                metadata.name,
                metadata.version,
                metadata.authors.join(", ")
            );
        }
    }
}
