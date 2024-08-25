use pumpkin_macros::packet;
use serde::Deserialize;

#[derive(Deserialize)]
#[packet(0x1D)]
pub struct SSetPlayerGround {
    pub on_ground: bool,
}
