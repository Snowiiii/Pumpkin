use pumpkin_macros::client_packet;
use serde::Serialize;

use crate::VarLong;

#[derive(Serialize)]
#[client_packet("play:set_border_lerp_size")]
pub struct CSetBorderLerpSize {
    old_diameter: f64,
    new_diameter: f64,
    speed: VarLong,
}

impl CSetBorderLerpSize {
    pub fn new(old_diameter: f64, new_diameter: f64, speed: VarLong) -> Self {
        Self {
            old_diameter,
            new_diameter,
            speed,
        }
    }
}
