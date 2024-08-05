use pumpkin_macros::packet;

use crate::{
    bytebuf::{packet_id::Packet, ByteBuffer},
    ClientPacket, Identifier, VarInt,
};

#[derive(serde::Serialize)]
#[packet(0x00)]
pub struct CCookieRequest {
    key: Identifier,
}
