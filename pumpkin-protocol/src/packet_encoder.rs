use std::io::Write;

use aes::cipher::{generic_array::GenericArray, BlockEncryptMut, BlockSizeUser, KeyIvInit};
use bytes::{BufMut, BytesMut};
use pumpkin_config::compression::CompressionInfo;
use thiserror::Error;

use std::io::Read;

use flate2::bufread::ZlibEncoder;
use flate2::Compression;

use crate::{bytebuf::ByteBuffer, ClientPacket, VarInt, MAX_PACKET_SIZE};

type Cipher = cfb8::Encryptor<aes::Aes128>;

// Encoder: Server -> Client
// Supports ZLib endecoding/compression
// Supports Aes128 Encyption
#[derive(Default, Debug)]
pub struct PacketEncoder {
    buf: BytesMut,
    compress_buf: Vec<u8>,
    compression: Option<CompressionInfo>,
    cipher: Option<Cipher>,
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
                let mut z =
                    ZlibEncoder::new(&self.buf[start_len..], Compression::new(compression.level));

                self.compress_buf.clear();

                let data_len_size = VarInt(data_len as i32).written_size();

                let packet_len = data_len_size + z.read_to_end(&mut self.compress_buf).unwrap();

                if packet_len >= MAX_PACKET_SIZE as usize {
                    Err(PacketEncodeError::TooLong)?
                }

                drop(z);

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
}

impl PacketEncodeError {
    pub fn kickable(&self) -> bool {
        // We no longer have a connection, so dont try to kick the player, just close
        !matches!(self, Self::EncodeData | Self::EncodeFailedWrite)
    }
}
