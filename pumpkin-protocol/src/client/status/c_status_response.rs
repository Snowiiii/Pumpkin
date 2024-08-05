use pumpkin_macros::packet;

use crate::{
    bytebuf::{packet_id::Packet, ByteBuffer},
    ClientPacket, VarInt,
};

#[derive(serde::Serialize)]
#[packet(0x00)]
pub struct CStatusResponse<'a> {
    json_response: &'a str, // 32767
}

impl<'a> CStatusResponse<'a> {
    pub fn new(json_response: &'a str) -> Self {
        Self { json_response }
    }
}

