use std::io::Write;

use aes::cipher::{generic_array::GenericArray, BlockEncryptMut, BlockSizeUser, KeyIvInit};
use bytes::{BufMut, BytesMut};
use pumpkin_config::compression::CompressionInfo;
use thiserror::Error;

use libdeflater::{CompressionLvl, Compressor};

use crate::{bytebuf::ByteBuffer, ClientPacket, VarInt, MAX_PACKET_SIZE};

type Cipher = cfb8::Encryptor<aes::Aes128>;

// Encoder: Server -> Client
// Supports ZLib endecoding/compression
// Supports Aes128 Encyption
pub struct PacketEncoder {
    buf: BytesMut,
    compress_buf: Vec<u8>,
    compression: Option<CompressionInfo>,
    cipher: Option<Cipher>,
    compressor: Compressor, // Reuse the compressor for all packets
}

// Manual implementation of Default trait for PacketEncoder
// Since compressor does not implement Default
impl Default for PacketEncoder {
    fn default() -> Self {
        Self {
            buf: BytesMut::with_capacity(1024),
            compress_buf: Vec::with_capacity(1024),
            compression: None,
            cipher: None,
            compressor: Compressor::new(CompressionLvl::fastest()), // init compressor with no compression level
        }
    }
}

impl PacketEncoder {
    pub fn append_packet<P: ClientPacket>(&mut self, packet: &P) -> Result<(), PacketEncodeError> {
        let start_len = self.buf.len();
        let mut writer = (&mut self.buf).writer();

        let mut packet_buf = ByteBuffer::empty();
        VarInt(P::PACKET_ID)
            .encode(&mut writer)
            .map_err(|_| PacketEncodeError::EncodeID)?;
        packet.write(&mut packet_buf);

        writer
            .write(packet_buf.buf())
            .map_err(|_| PacketEncodeError::EncodeFailedWrite)?;

        let data_len = self.buf.len() - start_len;

        if let Some(compression) = &self.compression {
            if data_len > compression.threshold as usize {
                // Get the data to compress
                let data_to_compress = &self.buf[start_len..];

                // Clear the compression buffer
                self.compress_buf.clear();

                // Compute the maximum size of compressed data
                let max_compressed_size =
                    self.compressor.zlib_compress_bound(data_to_compress.len());

                // Ensure compress_buf has enough capacity
                self.compress_buf.resize(max_compressed_size, 0);

                // Compress the data
                let compressed_size = self
                    .compressor
                    .zlib_compress(data_to_compress, &mut self.compress_buf)
                    .map_err(|_| PacketEncodeError::CompressionFailed)?;

                // Resize compress_buf to actual compressed size
                self.compress_buf.resize(compressed_size, 0);

                let data_len_size = VarInt(data_len as i32).written_size();

                let packet_len = data_len_size + compressed_size;

                if packet_len >= MAX_PACKET_SIZE as usize {
                    return Err(PacketEncodeError::TooLong);
                }

                self.buf.truncate(start_len);

                let mut writer = (&mut self.buf).writer();

                VarInt(packet_len as i32)
                    .encode(&mut writer)
                    .map_err(|_| PacketEncodeError::EncodeLength)?;
                VarInt(data_len as i32)
                    .encode(&mut writer)
                    .map_err(|_| PacketEncodeError::EncodeData)?;
                self.buf.extend_from_slice(&self.compress_buf);
            } else {
                let data_len_size = 1;
                let packet_len = data_len_size + data_len;

                if packet_len >= MAX_PACKET_SIZE as usize {
                    Err(PacketEncodeError::TooLong)?
                }

                let packet_len_size = VarInt(packet_len as i32).written_size();

                let data_prefix_len = packet_len_size + data_len_size;

                self.buf.put_bytes(0, data_prefix_len);
                self.buf
                    .copy_within(start_len..start_len + data_len, start_len + data_prefix_len);

                let mut front = &mut self.buf[start_len..];

                VarInt(packet_len as i32)
                    .encode(&mut front)
                    .map_err(|_| PacketEncodeError::EncodeLength)?;
                // Zero for no compression on this packet.
                VarInt(0)
                    .encode(front)
                    .map_err(|_| PacketEncodeError::EncodeData)?;
            }

            return Ok(());
        }

        let packet_len = data_len;

        if packet_len >= MAX_PACKET_SIZE as usize {
            Err(PacketEncodeError::TooLong)?
        }

        let packet_len_size = VarInt(packet_len as i32).written_size();

        self.buf.put_bytes(0, packet_len_size);
        self.buf
            .copy_within(start_len..start_len + data_len, start_len + packet_len_size);

        let front = &mut self.buf[start_len..];
        VarInt(packet_len as i32)
            .encode(front)
            .map_err(|_| PacketEncodeError::EncodeID)?;
        Ok(())
    }

