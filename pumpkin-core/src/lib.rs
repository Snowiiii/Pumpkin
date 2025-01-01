pub mod gamemode;
pub mod math;
pub mod permission;
pub mod random;
pub mod text;

pub use gamemode::GameMode;
pub use permission::PermissionLvl;
#[cfg(feature = "schemars")]
pub use schemars::JsonSchema;

use serde::{Deserialize, Serialize};

#[derive(PartialEq, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
pub enum Difficulty {
    Peaceful,
    Easy,
    Normal,
    Hard,
}

#[derive(Deserialize, Serialize, Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ProfileAction {
    ForcedNameChange,
    UsingBannedSkin,
}

#[macro_export]
macro_rules! assert_eq_delta {
    ($x:expr, $y:expr, $d:expr) => {
        if !(2f64 * ($x - $y).abs() <= $d * ($x.abs() + $y.abs())) {
            panic!("{} vs {} ({} vs {})", $x, $y, ($x - $y).abs(), $d);
        }
    };
}
