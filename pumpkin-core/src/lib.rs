pub mod gamemode;
pub mod random;
pub mod text;

pub use gamemode::GameMode;

use serde::{Deserialize, Serialize};

#[derive(PartialEq, Serialize, Deserialize)]
pub enum Difficulty {
    Peaceful,
    Easy,
    Normal,
    Hard,
}
