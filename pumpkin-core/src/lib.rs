pub mod gamemode;
pub mod math;
pub mod permission;
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

#[derive(Deserialize, Serialize, Clone, Debug, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ProfileAction {
    ForcedNameChange,
    UsingBannedSkin,
}
