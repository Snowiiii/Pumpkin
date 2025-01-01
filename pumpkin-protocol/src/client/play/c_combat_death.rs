use pumpkin_core::text::TextComponent;
use pumpkin_macros::client_packet;
use serde::Serialize;

use crate::VarInt;

#[derive(Serialize)]
#[client_packet("play:player_combat_kill")]
pub struct CCombatDeath<'a> {
    player_id: VarInt,
    message: &'a TextComponent,
}

impl<'a> CCombatDeath<'a> {
    pub fn new(player_id: VarInt, message: &'a TextComponent) -> Self {
        Self { player_id, message }
    }
}
