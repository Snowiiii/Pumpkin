use std::{ops::Deref, sync::Arc};

use pumpkin_core::text::TextComponent;
use tokio::sync::mpsc;
use uuid::Uuid;

use crate::{entity::player::Player, server::Server};

pub enum PlayerEventAction {
    SendMessage {
        message: TextComponent,
        player_id: Uuid,
        response: oneshot::Sender<()>,
    },
    Kick {
        reason: TextComponent,
        player_id: Uuid,
        response: oneshot::Sender<()>,
    },
    SetHealth {
        health: f32,
        food: i32,
        saturation: f32,
        player_id: Uuid,
        response: oneshot::Sender<()>,
    },
    Kill {
        player_id: Uuid,
        response: oneshot::Sender<()>,
    },
    SetGameMode {
        game_mode: pumpkin_core::GameMode,
        player_id: Uuid,
        response: oneshot::Sender<()>,
    },
}

pub struct PlayerEvent {
    pub player: Arc<Player>,
    channel: mpsc::Sender<PlayerEventAction>,
}

impl Deref for PlayerEvent {
    type Target = Player;

    fn deref(&self) -> &Self::Target {
        &self.player
    }
}

impl PlayerEvent {
    #[must_use]
    pub fn new(player: Arc<Player>, channel: mpsc::Sender<PlayerEventAction>) -> Self {
        Self { player, channel }
    }

    pub async fn send_message(&self, message: TextComponent) {
        let (tx, rx) = oneshot::channel();
        self.channel
            .send(PlayerEventAction::SendMessage {
                message,
                player_id: self.player.gameprofile.id,
                response: tx,
            })
            .await
            .unwrap();
        rx.await.unwrap();
    }

    pub async fn kick(&self, reason: TextComponent) {
        let (tx, rx) = oneshot::channel();
        self.channel
            .send(PlayerEventAction::Kick {
                reason,
                player_id: self.player.gameprofile.id,
                response: tx,
            })
            .await
            .unwrap();
        rx.await.unwrap();
    }
}

pub async fn player_event_handler(server: Arc<Server>, player: Arc<Player>) -> PlayerEvent {
    let (send, mut recv) = mpsc::channel(1);
    let player_event = PlayerEvent::new(player.clone(), send);
    let players_copy = server.get_all_players().await;
    tokio::spawn(async move {
        while let Some(action) = recv.recv().await {
            match action {
                PlayerEventAction::SendMessage {
                    message,
                    player_id,
                    response,
                } => {
                    if let Some(player) =
                        players_copy.iter().find(|p| p.gameprofile.id == player_id)
                    {
                        player.send_system_message(&message).await;
                    }
                    response.send(()).unwrap();
                }
                PlayerEventAction::Kick {
                    reason,
                    player_id,
                    response,
                } => {
                    if let Some(player) =
                        players_copy.iter().find(|p| p.gameprofile.id == player_id)
                    {
                        player.kick(reason).await;
                    }
                    response.send(()).unwrap();
                }
                PlayerEventAction::SetHealth {
                    health,
                    food,
                    saturation,
                    player_id,
                    response,
                } => {
                    if let Some(player) =
                        players_copy.iter().find(|p| p.gameprofile.id == player_id)
                    {
                        player.set_health(health, food, saturation).await;
                    }
                    response.send(()).unwrap();
                }
                PlayerEventAction::Kill {
                    player_id,
                    response,
                } => {
                    if let Some(player) =
                        players_copy.iter().find(|p| p.gameprofile.id == player_id)
                    {
                        player.kill().await;
                    }
                    response.send(()).unwrap();
                }
                PlayerEventAction::SetGameMode {
                    game_mode,
                    player_id,
                    response,
                } => {
                    if let Some(player) =
                        players_copy.iter().find(|p| p.gameprofile.id == player_id)
                    {
                        player.set_gamemode(game_mode).await;
                    }
                    response.send(()).unwrap();
                }
            }
        }
    });
    player_event
}
