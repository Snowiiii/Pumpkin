use crate::entity::player::Player;

pub struct World {
    pub players: Vec<Player>,
}

impl Default for World {
    fn default() -> Self {
        Self::new()
    }
}

impl World {
    pub fn new() -> Self {
        Self {
            players: Vec::new(),
        }
    }
}
