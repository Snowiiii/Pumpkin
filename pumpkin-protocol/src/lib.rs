use std::num::NonZeroU16;

use bytebuf::{packet_id::Packet, ReadingError};
use bytes::{Buf, BufMut, Bytes};
use codec::{identifier::Identifier, var_int::VarInt};
use pumpkin_core::text::{style::Style, TextComponent};
use serde::{Deserialize, Serialize, Serializer};

pub mod bytebuf;
#[cfg(feature = "clientbound")]
pub mod client;
pub mod codec;
pub mod packet_decoder;
pub mod packet_encoder;
#[cfg(feature = "query")]
pub mod query;
#[cfg(feature = "serverbound")]
pub mod server;

/// To current Minecraft protocol
/// Don't forget to change this when porting
pub const CURRENT_MC_PROTOCOL: NonZeroU16 = unsafe { NonZeroU16::new_unchecked(769) };

pub const MAX_PACKET_SIZE: usize = 2097152;

pub type FixedBitSet = bytes::Bytes;

/// Represents a compression threshold.
///
/// The threshold determines the minimum size of data that should be compressed.
/// Data smaller than the threshold will not be compressed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CompressionThreshold(pub u32);

/// Represents a compression level.
///
/// The level controls the amount of compression applied to the data.
/// Higher levels generally result in higher compression ratios but also
/// increase CPU usage.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CompressionLevel(pub u32);

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum ConnectionState {
    HandShake,
    Status,
    Login,
    Transfer,
    Config,
    Play,
}
pub struct InvalidConnectionState;

impl TryFrom<VarInt> for ConnectionState {
    type Error = InvalidConnectionState;

    fn try_from(value: VarInt) -> Result<Self, Self::Error> {
        let value = value.0;
        match value {
            1 => Ok(Self::Status),
            2 => Ok(Self::Login),
            3 => Ok(Self::Transfer),
            _ => Err(InvalidConnectionState),
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

#[derive(Serialize)]
pub struct IDOrSoundEvent {
    pub id: VarInt,
    pub sound_event: Option<SoundEvent>,
}

#[derive(Serialize)]
pub struct SoundEvent {
    pub sound_name: Identifier,
    pub range: Option<f32>,
}

pub struct RawPacket {
    pub id: VarInt,
    pub bytebuf: Bytes,
}

pub trait ClientPacket: Packet {
    fn write(&self, bytebuf: &mut impl BufMut);
}

pub trait ServerPacket: Packet + Sized {
    fn read(bytebuf: &mut impl Buf) -> Result<Self, ReadingError>;
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
    /// The current name of the Version (e.g. 1.21.4)
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
pub enum NumberFormat {
    /// Show nothing
    Blank,
    /// The styling to be used when formatting the score number
    Styled(Style),
    /// The text to be used as placeholder.
    Fixed(TextComponent),
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

pub enum Label {
    BuiltIn(LinkType),
    TextComponent(TextComponent),
}

impl Serialize for Label {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            Label::BuiltIn(link_type) => link_type.serialize(serializer),
            Label::TextComponent(component) => component.serialize(serializer),
        }
    }
}

#[derive(Serialize)]
pub struct Link<'a> {
    pub is_built_in: bool,
    pub label: Label,
    pub url: &'a String,
}

impl<'a> Link<'a> {
    pub fn new(label: Label, url: &'a String) -> Self {
        Self {
            is_built_in: match label {
                Label::BuiltIn(_) => true,
                Label::TextComponent(_) => false,
            },
            label,
            url,
        }
    }
}

pub enum LinkType {
    BugReport,
    CommunityGuidelines,
    Support,
    Status,
    Feedback,
    Community,
    Website,
    Forums,
    News,
    Announcements,
}

impl Serialize for LinkType {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            LinkType::BugReport => VarInt(0).serialize(serializer),
            LinkType::CommunityGuidelines => VarInt(1).serialize(serializer),
            LinkType::Support => VarInt(2).serialize(serializer),
            LinkType::Status => VarInt(3).serialize(serializer),
            LinkType::Feedback => VarInt(4).serialize(serializer),
            LinkType::Community => VarInt(5).serialize(serializer),
            LinkType::Website => VarInt(6).serialize(serializer),
            LinkType::Forums => VarInt(7).serialize(serializer),
            LinkType::News => VarInt(8).serialize(serializer),
            LinkType::Announcements => VarInt(9).serialize(serializer),
        }
    }
}
