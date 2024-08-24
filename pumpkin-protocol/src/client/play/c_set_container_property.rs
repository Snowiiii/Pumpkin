use pumpkin_macros::packet;
use serde::Serialize;
#[derive(Serialize)]
#[packet(0x14)]
pub struct CSetContainerProperty {
    window_id: u8,
    property: i16,
    value: i16,
}

impl CSetContainerProperty {
    pub const fn new(window_id: u8, property: i16, value: i16) -> Self {
        Self {
            window_id,
            property,
            value,
        }
    }
}
