use pumpkin_core::text::TextComponent;
use tokio::sync::mpsc;
use uuid::Uuid;

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
    pub name: String,
    pub uuid: Uuid,
    channel: mpsc::Sender<PlayerEventAction<'a>>,
}

impl<'a> PlayerEvent<'a> {
    #[must_use]
    pub fn new(name: String, uuid: Uuid, channel: mpsc::Sender<PlayerEventAction<'a>>) -> Self {
        Self {
            name,
            uuid,
            channel,
        }
    }

    pub async fn send_message(&self, message: TextComponent<'a>) {
        let (tx, rx) = oneshot::channel();
        self.channel
            .send(PlayerEventAction::SendMessage {
                message,
                player_id: self.uuid,
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
                player_id: self.uuid,
                response: tx,
            })
            .await
            .unwrap();
        rx.await.unwrap();
    }
}
