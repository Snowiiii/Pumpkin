use pumpkin_macros::client_packet;
use serde::Serialize;

use crate::VarInt;

#[derive(Serialize)]
#[client_packet("play:set_border_warning_delay")]
pub struct CSetBorderWarningDelay {
    warning_time: VarInt,
}

impl CSetBorderWarningDelay {
    pub fn new(warning_time: VarInt) -> Self {
        Self { warning_time }
    }
}
