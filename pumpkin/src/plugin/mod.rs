pub mod api;

pub use api::*;
use std::{any::Any, fs, future::Future, path::Path, pin::Pin, sync::Arc};

use crate::server::Server;

type PluginData = (
    PluginMetadata<'static>,
    Box<dyn Plugin>,
    libloading::Library,
    bool,
);

pub struct PluginManager {
    plugins: Vec<PluginData>,
    server: Option<Arc<Server>>,
}

impl Default for PluginManager {
    fn default() -> Self {
        Self::new()
    }
}

const EVENT_PLAYER_JOIN: &str = "player_join";
const EVENT_PLAYER_LEAVE: &str = "player_leave";

type EventResult = Result<bool, String>;
type EventFuture<'a> = Pin<Box<dyn Future<Output = EventResult> + Send + 'a>>;

fn create_default_handler() -> EventFuture<'static> {
    Box::pin(async { Ok(false) })
}

fn handle_player_event<'a>(
    hooks: &'a mut Box<dyn Plugin>,
    context: &'a Context,
    event: &'a (dyn Any + Send + Sync),
    handler: impl Fn(
        &'a mut Box<dyn Plugin>,
        &'a Context,
        &'a types::player::PlayerEvent,
    ) -> EventFuture<'a>,
) -> EventFuture<'a> {
    event
        .downcast_ref::<types::player::PlayerEvent>()
        .map_or_else(|| create_default_handler(), |e| handler(hooks, context, e))
}

fn match_event<'a>(
    event_name: &str,
    hooks: &'a mut Box<dyn Plugin>,
    context: &'a Context,
    event: &'a (dyn Any + Send + Sync),
) -> EventFuture<'a> {
    match event_name {
        EVENT_PLAYER_JOIN => {
            handle_player_event(hooks, context, event, |h, c, e| h.on_player_join(c, e))
        }
        EVENT_PLAYER_LEAVE => {
            handle_player_event(hooks, context, event, |h, c, e| h.on_player_leave(c, e))
        }
        _ => create_default_handler(),
    }
}

impl PluginManager {
    #[must_use]
    pub fn new() -> Self {
        Self {
            plugins: vec![],
            server: None,
        }
    }

    pub fn set_server(&mut self, server: Arc<Server>) {
        self.server = Some(server);
    }

    pub async fn load_plugins(&mut self) -> Result<(), String> {
        const PLUGIN_DIR: &str = "./plugins";

        let dir_entires = fs::read_dir(PLUGIN_DIR);

        for entry in dir_entires.unwrap() {
            if !entry.as_ref().unwrap().path().is_file() {
                continue;
            }
            self.try_load_plugin(entry.unwrap().path().as_path()).await;
        }

        Ok(())
    }

    async fn try_load_plugin(&mut self, path: &Path) {
        let library = unsafe { libloading::Library::new(path).unwrap() };

        let plugin_fn = unsafe { library.get::<fn() -> Box<dyn Plugin>>(b"plugin").unwrap() };
        let metadata: &PluginMetadata =
            unsafe { &**library.get::<*const PluginMetadata>(b"METADATA").unwrap() };

        // The chance that this will panic is non-existent, but just in case
        let context = handle_context(
            metadata.clone(), /* , dispatcher */
            self.server.clone().expect("Server not set"),
        );
        let mut plugin_box = plugin_fn();
        let res = plugin_box.on_load(&context).await;
        let mut loaded = true;
        if let Err(e) = res {
            log::error!("Error loading plugin: {}", e);
            loaded = false;
        }

        self.plugins
            .push((metadata.clone(), plugin_box, library, loaded));
    }

    pub fn is_plugin_loaded(&self, name: &str) -> bool {
        self.plugins
            .iter()
            .any(|(metadata, _, _, loaded)| metadata.name == name && *loaded)
    }

    pub async fn load_plugin(&mut self, name: &str) -> Result<(), String> {
        let plugin = self
            .plugins
            .iter_mut()
            .find(|(metadata, _, _, _)| metadata.name == name);

        if let Some((metadata, plugin, _, loaded)) = plugin {
            if *loaded {
                return Err(format!("Plugin {} is already loaded", name));
            }

            let context = handle_context(
                metadata.clone(), /* , dispatcher */
                self.server.clone().expect("Server not set"),
            );
            let res = plugin.on_load(&context).await;
            if let Err(e) = res {
                return Err(e);
            }
            *loaded = true;
            Ok(())
        } else {
            Err(format!("Plugin {} not found", name))
        }
    }

    pub async fn unload_plugin(&mut self, name: &str) -> Result<(), String> {
        let plugin = self
            .plugins
            .iter_mut()
            .find(|(metadata, _, _, _)| metadata.name == name);

        if let Some((metadata, plugin, _, loaded)) = plugin {
            let context = handle_context(
                metadata.clone(), /* , dispatcher */
                self.server.clone().expect("Server not set"),
            );
            let res = plugin.on_unload(&context).await;
            if let Err(e) = res {
                return Err(e);
            }
            *loaded = false;
            Ok(())
        } else {
            Err(format!("Plugin {} not found", name))
        }
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
                let context = handle_context(
                    metadata.clone(), /* , dispatcher.clone() */
                    self.server.clone().expect("Server not set"),
                );

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
            match match_event(event_name, hooks, &context, event).await {
                Ok(true) => return true,
                Err(e) => log::error!("Error in plugin: {}", e),
                _ => {}
            }
        }

        for (context, hooks) in non_blocking_hooks {
            match match_event(event_name, hooks, &context, event).await {
                Ok(true) => continue,
                Err(e) => log::error!("Error in plugin: {}", e),
                _ => {}
            }
        }

        false
    }
}