    pub fn set_encryption(&mut self, key: Option<&[u8; 16]>) {
        if let Some(key) = key {
            assert!(self.cipher.is_none(), "encryption is already enabled");

            self.cipher = Some(Cipher::new_from_slices(key, key).expect("invalid key"));
        } else {
            assert!(self.cipher.is_some(), "encryption is disabled");

            self.cipher = None;
        }
    }

    /// Enables ZLib Compression
    pub fn set_compression(&mut self, compression: Option<CompressionInfo>) {
        self.compression = compression;

        // Reset the compressor with the new compression level
        if let Some(compression) = &self.compression {
            let compression_level = compression.level as i32;

            let level = match CompressionLvl::new(compression_level) {
                Ok(level) => level,
                Err(_) => return,
            };

            self.compressor = Compressor::new(level);
        }
    }

    pub fn take(&mut self) -> BytesMut {
        if let Some(cipher) = &mut self.cipher {
            for chunk in self.buf.chunks_mut(Cipher::block_size()) {
                let gen_arr = GenericArray::from_mut_slice(chunk);
                cipher.encrypt_block_mut(gen_arr);
            }
        }

        self.buf.split()
    }
}

#[derive(Error, Debug)]
pub enum PacketEncodeError {
    #[error("failed to encode packet ID")]
    EncodeID,
    #[error("failed to encode packet Length")]
    EncodeLength,
    #[error("failed to encode packet data")]
    EncodeData,
    #[error("failed to write encoded packet")]
    EncodeFailedWrite,
    #[error("packet exceeds maximum length")]
    TooLong,
    #[error("invalid compression level")]
    InvalidCompressionLevel,
    #[error("compression failed")]
    CompressionFailed,
}

