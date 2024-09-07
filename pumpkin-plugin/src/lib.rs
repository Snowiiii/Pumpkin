mod api;

#[cfg(feature = "plugins")]
use extism::{Manifest, Plugin, Wasm};

pub const PLUGIN_DIR: &str = "plugins";

pub struct PumpkinPlugin {
    #[cfg(feature = "plugins")]
    _plugin: Plugin,
}

pub struct PluginLoader {
    plugins: Vec<PumpkinPlugin>,
}

impl PluginLoader {
    pub fn load() -> Self {
        #[cfg(feature = "plugins")]
        {
            use std::path::Path;
            let plugin_dir = Path::new(PLUGIN_DIR);
            if !plugin_dir.exists() || !plugin_dir.is_dir() {
                log::info!("Creating plugins dir...");
                std::fs::create_dir(plugin_dir).expect("Failed to create Plugin dir");
                return Self { plugins: vec![] };
            }
            let files = std::fs::read_dir(plugin_dir).expect("Failed to read plugin dir");
            let mut plugins = Vec::new();
            for file in files {
                let file = file.expect("Failed to get Plugin file");
                let path = file.path();
                if path
                    .extension()
                    .expect("Failed to get Plugin file extension")
                    == "wasm"
                {
                    log::info!(
                        "Loading Plugin {:?}",
                        path.file_name().expect("Failed to get Plugin file name")
                    );
                    let wasm = Wasm::file(path);
                    let manifest = Manifest::new([wasm]);
                    let mut plugin = Plugin::new(&manifest, [], true).unwrap();
                    plugin
                        .call::<(), ()>("on_enable", ())
                        .expect("Failed to call on_enable funcation");
                    let pumpkin_plugin = PumpkinPlugin { _plugin: plugin };
                    plugins.push(pumpkin_plugin);
                }
            }

            Self { plugins }
        }

        #[cfg(not(feature = "plugins"))]
        Self {
            plugins: Vec::new(),
        }
    }

    pub fn plugins(&mut self) -> &mut Vec<PumpkinPlugin> {
        &mut self.plugins
    }
}
