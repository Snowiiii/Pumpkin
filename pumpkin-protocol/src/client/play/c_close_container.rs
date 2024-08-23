use serde::Serialize;
use pumpkin_macros::packet;

#[derive(Serialize)]
#[packet(0x12)]
pub struct CCloseContainer {
    window_id: u8
}

impl CCloseContainer {
    pub const fn new(window_id: u8) -> Self {
        Self {
            window_id
        }
    }
}