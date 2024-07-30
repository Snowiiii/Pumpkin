use mio::Token;

use crate::{
    entity::{
        player::{GameMode, Player},
        Entity,
    },
    protocol::{client::play::CLogin, VarInt},
    server::Difficulty,
};

pub struct World {
    pub players: Vec<Player>,
}

impl World {
    pub fn new() -> Self {
        Self {
            players: Vec::new(),
        }
    }
}