impl PacketEncodeError {
    pub fn kickable(&self) -> bool {
        // We no longer have a connection, so dont try to kick the player, just close
        !matches!(self, Self::EncodeData | Self::EncodeFailedWrite)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bytebuf::packet_id::Packet;
    use crate::client::status::CStatusResponse;
    use crate::VarIntDecodeError;
    use aes::Aes128;
    use cfb8::cipher::AsyncStreamCipher;
    use cfb8::Decryptor as Cfb8Decryptor;
    use libdeflater::{DecompressionError, Decompressor};
    use pumpkin_macros::client_packet;
    use serde::Serialize;

    /// Define a custom packet for testing maximum packet size
    #[derive(Serialize)]
    #[client_packet("status:status_response")]
    pub struct MaxSizePacket {
        data: Vec<u8>,
    }

    impl MaxSizePacket {
        pub fn new(size: usize) -> Self {
            Self {
                data: vec![0xAB; size], // Fill with arbitrary data
            }
        }
    }

    /// Helper function to decode a VarInt from bytes
    fn decode_varint(buffer: &mut &[u8]) -> Result<i32, VarIntDecodeError> {
        VarInt::decode(buffer).map(|varint| varint.0)
    }

    /// Helper function to decompress data using libdeflater's Zlib decompressor
    fn decompress_zlib(data: &[u8], expected_size: usize) -> Result<Vec<u8>, DecompressionError> {
        let mut decompressor = Decompressor::new();
        let mut decompressed = vec![0u8; expected_size];
        let actual_size = decompressor.zlib_decompress(data, &mut decompressed)?;
        decompressed.truncate(actual_size);
        Ok(decompressed)
    }

    /// Helper function to decrypt data using AES-128 CFB-8 mode
    fn decrypt_aes128(encrypted_data: &[u8], key: &[u8; 16], iv: &[u8; 16]) -> Vec<u8> {
        let decryptor = Cfb8Decryptor::<Aes128>::new_from_slices(key, iv).expect("Invalid key/iv");
        let mut decrypted = encrypted_data.to_vec();
        decryptor.decrypt(&mut decrypted);
        decrypted
    }

    /// Helper function to build a packet with optional compression and encryption
    fn build_packet_with_encoder<T: ClientPacket>(
        packet: &T,
        compression_info: Option<CompressionInfo>,
        key: Option<&[u8; 16]>,
    ) -> BytesMut {
        let mut encoder = PacketEncoder::default();

        if let Some(compression) = compression_info {
            encoder.set_compression(Some(compression));
        } else {
            encoder.set_compression(None);
        }

        if let Some(key) = key {
            encoder.set_encryption(Some(key));
        }

        encoder
            .append_packet(packet)
            .expect("Failed to append packet");

        encoder.take()
    }

    /// Test encoding without compression and encryption
    #[test]
    fn test_encode_without_compression_and_encryption() {
        // Create a CStatusResponse packet
        let packet = CStatusResponse::new("{\"description\": \"A Minecraft Server\"}");

        // Build the packet without compression and encryption
        let packet_bytes = build_packet_with_encoder(&packet, None, None);

        // Decode the packet manually to verify correctness
        let mut buffer = &packet_bytes[..];

        // Read packet length VarInt
        let packet_length = decode_varint(&mut buffer).expect("Failed to decode packet length");
        assert_eq!(
            packet_length as usize,
            buffer.len(),
            "Packet length mismatch"
        );

        // Read packet ID VarInt
        let decoded_packet_id = decode_varint(&mut buffer).expect("Failed to decode packet ID");
        assert_eq!(decoded_packet_id, CStatusResponse::PACKET_ID);

        // Remaining buffer is the payload
        // We need to obtain the expected payload
        let mut expected_payload = ByteBuffer::empty();
        packet.write(&mut expected_payload);

        assert_eq!(buffer, expected_payload.buf());
    }

    /// Test encoding with compression
    #[test]
    fn test_encode_with_compression() {
        // Create a CStatusResponse packet
        let packet = CStatusResponse::new("{\"description\": \"A Minecraft Server\"}");

        // Compression threshold is set to 0 to force compression
        let compression_info = CompressionInfo {
            threshold: 0,
            level: 6, // Standard compression level
        };

        // Build the packet with compression enabled
        let packet_bytes = build_packet_with_encoder(&packet, Some(compression_info), None);

        // Decode the packet manually to verify correctness
        let mut buffer = &packet_bytes[..];

        // Read packet length VarInt
        let packet_length = decode_varint(&mut buffer).expect("Failed to decode packet length");
        assert_eq!(
            packet_length as usize,
            buffer.len(),
            "Packet length mismatch"
        );

        // Read data length VarInt (uncompressed data length)
        let data_length = decode_varint(&mut buffer).expect("Failed to decode data length");
        let mut expected_payload = ByteBuffer::empty();
        packet.write(&mut expected_payload);
        let uncompressed_data_length =
            VarInt(CStatusResponse::PACKET_ID).written_size() + expected_payload.buf().len();
        assert_eq!(data_length as usize, uncompressed_data_length);

        // Remaining buffer is the compressed data
        let compressed_data = buffer;

        // Decompress the data
        let decompressed_data = decompress_zlib(compressed_data, data_length as usize)
            .expect("Failed to decompress data");

        // Verify packet ID and payload
        let mut decompressed_buffer = &decompressed_data[..];

        // Read packet ID VarInt
        let decoded_packet_id =
            decode_varint(&mut decompressed_buffer).expect("Failed to decode packet ID");
        assert_eq!(decoded_packet_id, CStatusResponse::PACKET_ID);

        // Remaining buffer is the payload
        assert_eq!(decompressed_buffer, expected_payload.buf());
    }

    /// Test encoding with encryption
    #[test]
    fn test_encode_with_encryption() {
        // Create a CStatusResponse packet
        let packet = CStatusResponse::new("{\"description\": \"A Minecraft Server\"}");

        // Encryption key and IV (IV is the same as key in this case)
        let key = [0x00u8; 16]; // Example key

        // Build the packet with encryption enabled (no compression)
        let packet_bytes = build_packet_with_encoder(&packet, None, Some(&key));

        // Decrypt the packet
        let decrypted_packet = decrypt_aes128(&packet_bytes, &key, &key);

        // Decode the packet manually to verify correctness
        let mut buffer = &decrypted_packet[..];

        // Read packet length VarInt
        let packet_length = decode_varint(&mut buffer).expect("Failed to decode packet length");
        assert_eq!(
            packet_length as usize,
            buffer.len(),
            "Packet length mismatch"
        );

        // Read packet ID VarInt
        let decoded_packet_id = decode_varint(&mut buffer).expect("Failed to decode packet ID");
        assert_eq!(decoded_packet_id, CStatusResponse::PACKET_ID);

        // Remaining buffer is the payload
        let mut expected_payload = ByteBuffer::empty();
        packet.write(&mut expected_payload);

        assert_eq!(buffer, expected_payload.buf());
    }

    /// Test encoding with both compression and encryption
    #[test]
    fn test_encode_with_compression_and_encryption() {
        // Create a CStatusResponse packet
        let packet = CStatusResponse::new("{\"description\": \"A Minecraft Server\"}");

        // Compression threshold is set to 0 to force compression
        let compression_info = CompressionInfo {
            threshold: 0,
            level: 6, // Standard compression level
        };

        // Encryption key and IV (IV is the same as key in this case)
        let key = [0x01u8; 16]; // Example key

        // Build the packet with both compression and encryption enabled
        let packet_bytes = build_packet_with_encoder(&packet, Some(compression_info), Some(&key));

        // Decrypt the packet
        let decrypted_packet = decrypt_aes128(&packet_bytes, &key, &key);

        // Decode the packet manually to verify correctness
        let mut buffer = &decrypted_packet[..];

        // Read packet length VarInt
        let packet_length = decode_varint(&mut buffer).expect("Failed to decode packet length");
        assert_eq!(
            packet_length as usize,
            buffer.len(),
            "Packet length mismatch"
        );

        // Read data length VarInt (uncompressed data length)
        let data_length = decode_varint(&mut buffer).expect("Failed to decode data length");
        let mut expected_payload = ByteBuffer::empty();
        packet.write(&mut expected_payload);
        let uncompressed_data_length =
            VarInt(CStatusResponse::PACKET_ID).written_size() + expected_payload.buf().len();
        assert_eq!(data_length as usize, uncompressed_data_length);

        // Remaining buffer is the compressed data
        let compressed_data = buffer;

        // Decompress the data
        let decompressed_data = decompress_zlib(compressed_data, data_length as usize)
            .expect("Failed to decompress data");

        // Verify packet ID and payload
        let mut decompressed_buffer = &decompressed_data[..];

        // Read packet ID VarInt
        let decoded_packet_id =
            decode_varint(&mut decompressed_buffer).expect("Failed to decode packet ID");
        assert_eq!(decoded_packet_id, CStatusResponse::PACKET_ID);

        // Remaining buffer is the payload
        assert_eq!(decompressed_buffer, expected_payload.buf());
    }

    /// Test encoding with zero-length payload
    #[test]
    fn test_encode_with_zero_length_payload() {
        // Create a CStatusResponse packet with empty payload
        let packet = CStatusResponse::new("");

        // Build the packet without compression and encryption
        let packet_bytes = build_packet_with_encoder(&packet, None, None);

        // Decode the packet manually to verify correctness
        let mut buffer = &packet_bytes[..];

        // Read packet length VarInt
        let packet_length = decode_varint(&mut buffer).expect("Failed to decode packet length");
        assert_eq!(
            packet_length as usize,
            buffer.len(),
            "Packet length mismatch"
        );

        // Read packet ID VarInt
        let decoded_packet_id = decode_varint(&mut buffer).expect("Failed to decode packet ID");
        assert_eq!(decoded_packet_id, CStatusResponse::PACKET_ID);

        // Remaining buffer is the payload (empty)
        let mut expected_payload = ByteBuffer::empty();
        packet.write(&mut expected_payload);

        assert_eq!(
            buffer.len(),
            expected_payload.buf().len(),
            "Payload length mismatch"
        );
        assert_eq!(buffer, expected_payload.buf());
    }

    /// Test encoding with maximum length payload
    #[test]
    fn test_encode_with_maximum_string_length() {
        // Maximum allowed string length is 32767 bytes
        let max_string_length = 32767;
        let payload_str = "A".repeat(max_string_length);
        let packet = CStatusResponse::new(&payload_str);

        // Build the packet without compression and encryption
        let packet_bytes = build_packet_with_encoder(&packet, None, None);

        // Verify that the packet size does not exceed MAX_PACKET_SIZE
        assert!(
            packet_bytes.len() <= MAX_PACKET_SIZE as usize,
            "Packet size exceeds maximum allowed size"
        );

        // Decode the packet manually to verify correctness
        let mut buffer = &packet_bytes[..];

        // Read packet length VarInt
        let packet_length = decode_varint(&mut buffer).expect("Failed to decode packet length");
        assert_eq!(
            packet_length as usize,
            buffer.len(),
            "Packet length mismatch"
        );

        // Read packet ID VarInt
        let decoded_packet_id = decode_varint(&mut buffer).expect("Failed to decode packet ID");
        // Assume packet ID is 0 for CStatusResponse
        assert_eq!(decoded_packet_id, CStatusResponse::PACKET_ID);

        // Remaining buffer is the payload
        let mut expected_payload = ByteBuffer::empty();
        packet.write(&mut expected_payload);

        assert_eq!(buffer, expected_payload.buf());
    }

    /// Test encoding a packet that exceeds MAX_PACKET_SIZE
    #[test]
    #[should_panic(expected = "TooLong")]
    fn test_encode_packet_exceeding_maximum_size() {
        // Create a custom packet with data exceeding MAX_PACKET_SIZE
        let data_size = MAX_PACKET_SIZE as usize + 1; // Exceed by 1 byte
        let packet = MaxSizePacket::new(data_size);

        // Build the packet without compression and encryption
        // This should panic with PacketEncodeError::TooLong
        build_packet_with_encoder(&packet, None, None);
    }

    /// Test encoding with a small payload that should not be compressed
    #[test]
    fn test_encode_small_payload_no_compression() {
        // Create a CStatusResponse packet with small payload
        let packet = CStatusResponse::new("Hi");

        // Compression threshold is set to a value higher than payload length
        let compression_info = CompressionInfo {
            threshold: 10,
            level: 6, // Standard compression level
        };

        // Build the packet with compression enabled
        let packet_bytes = build_packet_with_encoder(&packet, Some(compression_info), None);

        // Decode the packet manually to verify that it was not compressed
        let mut buffer = &packet_bytes[..];

        // Read packet length VarInt
        let packet_length = decode_varint(&mut buffer).expect("Failed to decode packet length");
        assert_eq!(
            packet_length as usize,
            buffer.len(),
            "Packet length mismatch"
        );

        // Read data length VarInt (should be 0 indicating no compression)
        let data_length = decode_varint(&mut buffer).expect("Failed to decode data length");
        assert_eq!(
            data_length, 0,
            "Data length should be 0 indicating no compression"
        );

        // Read packet ID VarInt
        let decoded_packet_id = decode_varint(&mut buffer).expect("Failed to decode packet ID");
        assert_eq!(decoded_packet_id, CStatusResponse::PACKET_ID);

        // Remaining buffer is the payload
        let mut expected_payload = ByteBuffer::empty();
        packet.write(&mut expected_payload);

        assert_eq!(buffer, expected_payload.buf());
    }
}
