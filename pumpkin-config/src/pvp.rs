use serde::{Deserialize, Serialize};
use serde_inline_default::serde_inline_default;

#[serde_inline_default]
#[derive(Deserialize, Serialize)]
pub struct PVPConfig {
    /// Is PVP enabled ?
    #[serde_inline_default(true)]
    pub enabled: bool,
    /// Do we want to have the Red hurt animation & fov bobbing
    #[serde_inline_default(true)]
    pub hurt_animation: bool,
    /// Should players in creative be protected against PVP
    #[serde_inline_default(true)]
    pub protect_creative: bool,
    /// Has PVP Knockback?
    #[serde_inline_default(true)]
    pub knockback: bool,
    /// Should player swing when attacking?
    #[serde_inline_default(true)]
    pub swing: bool,
}

impl Default for PVPConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            hurt_animation: true,
            protect_creative: true,
            knockback: true,
            swing: true,
        }
    }
}
