use pumpkin_macros::client_packet;
use serde::Serialize;

#[derive(Serialize)]
#[client_packet("play:set_border_center")]
pub struct CSetBorderCenter {
    x: f64,
    z: f64,
}

impl CSetBorderCenter {
    pub fn new(x: f64, z: f64) -> Self {
        Self { x, z }
    }
}
