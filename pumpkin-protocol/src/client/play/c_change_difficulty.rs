use pumpkin_macros::packet;
use serde::Serialize;

#[derive(Serialize)]
#[packet(0x0B)]
pub struct CChangeDifficulty {
    difficulty: u8,
    locked: bool,
}

impl CChangeDifficulty {
    pub fn new(difficulty: u8, locked: bool) -> Self {
        Self { difficulty, locked }
    }
}
