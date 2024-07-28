use bytebuf::buffer::ByteBuffer;
use serde::{Deserialize, Serialize};

pub mod bytebuf;

pub mod client;
pub mod server;

type VarInt = i32;
type VarLong = i64;

#[derive(Debug)]
pub enum ConnectionState {
    HandShake,
    Status,
    Login,
    Transfer,
}

impl ConnectionState {
    pub fn from_varint(var_int: VarInt) -> Self {
        match var_int {
            1 => Self::Status,
            2 => Self::Login,
            3 => Self::Transfer,
            _ => panic!("Unexpected Status {}", var_int),
        }
    }
}

#[derive(Debug)]
pub struct RawPacket {
    pub len: VarInt,
    pub id: VarInt,
    pub bytebuf: ByteBuffer,
}

pub trait ClientPacket {
    const PACKET_ID: VarInt;

    fn write(&self, bytebuf: &mut ByteBuffer);
}

#[derive(Serialize, Deserialize)]
pub struct StatusResponse {
    pub version: Version,
    pub description: String,
    // Players, favicon ...
}
#[derive(Serialize, Deserialize)]
pub struct Version {
    pub version: String,
    pub protocol: u32,
}
