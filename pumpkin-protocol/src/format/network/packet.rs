use serde::{de::DeserializeOwned, Serialize};

use crate::{bytebuf::ByteBuffer, format::varint::VarIntType};

#[derive(Debug)]
pub struct RawPacket {
    pub id: i32,
    pub bytebuf: ByteBuffer,
}

pub trait Packet {
    const PACKET_ID: VarIntType;
}

pub trait ClientPacket: Packet + Serialize {}

pub trait ServerPacket: Packet + DeserializeOwned {
}
