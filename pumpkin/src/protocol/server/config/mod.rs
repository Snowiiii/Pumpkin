use crate::{
    entity::player::{ChatMode, Hand},
    protocol::{bytebuf::buffer::ByteBuffer, VarInt},
};

pub struct SClientInformation {
    pub locale: String, // 16
    pub view_distance: i8,
    pub chat_mode: ChatMode, // Varint
    pub chat_colors: bool,
    pub skin_parts: u8,
    pub main_hand: Hand,
    pub text_filtering: bool,
    pub server_listing: bool,
}

impl SClientInformation {
    pub const PACKET_ID: VarInt = 0;

    pub fn read(bytebuf: &mut ByteBuffer) -> Self {
        Self {
            locale: bytebuf.read_string_len(16).unwrap(),
            view_distance: bytebuf.read_i8().unwrap(),
            chat_mode: ChatMode::from_varint(bytebuf.read_var_int().unwrap()),
            chat_colors: bytebuf.read_bool().unwrap(),
            skin_parts: bytebuf.read_u8().unwrap(),
            main_hand: Hand::from_varint(bytebuf.read_var_int().unwrap()),
            text_filtering: bytebuf.read_bool().unwrap(),
            server_listing: bytebuf.read_bool().unwrap(),
        }
    }
}

pub struct SAcknowledgeFinishConfig {}

impl SAcknowledgeFinishConfig {
    pub const PACKET_ID: VarInt = 3;

    pub fn read(_bytebuf: &mut ByteBuffer) -> Self {
        Self {}
    }
}
