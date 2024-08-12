use pumpkin_macros::packet;
use serde::Serialize;

use crate::{uuid::UUID, VarInt};

#[derive(Serialize, Clone)]
#[packet(0x3D)]
pub struct CRemovePlayerInfo<'a> {
    players_count: VarInt,
    players: &'a [UUID],
}

impl<'a> CRemovePlayerInfo<'a> {
    pub fn new(players_count: VarInt, players: &'a [UUID]) -> Self {
        Self {
            players_count,
            players,
        }
    }
}
