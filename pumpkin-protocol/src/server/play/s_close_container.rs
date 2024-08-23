use pumpkin_macros::packet;
use serde::Serialize;

#[derive(Serialize)]
#[packet(0x0F)]
pub struct SCloseContainer {
    pub window_id: u8,
}
