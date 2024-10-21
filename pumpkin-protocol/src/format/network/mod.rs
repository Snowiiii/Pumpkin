pub mod deserializer;
pub mod packet;
pub mod serializer;

use std::io;

use thiserror::Error;

pub use packet::Packet;

/// An error that can occur when encoding or decoding a packet
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
    #[error("malformed packet")]
    MalformedPacket,
}

impl serde::ser::Error for PacketError {
    fn custom<T: std::fmt::Display>(msg: T) -> Self {
        PacketError::EncodeData
    }
}

impl serde::de::Error for PacketError {
    fn custom<T: std::fmt::Display>(msg: T) -> Self {
        PacketError::MalformedPacket
    }
}

impl From<io::Error> for PacketError {
    fn from(_: io::Error) -> Self {
        PacketError::FailedWrite
    }
}
