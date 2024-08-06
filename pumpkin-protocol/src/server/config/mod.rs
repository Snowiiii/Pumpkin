use crate::{bytebuf::ByteBuffer, Identifier, ServerPacket, VarInt};

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

impl ServerPacket for SClientInformation {
    const PACKET_ID: VarInt = VarInt(0x00);

    fn read(bytebuf: &mut ByteBuffer) -> Self {
        Self {
            locale: bytebuf.get_string_len(16).unwrap(),
            view_distance: bytebuf.get_i8(),
            chat_mode: bytebuf.get_var_int(),
            chat_colors: bytebuf.get_bool(),
            skin_parts: bytebuf.get_u8(),
            main_hand: bytebuf.get_var_int(),
            text_filtering: bytebuf.get_bool(),
            server_listing: bytebuf.get_bool(),
        }
    }
}

pub struct SPluginMessage {
    pub channel: Identifier,
    pub data: Vec<u8>,
}

impl ServerPacket for SPluginMessage {
    const PACKET_ID: VarInt = VarInt(0x02);

    fn read(bytebuf: &mut ByteBuffer) -> Self {
        Self {
            channel: bytebuf.get_string().unwrap(),
            data: bytebuf.get_slice().to_vec(),
        }
    }
}

pub struct SAcknowledgeFinishConfig {}

impl ServerPacket for SAcknowledgeFinishConfig {
    const PACKET_ID: VarInt = VarInt(0x03);

    fn read(_bytebuf: &mut ByteBuffer) -> Self {
        Self {}
    }
}

pub struct SKnownPacks {
    pub known_pack_count: VarInt,
    // known_packs: &'a [KnownPack]
}

impl ServerPacket for SKnownPacks {
    const PACKET_ID: VarInt = VarInt(0x07);

    fn read(bytebuf: &mut ByteBuffer) -> Self {
        Self {
            known_pack_count: bytebuf.get_var_int(),
        }
    }
}
