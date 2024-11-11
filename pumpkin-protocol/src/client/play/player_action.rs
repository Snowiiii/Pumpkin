use crate::{Property, VarInt};

pub enum PlayerAction<'a> {
    AddPlayer {
        name: &'a str,
        properties: &'a [Property],
    },
    InitializeChat(Option<InitChat<'a>>),
    /// Gamemode ?
    UpdateGameMode(VarInt),
    /// Listed ?
    UpdateListed(bool),
    UpdateLatency(u8),
    UpdateDisplayName(u8),
    UpdateListOrder,
}

pub struct InitChat<'a> {
    pub session_id: uuid::Uuid,
    pub expires_at: i64,
    pub public_key: &'a [u8],
    pub signature: &'a [u8],
}
