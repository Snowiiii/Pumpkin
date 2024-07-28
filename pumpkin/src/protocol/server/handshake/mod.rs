use crate::protocol::{bytebuf::buffer::ByteBuffer, ConnectionState, VarInt};

pub struct SHandShake {
    pub protocol_version: VarInt,
    pub server_address: String, // 255
    pub server_port: u16,
    pub next_state: ConnectionState,
}

impl SHandShake {
    pub const PACKET_ID: VarInt = 0;

    pub fn read(bytebuf: &mut ByteBuffer) -> Self {
        Self {
            protocol_version: bytebuf.read_var_int().unwrap(),
            server_address: bytebuf.read_string().unwrap(),
            server_port: bytebuf.read_u16().unwrap(),
            next_state: ConnectionState::from_varint(bytebuf.read_var_int().unwrap()),
        }
    }
}
