use std::io::{Read, Write as _};

use crate::{
    raw_packet::{CompressedPacket, RawPacket, UncompressedPacket},
    PacketError, VarInt, VarIntDecodeError,
};
use aes::cipher::{BlockDecryptMut, BlockEncryptMut, BlockSizeUser, KeyIvInit as _};
use bytes::{Buf, BufMut};
use flate2::bufread::ZlibDecoder;
use flate2::write::ZlibEncoder;
use tokio_util::codec::{Decoder, Encoder};

/// An encoder for the simplest representation of a packet
///
/// The protocol specifies that a packet is always prefixed by its length, regardless of or state.
#[derive(Debug, Default, Clone)]
pub struct RawPacketCodec {
    current_packet_len: Option<usize>,
}

impl Decoder for RawPacketCodec {
    type Item = RawPacket;

    type Error = PacketError;

    fn decode(&mut self, src: &mut bytes::BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        let packet_len = match self.current_packet_len {
            Some(len) => len,
            None => match VarInt::decode_partial_buf(src) {
                Ok(len) => {
                    // TODO: this is actually somewhat unsafe, since we allocate assuming the client is honest about packet size. This should be limited
                    src.reserve(len as usize - src.len());
                    self.current_packet_len = Some(len as usize);
                    len as usize
                }
                Err(VarIntDecodeError::Incomplete) => return Ok(None),
                Err(VarIntDecodeError::TooLarge) => return Err(PacketError::MalformedLength),
            },
        };
        if src.len() < packet_len {
            return Ok(None);
        }
        let packet_data = src.split_to(packet_len);
        self.current_packet_len = None;
        Ok(Some(RawPacket::new(packet_data)))
    }
}

impl Encoder<RawPacket> for RawPacketCodec {
    type Error = PacketError;

    fn encode(&mut self, item: RawPacket, dst: &mut bytes::BytesMut) -> Result<(), Self::Error> {
        // A raw packet is always prefixed by a VarInt indicating its length, then the data.
        VarInt::from(item.len() as i32).encode(dst.writer())?;
        dst.extend(item.into_inner());
        Ok(())
    }
}

/// A codec that decodes and encodes packets
#[derive(Default)]
pub struct UncompressedPacketCodec {
    raw_codec: RawPacketCodec,
}

impl Decoder for UncompressedPacketCodec {
    type Item = UncompressedPacket;

    type Error = PacketError;

    fn decode(&mut self, src: &mut bytes::BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        // Decode the raw packet frame
        let raw_packet = match self.raw_codec.decode(src)? {
            Some(packet) => packet.into_inner(),
            None => return Ok(None),
        };
        Ok(Some(UncompressedPacket::try_from(raw_packet)?))
    }
}

impl Encoder<UncompressedPacket> for UncompressedPacketCodec {
    type Error = PacketError;

    fn encode(
        &mut self,
        item: UncompressedPacket,
        dst: &mut bytes::BytesMut,
    ) -> Result<(), Self::Error> {
        // Encode the packet length
        VarInt(item.len() as i32).encode(dst.writer())?;
        // Encode the packet ID
        VarInt(item.packet_id()).encode(dst.writer())?;
        // Encode the packet data
        dst.extend(item.into_inner());
        Ok(())
    }
}

#[derive(Debug, Clone)]
struct CompressedPacketCodec {
    raw_codec: RawPacketCodec,
    compression_level: flate2::Compression,
    threshold: Option<usize>,
}

impl CompressedPacketCodec {
    pub fn new(compression_level: flate2::Compression, threshold: Option<usize>) -> Self {
        Self {
            raw_codec: RawPacketCodec::default(),
            compression_level,
            threshold,
        }
    }

    /// Compress an uncompressed packet
    fn compress(&self, packet: UncompressedPacket) -> Result<CompressedPacket, PacketError> {
        if let Some(threshold) = self.threshold {
            if packet.len() < threshold {
                let mut bytes = bytes::BytesMut::new();
                UncompressedPacketCodec::default().decode(&mut bytes)?;
                return Ok(CompressedPacket::new(0, bytes));
            }
        }
        let uncompressed_size = packet.len();
        let mut uncompressed_data = bytes::BytesMut::new();
        VarInt(packet.packet_id()).encode((&mut uncompressed_data).writer())?;
        uncompressed_data.extend(packet.into_inner());
        let compressed_bytes = self.compress_bytes(uncompressed_data);
        Ok(CompressedPacket::new(uncompressed_size, compressed_bytes))
    }

