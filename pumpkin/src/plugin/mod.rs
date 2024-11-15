use std::{
    collections::HashMap,
    env,
    fmt::{Debug, Display},
    fs::{self},
    io::{self, BufReader, Read, Write},
    path::Path,
    sync::Arc,
};

use itertools::Itertools;
use serde::Deserialize;
use tokio::sync::RwLock;

pub use events::*;

mod events;

pub(crate) const EXTRA_PLUGIN_DIR_NAME: &str = "EXTRA_PLUGIN_DIR";
pub(crate) const VALID_PLUGIN_EXTENSIONS: [&str; 2] = ["zip", "pplugin"];
pub(crate) const PLUGIN_ENTRY_POINT_NAME: &[u8; 23] = b"pumpkin_register_plugin";

#[cfg(target_os = "windows")]
const THIS_PLUGIN_PLATFORM: PluginPlatform = PluginPlatform::Windows;
#[cfg(target_os = "linux")]
const THIS_PLUGIN_PLATFORM: PluginPlatform = PluginPlatform::Linux;
#[cfg(target_os = "macos")]
const THIS_PLUGIN_PLATFORM: PluginPlatform = PluginPlatform::MacOS;

#[macro_export]
macro_rules! register_plugin {
    ($plugin_type:ident) => {
        #[no_mangle]
        pub fn pumpkin_register_plugin() -> Box<dyn pumpkin::plugin::PumpkinPlugin> {
            pumpkin::plugin::logging::init_logger();
            Box::new($plugin_type::default())
        }
    };
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
pub(crate) enum PluginPlatform {
    Windows,
    MacOS,
    Linux,
}

impl Display for PluginPlatform {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            PluginPlatform::Windows => "Windows",
            PluginPlatform::MacOS => "MacOS",
            PluginPlatform::Linux => "Linux",
        })
    }
}

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct PluginDefinition {
    name: String,
    identifier: String,
    version: String,
    supported_platforms: Vec<PluginPlatform>,
}

impl Display for PluginDefinition {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} ({}:{})", self.name, self.identifier, self.version)
    }
}

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct PluginToml {
    plugin: PluginDefinition,
}

pub trait PumpkinPlugin: Send + Sync + 'static {
    fn init(&mut self, event_registry: &mut EventRegistry);
}

pub type PluginEntryPoint = fn() -> Box<dyn PumpkinPlugin>;

pub struct PluginManager {
    plugins: HashMap<String, (PluginToml, libloading::Library, Box<dyn PumpkinPlugin>)>,
    event_registry: Arc<RwLock<EventRegistry>>,
}

impl PluginManager {
    #[must_use]
    pub fn new() -> Self {
        PluginManager {
            plugins: HashMap::new(),
            event_registry: Arc::new(RwLock::new(EventRegistry::default())),
        }
    }

    #[must_use]
    pub fn event_registry(&self) -> Arc<RwLock<EventRegistry>> {
        self.event_registry.clone()
    }

    pub fn load_plugins(&mut self) {
        const DEFAULT_PLUGIN_DIR: &str = "./plugins";
        log::info!("Starting Plugin loading...");

        if let Err(e) = self.load_plugins_from_dir(DEFAULT_PLUGIN_DIR) {
            log::error!("Failed to load plugins: {e:?}");
        }

        if let Ok(dir) = env::var(EXTRA_PLUGIN_DIR_NAME) {
            if let Err(e) = self.load_plugins_from_dir(&dir) {
                log::error!("Failed to load plugins from extra dir: {e:?}");
            }
        }

        log::info!("Finished plugin loading! ({} loaded)", self.plugins.len());
    }

    fn load_plugins_from_dir(&mut self, dir: &str) -> io::Result<()> {
        for entry in (fs::read_dir(dir)?).flatten() {
            let path = entry.path();
            if path.is_file()
                && VALID_PLUGIN_EXTENSIONS
                    .contains(&path.extension().and_then(|o| o.to_str()).unwrap_or(""))
            {
                if let Err(e) = self.try_load_file(&path) {
                    log::info!("Failed to load file '{:?}' error: {e:?}", &path);
                }
            }
        }

        Ok(())
    }

