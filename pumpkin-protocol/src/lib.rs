use bytebuf::{packet_id::Packet, ByteBuffer, DeserializerError};
use bytes::Buf;
use serde::{Deserialize, Serialize};
use std::io::{self, Write};
use thiserror::Error;

pub mod bytebuf;
pub mod client;
pub mod packet_decoder;
pub mod packet_encoder;
pub mod server;
pub mod slot;
pub mod uuid;

pub const CURRENT_MC_PROTOCOL: u32 = 767;

pub const MAX_PACKET_SIZE: i32 = 2097152;

/// usally uses a namespace like "minecraft:thing"
pub type Identifier = String;
pub type VarIntType = i32;
pub type VarLongType = i64;
pub type FixedBitSet = bytes::Bytes;

pub struct BitSet<'a>(pub VarInt, pub &'a [i64]);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VarInt(pub VarIntType);

impl VarInt {
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
            if !r.has_remaining() {
                return Err(VarIntDecodeError::Incomplete);
            }
            let byte = r.get_u8();
            val |= (i32::from(byte) & 0b01111111) << (i * 7);
            if byte & 0b10000000 == 0 {
                return Ok(val);
            }
        }

        Err(VarIntDecodeError::TooLarge)
    }

    pub fn encode(&self, mut w: impl Write) -> Result<(), io::Error> {
        let mut x = self.0 as u64;
        loop {
            let byte = (x & 0x7F) as u8;
            x >>= 7;
            if x == 0 {
                w.write_all(&[byte])?;
                break;
            }
            w.write_all(&[byte | 0x80])?;
        }
        Ok(())
    }

    pub fn decode(r: &mut &[u8]) -> Result<Self, VarIntDecodeError> {
        let mut val = 0;
        for i in 0..Self::MAX_SIZE {
            if !r.has_remaining() {
                return Err(VarIntDecodeError::Incomplete);
            }
            let byte = r.get_u8();
            val |= (i32::from(byte) & 0b01111111) << (i * 7);
            if byte & 0b10000000 == 0 {
                return Ok(VarInt(val));
            }
        }
        Err(VarIntDecodeError::TooLarge)
    }
}

impl From<i32> for VarInt {
    fn from(value: i32) -> Self {
        VarInt(value)
    }
}

impl From<u32> for VarInt {
    fn from(value: u32) -> Self {
        VarInt(value as i32)
    }
}

impl From<u8> for VarInt {
    fn from(value: u8) -> Self {
        VarInt(value as i32)
    }
}

impl From<usize> for VarInt {
    fn from(value: usize) -> Self {
        VarInt(value as i32)
    }
}

impl From<VarInt> for i32 {
    fn from(value: VarInt) -> Self {
        value.0
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
    #[error("failed to encode packet Length")]
    EncodeLength,
    #[error("failed to encode packet data")]
    EncodeData,
    #[error("failed to write encoded packet")]
    EncodeFailedWrite,
    #[error("failed to write into decoder")]
    FailedWrite,
    #[error("failed to flush decoder")]
    FailedFinish,
    #[error("failed to write encoded packet to connection")]
    ConnectionWrite,
    #[error("packet exceeds maximum length")]
    TooLong,
    #[error("packet length is out of bounds")]
    OutOfBounds,
    #[error("malformed packet length VarInt")]
    MalformedLength,
}

#[derive(Debug, PartialEq, Clone)]
pub enum ConnectionState {
    HandShake,
    Status,
    Login,
    Transfer,
    Config,
    Play,
}

impl From<VarInt> for ConnectionState {
    fn from(value: VarInt) -> Self {
        let value = value.0;
        match value {
            1 => Self::Status,
            2 => Self::Login,
            3 => Self::Transfer,
            _ => {
                log::info!("Unexpected Status {}", value);
                Self::Status
            }
        }
    }
}

pub struct RawPacket {
    pub id: VarInt,
    pub bytebuf: ByteBuffer,
}

pub trait ClientPacket: Packet {
    fn write(&self, bytebuf: &mut ByteBuffer);
}

pub trait ServerPacket: Packet + Sized {
    fn read(bytebuf: &mut ByteBuffer) -> Result<Self, DeserializerError>;
}

#[derive(Serialize)]
pub struct StatusResponse {
    pub version: Version,
    pub players: Players,
    pub description: String,
    pub favicon: Option<String>, // data:image/png;base64,<data>
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

// basicly game profile
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Property {
    pub name: String,
    // base 64
    pub value: String,
    // base 64
    pub signature: Option<String>,
}

pub struct KnownPack<'a> {
    pub namespace: &'a str,
    pub id: &'a str,
    pub version: &'a str,
}
