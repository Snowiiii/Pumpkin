use crate::{
    entity::player::{ChatMode, Hand},
    protocol::{bytebuf::ByteBuffer, VarInt},
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
    pub const PACKET_ID: VarInt = 0x00;

    pub fn read(bytebuf: &mut ByteBuffer) -> Self {
        Self {
            locale: bytebuf.get_string_len(16).unwrap(),
            view_distance: bytebuf.get_i8(),
            chat_mode: ChatMode::from_varint(bytebuf.get_var_int()),
            chat_colors: bytebuf.get_bool(),
            skin_parts: bytebuf.get_u8(),
            main_hand: Hand::from_varint(bytebuf.get_var_int()),
            text_filtering: bytebuf.get_bool(),
            server_listing: bytebuf.get_bool(),
        }
    }
}

pub struct SAcknowledgeFinishConfig {}

impl SAcknowledgeFinishConfig {
    pub const PACKET_ID: VarInt = 0x03;

    pub fn read(_bytebuf: &mut ByteBuffer) -> Self {
        Self {}
    }
}

pub struct SKnownPacks {
    known_pack_count: VarInt,
    // known_packs: &'a [KnownPack]
}

impl SKnownPacks {
    pub const PACKET_ID: VarInt = 0x07;

    pub fn read(bytebuf: &mut ByteBuffer) -> Self {
        Self {
            known_pack_count: bytebuf.get_var_int(),
        }
    }
}
