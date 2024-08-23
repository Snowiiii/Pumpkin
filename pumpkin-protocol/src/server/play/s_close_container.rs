use pumpkin_macros::packet;
use serde::Deserialize;

#[derive(Deserialize)]
#[packet(0x0F)]
pub struct SCloseContainer {
    pub window_id: u8,
}
