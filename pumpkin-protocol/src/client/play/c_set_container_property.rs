use pumpkin_macros::client_packet;
use serde::Serialize;

use crate::VarInt;

#[derive(Serialize)]
#[client_packet("play:container_set_data")]
pub struct CSetContainerProperty {
    window_id: VarInt,
    property: i16,
    value: i16,
}

impl CSetContainerProperty {
    pub const fn new(window_id: VarInt, property: i16, value: i16) -> Self {
        Self {
            window_id,
            property,
            value,
        }
    }
}
