use pumpkin_macros::client_packet;
use serde::Serialize;

use crate::VarInt;

#[derive(Serialize)]
#[client_packet("play:set_border_warning_distance")]
pub struct CSetBorderWarningDistance {
    warning_blocks: VarInt,
}

impl CSetBorderWarningDistance {
    pub fn new(warning_blocks: VarInt) -> Self {
        Self { warning_blocks }
    }
}
