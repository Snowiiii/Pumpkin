use std::{fs, path::Path, sync::Arc};

use tokio::sync::mpsc::{self, Sender};

use crate::server::Server;

use super::{
    types::player::{player_event_handler, PlayerEvent},
    PluginMetadata,
};

pub struct Context {
    metadata: PluginMetadata<'static>,
    channel: Sender<ContextAction>,
}
impl Context {
    #[must_use]
    pub fn new(metadata: PluginMetadata<'static>, channel: Sender<ContextAction>) -> Self {
        Self { metadata, channel }
    }

    #[must_use]
    pub fn get_logger(&self) -> Logger {
        Logger {
            plugin_name: self.metadata.name.to_string(),
        }
    }

    #[must_use]
    pub fn get_data_folder(&self) -> String {
        let path = format!("./plugins/{}", self.metadata.name);
        if !Path::new(&path).exists() {
            fs::create_dir_all(&path).unwrap();
        }
        path
    }

    pub async fn get_player_by_name(
        &self,
        player_name: String,
    ) -> Result<PlayerEvent<'static>, String> {
        let (send, recv) = oneshot::channel();
        let _ = self
            .channel
            .send(ContextAction::GetPlayerByName {
                player_name,
                response: send,
            })
            .await;
        recv.await.unwrap()
    }

    /*  TODO: Implement when dispatcher is mutable
    pub async fn register_command(&self, tree: crate::command::tree::CommandTree<'static>) {
        self.channel.send(ContextAction::RegisterCommand(tree)).await;
    } */
}

pub enum ContextAction {
    // TODO: Implement when dispatcher is mutable
    GetPlayerByName {
        player_name: String,
        response: oneshot::Sender<Result<PlayerEvent<'static>, String>>,
    },
}

pub fn handle_context(
    metadata: PluginMetadata<'static>, /* , dispatcher: Arc<CommandDispatcher<'static>> */
    server: Arc<Server>,
) -> Context {
    let (send, mut recv) = mpsc::channel(1);
    tokio::spawn(async move {
        while let Some(action) = recv.recv().await {
            match action {
                /* ContextAction::RegisterCommand(_tree) => {
                    // TODO: Implement when dispatcher is mutable
                } */
                ContextAction::GetPlayerByName {
                    player_name,
                    response,
                } => {
                    let player = server.get_player_by_name(&player_name).await;
                    if let Some(player) = player {
                        response
                            .send(Ok(
                                player_event_handler(server.clone(), player.clone()).await
                            ))
                            .unwrap();
                    } else {
                        response.send(Err("Player not found".to_string())).unwrap();
                    }
                }
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
