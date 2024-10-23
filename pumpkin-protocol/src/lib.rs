use bytebuf::{packet_id::ClientPacketID, ByteBuffer, DeserializerError};
use bytes::Buf;
use pumpkin_core::text::{style::Style, TextComponent};
use serde::{Deserialize, Serialize};
use std::io::{self, Write};
use thiserror::Error;

pub mod bytebuf;
pub mod client;
pub mod packet_decoder;
pub mod packet_encoder;
pub mod server;
pub mod slot;

/// To current Minecraft protocol
/// Don't forget to change this when porting
pub const CURRENT_MC_PROTOCOL: u32 = 768;

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
    #[error("failed to write into decoder: {0}")]
    FailedWrite(String),
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

#[derive(Debug, PartialEq, Clone, Copy)]
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

pub enum SoundCategory {
    Master,
    Music,
    Records,
    Weather,
    Blocks,
    Hostile,
    Neutral,
    Players,
    Ambient,
    Voice,
}

pub struct RawPacket {
    pub id: VarInt,
    pub bytebuf: ByteBuffer,
}

pub trait ClientPacket: ClientPacketID {
    fn write(&self, bytebuf: &mut ByteBuffer);
}

pub trait ServerPacket: Sized {
    fn read(bytebuf: &mut ByteBuffer) -> Result<Self, DeserializerError>;
}

#[derive(Serialize)]
pub struct StatusResponse {
    /// The version on which the Server is running. Optional
    pub version: Option<Version>,
    /// Information about currently connected Players. Optional
    pub players: Option<Players>,
    /// The description displayed also called MOTD (Message of the day). Optional
    pub description: String,
    /// The icon displayed, Optional
    pub favicon: Option<String>,
    /// Players are forced to use Secure chat
    pub enforce_secure_chat: bool,
}
#[derive(Serialize)]
pub struct Version {
    /// The current name of the Version (e.g. 1.21.2)
    pub name: String,
    /// The current Protocol Version (e.g. 767)
    pub protocol: u32,
}

#[derive(Serialize)]
pub struct Players {
    /// The maximum Player count the server allows
    pub max: u32,
    /// The current online player count
    pub online: u32,
    /// Information about currently connected players.
    /// Note player can disable listing here.
    pub sample: Vec<Sample>,
}

#[derive(Serialize)]
pub struct Sample {
    /// Players Name
    pub name: String,
    /// Players UUID
    pub id: String,
}

// basically game profile
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

#[derive(Serialize)]
pub enum NumberFormat<'a> {
    /// Show nothing
    Blank,
    /// The styling to be used when formatting the score number
    Styled(Style<'a>),
    /// The text to be used as placeholder.
    Fixed(TextComponent<'a>),
}

#[derive(Debug, PartialEq, Eq, Hash)]
pub enum PositionFlag {
    X,
    Y,
    Z,
    YRot,
    XRot,
    DeltaX,
    DeltaY,
    DeltaZ,
    RotateDelta,
}

impl PositionFlag {
    fn get_mask(&self) -> i32 {
        match self {
            PositionFlag::X => 1 << 0,
            PositionFlag::Y => 1 << 1,
            PositionFlag::Z => 1 << 2,
            PositionFlag::YRot => 1 << 3,
            PositionFlag::XRot => 1 << 4,
            PositionFlag::DeltaX => 1 << 5,
            PositionFlag::DeltaY => 1 << 6,
            PositionFlag::DeltaZ => 1 << 7,
            PositionFlag::RotateDelta => 1 << 8,
        }
    }

    pub fn get_bitfield(flags: &[PositionFlag]) -> i32 {
        flags.iter().fold(0, |acc, flag| acc | flag.get_mask())
    }
}