    fn decompress(&self, packet: CompressedPacket) -> Result<UncompressedPacket, PacketError> {
        let decompressed_length = packet.uncompressed_size();
        let mut decoder = ZlibDecoder::new(packet.into_inner().reader());
        let mut decompressed = bytes::BytesMut::zeroed(decompressed_length);
        decoder.read_exact(&mut decompressed)?;
        UncompressedPacket::try_from(decompressed)
    }

    fn compress_bytes(&self, bytes: bytes::BytesMut) -> bytes::BytesMut {
        let compressed_bytes = bytes::BytesMut::new();
        let mut encoder = ZlibEncoder::new(compressed_bytes.writer(), self.compression_level);
        encoder.write_all(&bytes).unwrap();
        encoder.finish().unwrap().into_inner()
    }
}

impl Decoder for CompressedPacketCodec {
    type Item = UncompressedPacket;

    type Error = PacketError;

    fn decode(&mut self, src: &mut bytes::BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        let packet_data = match self.raw_codec.decode(src)? {
            Some(packet) => packet.into_inner(),
            None => return Ok(None),
        };
        match self.decompress(CompressedPacket::try_from(packet_data)?) {
            Ok(packet) => Ok(Some(packet)),
            Err(e) => Err(e),
        }
    }
}

impl Encoder<UncompressedPacket> for CompressedPacketCodec {
    type Error = PacketError;

    fn encode(
        &mut self,
        item: UncompressedPacket,
        dst: &mut bytes::BytesMut,
    ) -> Result<(), Self::Error> {
        let compressed = self.compress(item)?;
        VarInt::from(compressed.len()).encode(dst.writer())?;
        VarInt::from(compressed.uncompressed_size() as i32).encode(dst.writer())?;
        dst.extend(compressed.into_inner());
        Ok(())
    }
}

type DecryptionCipher = cfb8::Decryptor<aes::Aes128>;
type EncryptionCipher = cfb8::Encryptor<aes::Aes128>;
#[derive(Debug, Clone)]
pub struct EncryptedCodec {
    encrypter: EncryptionCipher,
    decrypter: DecryptionCipher,
}

impl Decoder for EncryptedCodec {
    type Item = bytes::BytesMut;

    type Error = PacketError;

    fn decode(&mut self, src: &mut bytes::BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        let mut bytes_consumed = 0;
        src.chunks_exact_mut(DecryptionCipher::block_size())
            .for_each(|chunk| {
                self.decrypter.decrypt_block_mut(chunk.into());
                bytes_consumed += DecryptionCipher::block_size();
            });
        let decrypted = src.split_to(bytes_consumed);
        Ok(Some(decrypted))
    }
}

impl Encoder<bytes::BytesMut> for EncryptedCodec {
    type Error = PacketError;

    fn encode(
        &mut self,
        mut item: bytes::BytesMut,
        dst: &mut bytes::BytesMut,
    ) -> Result<(), Self::Error> {
        let mut bytes_consumed = 0;
        item.chunks_exact_mut(EncryptionCipher::block_size())
            .for_each(|chunk| {
                self.encrypter.encrypt_block_mut(chunk.into());
                bytes_consumed += EncryptionCipher::block_size();
            });
        dst.extend(item);
        Ok(())
    }
}

impl EncryptedCodec {
    pub fn new(key: [u8; 16]) -> Self {
        let decrypter =
            DecryptionCipher::new_from_slices(&key, &key).expect("Failed to initialize decryptor");
        let encrypter =
            EncryptionCipher::new_from_slices(&key, &key).expect("Failed to initialize encryptor");
        Self {
            encrypter,
            decrypter,
        }
    }
}

#[cfg(test)]
mod test {
    use flate2::Compression;
    use futures::SinkExt as _;
    use tokio_stream::StreamExt;
    use tokio_util::codec::{Decoder, Encoder as _, FramedRead, FramedWrite};

