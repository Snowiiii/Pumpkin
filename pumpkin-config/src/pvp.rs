use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
pub struct PVPConfig {
    /// Is PVP enabled ?
    pub enabled: bool,
    /// Do we want to have the Red hurt animation & fov bobbing
    pub hurt_animation: bool,
    /// Should players in creative be protected against PVP
    pub protect_creative: bool,
    /// Has PVP Knockback?
    pub knockback: bool,
    /// Should player swing when attacking?
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
