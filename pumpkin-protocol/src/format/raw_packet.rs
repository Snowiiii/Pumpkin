// TODO: One potential optimization to consider later is to use a custom arena allocator for the
// data buffer. This would almost certainly improve performance.

use crate::{format::varint, PacketError, VarInt};

//TODO: Implement
pub trait AnyPacket {
    /// The length of the packet in bytes
    fn len(&self) -> usize;
    /// The underlying packet data
    fn into_inner(self) -> bytes::BytesMut;
}

/// A raw, framed packet. May be compressed or uncompressed.
#[derive(Debug, Clone)]
pub struct RawPacket {
    data: bytes::BytesMut,
}

/// An uncompressed packet containing an ID
#[derive(Debug, Clone, PartialEq)]
pub struct UncompressedPacket {
    packet_id: i32,
    data: bytes::BytesMut,
}

/// A maybe-Zlib compressed raw packet. When data_length is zero, the packet is uncompressed.
#[derive(Debug, Clone)]
pub struct CompressedPacket {
    uncompressed_size: usize,
    compressed_data: bytes::BytesMut,
}

impl TryFrom<RawPacket> for UncompressedPacket {
    type Error = PacketError;

    fn try_from(mut raw_packet: RawPacket) -> Result<Self, Self::Error> {
        let packet_id =
        varint::decode(&mut raw_packet.data).map_err(|_| PacketError::DecodeID)?;
        Ok(UncompressedPacket::new(packet_id, raw_packet.into_inner()))
    }
}

impl TryFrom<RawPacket> for CompressedPacket {
    type Error = PacketError;

    fn try_from(mut raw_packet: RawPacket) -> Result<Self, Self::Error> {
        let uncompressed_size = varint::decode(&mut raw_packet.data)
            .map_err(|_| PacketError::MalformedPacket)?;
        Ok(CompressedPacket::new(
            uncompressed_size as usize,
            raw_packet.into_inner(),
        ))
    }
}

impl TryFrom<bytes::BytesMut> for UncompressedPacket {
    type Error = PacketError;

    /// When converting to/from a BytesMut this is the packet only. There is no raw packet framing.
    fn try_from(mut value: bytes::BytesMut) -> Result<Self, Self::Error> {
        // Packet ID comes first
        let packet_id =
        varint::decode(&mut value).map_err(|_| PacketError::MalformedPacket)?;
        Ok(UncompressedPacket::new(packet_id, value))
    }
}

// impl<T> TryFrom<T> for UncompressedPacket
// where T: ClientPacket {
//     type Error;

//     fn try_from(value: T) -> Result<Self, Self::Error> {
//         todo!()
//     }
// }

impl TryFrom<bytes::BytesMut> for CompressedPacket {
    type Error = PacketError;

    fn try_from(mut value: bytes::BytesMut) -> Result<Self, Self::Error> {
        let uncompressed_size =
        varint::decode(&mut value).map_err(|_| PacketError::MalformedPacket)?;
        Ok(CompressedPacket::new(uncompressed_size as usize, value))
    }
}

impl RawPacket {
    pub fn new(data: bytes::BytesMut) -> Self {
        Self { data }
    }

    pub fn len(&self) -> usize {
        self.data.len()
    }

    pub fn into_inner(self) -> bytes::BytesMut {
        self.data
    }
}

impl UncompressedPacket {
    pub fn new(packet_id: i32, data: bytes::BytesMut) -> Self {
        Self { packet_id, data }
    }

    pub fn len(&self) -> usize {
        self.data.len() + VarInt::from(self.packet_id).written_size()
    }

    pub fn packet_id(&self) -> i32 {
        self.packet_id
    }

    pub fn into_inner(self) -> bytes::BytesMut {
        self.data
    }
}

impl CompressedPacket {
    pub fn new(uncompressed_size: usize, compressed_data: bytes::BytesMut) -> Self {
        Self {
            uncompressed_size,
            compressed_data,
        }
    }

    pub fn len(&self) -> usize {
        self.compressed_data.len() + VarInt::from(self.uncompressed_size).written_size()
    }

    pub fn into_inner(self) -> bytes::BytesMut {
        self.compressed_data
    }

    pub fn uncompressed_size(&self) -> usize {
        self.uncompressed_size
    }
}