    use crate::{
        packet_codec::{CompressedPacketCodec, EncryptedCodec, UncompressedPacketCodec},
        raw_packet::UncompressedPacket,
        VarInt,
    };

    #[test]
    fn encode_uncompressed() {
        let raw_packet_bytes = {
            let mut packet = Vec::new();
            VarInt(13).encode(&mut packet).unwrap(); // Packet Length
            VarInt(1).encode(&mut packet).unwrap(); // Packet ID
            packet.extend(b"Hello, world"); // Packet Contents
            packet
        };
        let packet = UncompressedPacket::new(1, "Hello, world".into());
        let mut encoded = bytes::BytesMut::new();
        UncompressedPacketCodec::default()
            .encode(packet, &mut encoded)
            .unwrap();
        assert_eq!(encoded[..], raw_packet_bytes);
        let decoded = UncompressedPacketCodec::default()
            .decode(&mut encoded)
            .expect("Failed to decode packet with packet error")
            .unwrap();
        assert_eq!(decoded.packet_id(), 1);
        assert_eq!(decoded.into_inner(), "Hello, world".as_bytes());
    }

    #[tokio::test]
    async fn uncompressed_packet_roundtrip() {
        let packets = vec![
            UncompressedPacket::new(1, "Hello, world".into()),
            UncompressedPacket::new(127, "Single Byte".into()),
            UncompressedPacket::new(128, "Two Bytes".into()),
            UncompressedPacket::new(25565, "Goodbye, world".into()),
        ];

        let (a, b) = tokio::io::duplex(1024);
        let mut writer = FramedWrite::new(a, UncompressedPacketCodec::default());
        let mut reader = FramedRead::new(b, UncompressedPacketCodec::default());
        for packet in packets {
            writer.send(packet.clone()).await.unwrap();
            let received = reader.next().await.unwrap().unwrap();
            assert_eq!(packet.packet_id(), received.packet_id());
            assert_eq!(packet.into_inner(), received.into_inner());
        }
    }

    #[tokio::test]
    async fn framed_packet_roundtrip() {
        let a_128 = "a".repeat(128);
        let packets = vec![
            UncompressedPacket::new(1, "Hello, world".into()),
            UncompressedPacket::new(127, "This test involves encoding the packetID in one byte and the message length in one byte".into()),
            UncompressedPacket::new(128, a_128.as_bytes().into()),
            UncompressedPacket::new(25565, "Goodbye, world".into()),
        ];

        let (a, b) = tokio::io::duplex(1024);
        let compressed_codec = CompressedPacketCodec::new(Compression::default(), Some(0));

        let mut writer = FramedWrite::new(a, compressed_codec.clone());
        let mut reader = FramedRead::new(b, compressed_codec);
        for packet in packets {
            writer.send(packet.clone()).await.unwrap();
            let received = reader.next().await.unwrap().unwrap();
            assert_eq!(packet, received);
        }
    }

    #[tokio::test]
    async fn compression_roundtrip() {
        let mut compressed_codec = CompressedPacketCodec::new(Compression::default(), Some(0));
        // Something which will compress
        let packet = UncompressedPacket::new(
            1,
            "Hello, world!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!".into(),
        );
        let mut encoded = bytes::BytesMut::new();
        compressed_codec
            .encode(packet.clone(), &mut encoded)
            .unwrap();

        let decoded = compressed_codec
            .decode(dbg!(&mut encoded))
            .expect("Failed to decode packet with packet error")
            .unwrap();
        assert_eq!(packet, decoded);
    }

    #[tokio::test]
    async fn encryption_roundtrip() {
        let key = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15];
        let (a, b) = tokio::io::duplex(1024);
        let encryption_codec = EncryptedCodec::new(key);
        let mut writer = FramedWrite::new(a, encryption_codec.clone());
        let mut reader = FramedRead::new(b, encryption_codec);
        let data = b"Hello, world!";
        writer.send(data[..].into()).await.unwrap();
        let received = reader.next().await.unwrap().unwrap();
        assert_eq!(data[..], received);
    }
}
