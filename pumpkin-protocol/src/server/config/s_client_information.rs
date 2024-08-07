use pumpkin_macros::packet;

use crate::{bytebuf::ByteBuffer, Identifier, ServerPacket, VarInt};

#[derive(serde::Deserialize)]
#[packet(0x00)]
pub struct SClientInformation {
    pub locale: String, // 16
    pub view_distance: i8,
    pub chat_mode: VarInt, // Varint
    pub chat_colors: bool,
    pub skin_parts: u8,
    pub main_hand: VarInt,
    pub text_filtering: bool,
    pub server_listing: bool,
}