    fn try_load_file(&mut self, path: &Path) -> io::Result<()> {
        use zip::ZipArchive;

        log::info!("Trying to load plugin from: {path:?}");

        let file = std::fs::File::open(path)?;
        let reader = BufReader::new(file);

        let mut zip = ZipArchive::new(reader)?;

        let config = if let Ok(mut plugin_toml) = zip.by_name("plugin.toml") {
            let mut plugin_toml_str = String::new();
            plugin_toml.read_to_string(&mut plugin_toml_str)?;

            let plugin_config: PluginToml = match toml::from_str::<PluginToml>(&plugin_toml_str) {
                Ok(config) => config,
                Err(e) => {
                    log::info!(
                        "Failed to parse plugin.toml from file '{path:?}' with error: {e:?}"
                    );
                    return Ok(());
                }
            };

            if !self.valid_plugin_identifier(&plugin_config.plugin.identifier) {
                return Ok(());
            }

            if !plugin_config
                .plugin
                .supported_platforms
                .contains(&THIS_PLUGIN_PLATFORM)
            {
                log::error!(
                    "Plugin {} ({}) is not supported on this platform!",
                    plugin_config.plugin.name,
                    plugin_config.plugin.identifier
                );
                return Ok(());
            }

            plugin_config
        } else {
            log::error!("No plugin.toml found for file: {path:?}");
            return Ok(());
        };

        if let Ok(mut library_file) = zip.by_name(get_platform_lib_name(THIS_PLUGIN_PLATFORM)) {
            let library_temp_file = {
                let temp_path = env::temp_dir().join(&config.plugin.identifier);
                let temp_file_path = temp_path.join(get_platform_lib_name(THIS_PLUGIN_PLATFORM));
                std::fs::create_dir_all(&temp_path)?;
                let mut temp_file = std::fs::File::create(temp_path.join(&temp_file_path))?;
                let mut lib_content = Vec::new();
                library_file.read_to_end(&mut lib_content)?;
                temp_file.write_all(&lib_content)?;
                temp_file_path
            };

            let library = unsafe {
                match libloading::Library::new(&library_temp_file) {
                    Ok(lib) => lib,
                    Err(e) => {
                        log::error!(
                            "Failed to load library of plugin {} with error: {e:?}",
                            config.plugin.name
                        );
                        return Ok(());
                    }
                }
            };

            let plugin = unsafe {
                if let Ok(pl) = library.get::<PluginEntryPoint>(PLUGIN_ENTRY_POINT_NAME) {
                    pl()
                } else {
                    log::error!("Failed to find plugin entrypoint! Is it defined?");
                    return Ok(());
                }
            };

            log::info!(
                "Successfully loaded plugin {} ({}:{})",
                config.plugin.name,
                config.plugin.identifier,
                config.plugin.version
            );
            self.plugins.insert(
                config.plugin.identifier.to_string(),
                (config, library, plugin),
            );
        } else {
            log::error!(
                "Libray for plugin {} on platform {THIS_PLUGIN_PLATFORM} not present!",
                config.plugin.name
            );
        }

        Ok(())
    }

    fn valid_plugin_identifier(&self, identifier: &str) -> bool {
        const VALID_CHARS: &str = "abcdefghijklmnopqrstuvwxyz_";

        if self.plugins.keys().contains(&identifier.to_string()) {
            log::warn!("Plugin with identifier {identifier} already loaded!");
            return false;
        }

        if !identifier
            .chars()
            .fold(String::new(), |acc, c| {
                if VALID_CHARS.contains(c) {
                    acc
                } else {
                    acc + &c.to_string()
                }
            })
            .is_empty()
        {
            log::error!("Plugin identifier '{identifier}' invalid. Identifier may only consist of these characters: {VALID_CHARS}");
            return false;
        }

        true
    }

    pub async fn init(&mut self) {
        let mut event_registry = self.event_registry.write().await;
        for pl in self.plugins.values_mut() {
            log::info!("Running initialization for {}", pl.0.plugin);
            pl.2.init(&mut event_registry);
        }
    }
}

impl Default for PluginManager {
    fn default() -> Self {
        Self::new()
    }
}

fn get_platform_lib_name(this_plugin_platform: PluginPlatform) -> &'static str {
    match this_plugin_platform {
        PluginPlatform::Windows => "plugin.dll",
        PluginPlatform::MacOS => "libplugin.dylib",
        PluginPlatform::Linux => "libplugin.so",
    }
}

pub mod logging {
    use log::LevelFilter;
    pub use log::{debug, error, info, log, trace, warn};

    pub fn init_logger() {
        use pumpkin_config::ADVANCED_CONFIG;
        if ADVANCED_CONFIG.logging.enabled {
            let mut logger = simple_logger::SimpleLogger::new();

            if !ADVANCED_CONFIG.logging.timestamp {
                logger = logger.without_timestamps();
            }

            if ADVANCED_CONFIG.logging.env {
                logger = logger.env();
            }

            logger = logger.with_level(convert_logger_filter(ADVANCED_CONFIG.logging.level));

            logger = logger.with_colors(ADVANCED_CONFIG.logging.color);
            logger = logger.with_threads(ADVANCED_CONFIG.logging.threads);
            logger.init().unwrap();
        }
    }

    const fn convert_logger_filter(level: pumpkin_config::logging::LevelFilter) -> LevelFilter {
        match level {
            pumpkin_config::logging::LevelFilter::Off => LevelFilter::Off,
            pumpkin_config::logging::LevelFilter::Error => LevelFilter::Error,
            pumpkin_config::logging::LevelFilter::Warn => LevelFilter::Warn,
            pumpkin_config::logging::LevelFilter::Info => LevelFilter::Info,
            pumpkin_config::logging::LevelFilter::Debug => LevelFilter::Debug,
            pumpkin_config::logging::LevelFilter::Trace => LevelFilter::Trace,
        }
    }
}
