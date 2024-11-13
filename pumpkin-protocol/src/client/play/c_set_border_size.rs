use pumpkin_macros::client_packet;
use serde::Serialize;

#[derive(Serialize)]
#[client_packet("play:set_border_size")]
pub struct CSetBorderSize {
    diameter: f64,
}

impl CSetBorderSize {
    pub fn new(diameter: f64) -> Self {
        Self { diameter }
    }
}
