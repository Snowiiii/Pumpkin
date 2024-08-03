use std::io::{self, Write};

use bytebuf::ByteBuffer;
use bytes::Buf;
use serde::Serialize;
use thiserror::Error;

pub mod bytebuf;
pub mod client;
pub mod server;

pub mod packet_decoder;
pub mod packet_encoder;

pub const MAX_PACKET_SIZE: i32 = 2097152;

/// usally uses a namespace like "minecraft:thing"
pub type Identifier = String;
pub type VarInt = i32;
pub type VarLong = i64;

pub struct BitSet(pub VarInt, pub Vec<i64>);

pub struct VarInt32(pub i32);

impl VarInt32 {
    /// The maximum number of bytes a `VarInt` could occupy when read from and
    /// written to the Minecraft protocol.
    pub const MAX_SIZE: usize = 5;

    /// Returns the exact number of bytes this varint will write when
    /// [`Encode::encode`] is called, assuming no error occurs.
    pub const fn written_size(self) -> usize {
        match self.0 {
            0 => 1,
            n => (31 - n.leading_zeros() as usize) / 7 + 1,
        }
    }

    pub fn decode_partial(r: &mut &[u8]) -> Result<i32, VarIntDecodeError> {
        let mut val = 0;
        for i in 0..Self::MAX_SIZE {
            let byte = r.get_u8();
            val |= (i32::from(byte) & 0b01111111) << (i * 7);
            if byte & 0b10000000 == 0 {
                return Ok(val);
            }
        }

        Err(VarIntDecodeError::TooLarge)
    }

    pub fn encode(&self, mut w: impl Write) -> Result<(), io::Error> {
        let x = self.0 as u64;
        let stage1 = (x & 0x000000000000007f)
            | ((x & 0x0000000000003f80) << 1)
            | ((x & 0x00000000001fc000) << 2)
            | ((x & 0x000000000fe00000) << 3)
            | ((x & 0x00000000f0000000) << 4);

        let leading = stage1.leading_zeros();

        let unused_bytes = (leading - 1) >> 3;
        let bytes_needed = 8 - unused_bytes;

        // set all but the last MSBs
        let msbs = 0x8080808080808080;
        let msbmask = 0xffffffffffffffff >> (((8 - bytes_needed + 1) << 3) - 1);

        let merged = stage1 | (msbs & msbmask);
        let bytes = merged.to_le_bytes();

        w.write_all(unsafe { bytes.get_unchecked(..bytes_needed as usize) })?;
        Ok(())
    }

    pub fn decode(r: &mut &[u8]) -> Result<Self, VarIntDecodeError> {
        let mut val = 0;
        for i in 0..Self::MAX_SIZE {
            let byte = r.get_u8();
            val |= (i32::from(byte) & 0b01111111) << (i * 7);
            if byte & 0b10000000 == 0 {
                return Ok(VarInt32(val));
            }
        }
        Err(VarIntDecodeError::TooLarge)
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Debug, Error)]
pub enum VarIntDecodeError {
    #[error("incomplete VarInt decode")]
    Incomplete,
    #[error("VarInt is too large")]
    TooLarge,
}

#[derive(Error, Debug)]
pub enum PacketError {
    #[error("failed to decode packet ID")]
    DecodeID,
    #[error("failed to encode packet ID")]
    EncodeID,
    #[error("failed to write encoded packet")]
    EncodeFailedWrite,
    #[error("failed to write encoded packet to connection")]
    ConnectionWrite,
    #[error("packet exceeds maximum length")]
    TooLong,
    #[error("packet length is out of bounds")]
    OutOfBounds,
    #[error("malformed packet length VarInt")]
    MailformedLength,
}

#[derive(Debug, PartialEq)]
pub enum ConnectionState {
    HandShake,
    Status,
    Login,
    Transfer,
    Config,
    Play,
}

impl ConnectionState {
    pub fn from_varint(var_int: VarInt) -> Self {
        match var_int {
            1 => Self::Status,
            2 => Self::Login,
            3 => Self::Transfer,
            _ => {
                log::info!("Unexpected Status {}", var_int);
                Self::Status
            }
        }
    }
}

pub struct RawPacket {
    pub len: VarInt,
    pub id: VarInt,
    pub bytebuf: ByteBuffer,
}

pub trait ClientPacket {
    const PACKET_ID: VarInt;

    fn write(&self, bytebuf: &mut ByteBuffer);
}

pub trait ServerPacket {
    const PACKET_ID: VarInt;

    fn read(bytebuf: &mut ByteBuffer) -> Self;
}

#[derive(Serialize)]
pub struct StatusResponse {
    pub version: Version,
    pub players: Players,
    pub description: String,
    pub favicon: String, // data:image/png;base64,<data>
                         // Players, favicon ...
}
#[derive(Serialize)]
pub struct Version {
    pub name: String,
    pub protocol: u32,
}

#[derive(Serialize)]
pub struct Players {
    pub max: u32,
    pub online: u32,
    pub sample: Vec<Sample>,
}

#[derive(Serialize)]
pub struct Sample {
    pub name: String,
    pub id: String, // uuid
}

pub struct KnownPack<'a> {
    pub namespace: &'a str,
    pub id: &'a str,
    pub version: &'a str,
}
