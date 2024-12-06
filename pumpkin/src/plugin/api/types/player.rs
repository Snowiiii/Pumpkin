use std::{ops::Deref, sync::Arc};

use pumpkin_core::text::TextComponent;
use tokio::sync::mpsc;
use uuid::Uuid;

use crate::entity::player::Player;

pub enum PlayerEventAction<'a> {
    SendMessage {
        message: TextComponent<'a>,
        player_id: Uuid,
        response: oneshot::Sender<()>,
    },
    Kick {
        reason: TextComponent<'a>,
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

pub struct PlayerEvent<'a> {
    pub player: Arc<Player>,
    channel: mpsc::Sender<PlayerEventAction<'a>>,
}

impl Deref for PlayerEvent<'_> {
    type Target = Player;

    fn deref(&self) -> &Self::Target {
        &self.player
    }
}

impl<'a> PlayerEvent<'a> {
    #[must_use]
    pub fn new(player: Arc<Player>, channel: mpsc::Sender<PlayerEventAction<'a>>) -> Self {
        Self { player, channel }
    }

    pub async fn send_message(&self, message: TextComponent<'a>) {
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

    pub async fn kick(&self, reason: TextComponent<'a>) {
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
