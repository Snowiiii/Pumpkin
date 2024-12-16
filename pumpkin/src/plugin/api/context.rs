use std::{fs, path::Path};

use tokio::sync::mpsc::{self, Sender};

use super::PluginMetadata;

pub struct Context {
    metadata: PluginMetadata<'static>,
    _channel: Sender<ContextAction>,
}
impl Context {
    pub fn new(metadata: PluginMetadata<'static>, channel: Sender<ContextAction>) -> Context {
        Context { metadata, _channel: channel }
    }

    pub fn get_logger(&self) -> Logger {
        Logger {
            plugin_name: self.metadata.name.to_string(),
        }
    }

    pub fn get_data_folder(&self) -> String {
        let path = format!("./plugins/{}", self.metadata.name);
        if !Path::new(&path).exists() {
            fs::create_dir_all(&path).unwrap();
        }
        path
    }
/*  TODO: Implement when dispatcher is mutable
    pub async fn register_command(&self, tree: crate::command::tree::CommandTree<'static>) {
        self.channel.send(ContextAction::RegisterCommand(tree)).await;
    } */
}

pub enum ContextAction {
    // TODO: Implement when dispatcher is mutable
}

pub fn handle_context(metadata: PluginMetadata<'static>/* , dispatcher: Arc<CommandDispatcher<'static>> */) -> Context {
    let (send, mut recv) = mpsc::channel(1);
    tokio::spawn(async move {
        while let Some(action) = recv.recv().await {
            match action {
                /* ContextAction::RegisterCommand(_tree) => {
                    // TODO: Implement when dispatcher is mutable
                } */
            }
        }
    });
    Context::new(metadata, send)
}

pub struct Logger {
    plugin_name: String,
}

impl Logger {
    pub fn info(&self, message: &str) {
        log::info!("[{}] {}", self.plugin_name, message);
    }

    pub fn warn(&self, message: &str) {
        log::warn!("[{}] {}", self.plugin_name, message);
    }

    pub fn error(&self, message: &str) {
        log::error!("[{}] {}", self.plugin_name, message);
    }
}
