pub mod api;

pub use api::*;
use std::{any::Any, fs, path::Path, sync::Arc};

use crate::command::dispatcher::CommandDispatcher;

type PluginData = (
    PluginMetadata<'static>,
    Box<dyn Plugin>,
    libloading::Library,
    bool,
);

pub struct PluginManager {
    plugins: Vec<PluginData>,
    command_dispatcher: Option<Arc<CommandDispatcher<'static>>>,
}

impl Default for PluginManager {
    fn default() -> Self {
        Self::new()
    }
}

impl PluginManager {
    #[must_use]
    pub fn new() -> Self {
        PluginManager {
            plugins: vec![],
            command_dispatcher: None,
        }
    }

    pub fn set_command_dispatcher(&mut self, dispatcher: Arc<CommandDispatcher<'static>>) {
        self.command_dispatcher = Some(dispatcher);
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
        let metadata: &PluginMetadata =
            unsafe { &**library.get::<*const PluginMetadata>(b"METADATA").unwrap() };

        // let dispatcher = self.command_dispatcher.clone().expect("Command dispatcher not set").clone();
        let context = handle_context(metadata.clone() /* , dispatcher */);
        let mut plugin_box = plugin_fn();
        let res = plugin_box.on_load(&context);
        let mut loaded = true;
        if let Err(e) = res {
            log::error!("Error loading plugin: {}", e);
            loaded = false;
        }

        self.plugins
            .push((metadata.clone(), plugin_box, library, loaded));
    }

    #[must_use]
    pub fn list_plugins(&self) -> Vec<(&PluginMetadata, &bool)> {
        self.plugins
            .iter()
            .map(|(metadata, _, _, loaded)| (metadata, loaded))
            .collect()
    }

    pub async fn emit<T: Any + Send + Sync>(&mut self, event_name: &str, event: &T) -> bool {
        let mut blocking_hooks = Vec::new();
        let mut non_blocking_hooks = Vec::new();

        /* let dispatcher = self.command_dispatcher
        .clone()
        .expect("Command dispatcher not set"); // This should not happen */

        for (metadata, hooks, _, loaded) in &mut self.plugins {
            if !*loaded {
                continue;
            }

            let registered_events = match hooks.registered_events() {
                Ok(events) => events,
                Err(e) => {
                    log::error!("Failed to get registered events: {}", e);
                    continue;
                }
            };

            if let Some(matching_event) = registered_events.iter().find(|e| e.name == event_name) {
                let context = handle_context(metadata.clone() /* , dispatcher.clone() */);

                if matching_event.blocking {
                    blocking_hooks.push((context, hooks));
                } else {
                    non_blocking_hooks.push((context, hooks));
                }
            }
        }

        let event_sort = |a: &(_, &mut Box<dyn Plugin>), b: &(_, &mut Box<dyn Plugin>)| {
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
        };

        blocking_hooks.sort_by(event_sort);
        non_blocking_hooks.sort_by(event_sort);

        let event = event as &(dyn Any + Sync + Send);

        for (context, hooks) in blocking_hooks {
            let r = match event_name {
                "player_join" => {
                    if let Some(event) = event.downcast_ref::<types::player::PlayerEvent>() {
                        hooks.on_player_join(&context, event)
                    } else {
                        Box::pin(async { Ok(false) })
                    }
                }
                "player_leave" => {
                    if let Some(event) = event.downcast_ref::<types::player::PlayerEvent>() {
                        hooks.on_player_leave(&context, event)
                    } else {
                        Box::pin(async { Ok(false) })
                    }
                }
                _ => Box::pin(async { Ok(false) }),
            };
            match r.await {
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
                    if let Some(event) = event.downcast_ref::<types::player::PlayerEvent>() {
                        hooks.on_player_join(&context, event)
                    } else {
                        Box::pin(async { Ok(false) })
                    }
                }
                "player_leave" => {
                    if let Some(event) = event.downcast_ref::<types::player::PlayerEvent>() {
                        hooks.on_player_leave(&context, event)
                    } else {
                        Box::pin(async { Ok(false) })
                    }
                }
                _ => Box::pin(async { Ok(false) }),
            };
            match r.await {
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
