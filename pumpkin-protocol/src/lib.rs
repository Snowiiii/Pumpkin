use bytebuf::{packet_id::Packet, ByteBuffer, DeserializerError};
use pumpkin_core::text::{style::Style, TextComponent};
use serde::{Deserialize, Serialize};

pub mod bytebuf;
pub mod client;
pub mod packet_decoder;
pub mod packet_encoder;
pub mod query;
pub mod server;
pub mod slot;

mod var_int;
pub use var_int::*;

mod var_long;
pub use var_long::*;

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

pub trait ClientPacket: Packet {
    fn write(&self, bytebuf: &mut ByteBuffer);
}

pub trait ServerPacket: Packet + Sized {
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
    /// The current name of the Version (e.g. 1.21.3)
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
