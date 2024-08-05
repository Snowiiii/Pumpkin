use pumpkin_macros::packet;
use serde::Serialize;

#[derive(Serialize)]
#[packet(0x22)]
pub struct CGameEvent {
    event: u8,
    value: f32,
}

impl CGameEvent {
    pub fn new(event: u8, value: f32) -> Self {
        Self { event, value }
    }
}
