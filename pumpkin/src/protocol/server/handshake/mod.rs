use crate::protocol::{bytebuf::ByteBuffer, ConnectionState, VarInt};

pub struct SHandShake {
    pub protocol_version: VarInt,
    pub server_address: String, // 255
    pub server_port: u16,
    pub next_state: ConnectionState,
}

impl SHandShake {
    pub const PACKET_ID: VarInt = 0x00;

    pub fn read(bytebuf: &mut ByteBuffer) -> Self {
        Self {
            protocol_version: bytebuf.get_var_int(),
            server_address: bytebuf.get_string_len(255).unwrap(),
            server_port: bytebuf.get_u16(),
            next_state: ConnectionState::from_varint(bytebuf.get_var_int()),
        }
    }
}
