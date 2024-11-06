use aes::cipher::{generic_array::GenericArray, BlockDecryptMut, BlockSizeUser, KeyIvInit};
use bytes::{Buf, BytesMut};
use thiserror::Error;

use std::io::Write;

use bytes::BufMut;
use flate2::write::ZlibDecoder;

use crate::{bytebuf::ByteBuffer, RawPacket, VarEncodedInteger, VarInt, VarIntDecodeError, MAX_PACKET_SIZE};

type Cipher = cfb8::Decryptor<aes::Aes128>;

// Decoder: Client -> Server
// Supports ZLib decoding/decompression
// Supports Aes128 Encyption
#[derive(Default)]
pub struct PacketDecoder {
    buf: BytesMut,
    decompress_buf: BytesMut,
    compression: bool,
    cipher: Option<Cipher>,
}

impl PacketDecoder {
    pub fn decode(&mut self) -> Result<Option<RawPacket>, PacketDecodeError> {
        let mut r = &self.buf[..];

        let packet_len = match VarInt::decode_from_slice(&mut r).map(|x| x.get()) {
            Ok(len) => len,
            Err(VarIntDecodeError::Incomplete) => return Ok(None),
            Err(VarIntDecodeError::TooLarge) => Err(PacketDecodeError::MalformedLength)?,
        };

        if !(0..=MAX_PACKET_SIZE).contains(&packet_len) {
            Err(PacketDecodeError::OutOfBounds)?
        }

        if r.len() < packet_len as usize {
            // Not enough data arrived yet.
            return Ok(None);
        }

        let packet_len_len = VarInt::new(packet_len).written_size();

        let mut data;
        if self.compression {
            r = &r[..packet_len as usize];

            let data_len = VarInt::decode_from_slice(&mut r)
                .map_err(|_| PacketDecodeError::TooLong)?
                .get();

            if !(0..=MAX_PACKET_SIZE).contains(&data_len) {
                Err(PacketDecodeError::OutOfBounds)?
            }

            // Is this packet compressed?
            if data_len > 0 {
                debug_assert!(self.decompress_buf.is_empty());

                self.decompress_buf.put_bytes(0, data_len as usize);

                // TODO: use libdeflater or zune-inflate?
                let mut z = ZlibDecoder::new(&mut self.decompress_buf[..]);

                z.write_all(r)
                    .map_err(|e| PacketDecodeError::FailedWrite(e.to_string()))?;
                z.finish().map_err(|_| PacketDecodeError::FailedFinish)?;

                let total_packet_len = VarInt::new(packet_len).written_size() + packet_len as usize;

                self.buf.advance(total_packet_len);

                data = self.decompress_buf.split();
            } else {
                debug_assert_eq!(data_len, 0);

                let remaining_len = r.len();

                self.buf.advance(packet_len_len + 1);

                data = self.buf.split_to(remaining_len);
            }
        } else {
            // no compression
            self.buf.advance(packet_len_len);
            data = self.buf.split_to(packet_len as usize);
        }

        r = &data[..];
        let packet_id = VarInt::decode_from_slice(&mut r).map_err(|_| PacketDecodeError::DecodeID)?;

        data.advance(data.len() - r.len());
        Ok(Some(RawPacket {
            id: packet_id,
            bytebuf: ByteBuffer::new(data),
        }))
    }

    pub fn set_encryption(&mut self, key: Option<&[u8; 16]>) {
        if let Some(key) = key {
            assert!(self.cipher.is_none(), "encryption is already enabled");

            let mut cipher = Cipher::new_from_slices(key, key).expect("invalid key");

            // Don't forget to decrypt the data we already have.

            Self::decrypt_bytes(&mut cipher, &mut self.buf);

            self.cipher = Some(cipher);
        } else {
            assert!(self.cipher.is_some(), "encryption is already disabled");

            self.cipher = None;
        }
    }

    /// Sets ZLib Deompression
    pub fn set_compression(&mut self, compression: bool) {
        self.compression = compression;
    }

    fn decrypt_bytes(cipher: &mut Cipher, bytes: &mut [u8]) {
        for chunk in bytes.chunks_mut(Cipher::block_size()) {
            let gen_arr = GenericArray::from_mut_slice(chunk);
            cipher.decrypt_block_mut(gen_arr);
        }
    }

    pub fn queue_bytes(&mut self, mut bytes: BytesMut) {
        if let Some(cipher) = &mut self.cipher {
            Self::decrypt_bytes(cipher, &mut bytes);
        }

        self.buf.unsplit(bytes);
    }

    pub fn queue_slice(&mut self, bytes: &[u8]) {
        let len = self.buf.len();

        self.buf.extend_from_slice(bytes);

        if let Some(cipher) = &mut self.cipher {
            let slice = &mut self.buf[len..];
            Self::decrypt_bytes(cipher, slice);
        }
    }

    pub fn take_capacity(&mut self) -> BytesMut {
        self.buf.split_off(self.buf.len())
    }

    pub fn clear(&mut self) {
        self.buf.clear()
    }

    pub fn reserve(&mut self, additional: usize) {
        self.buf.reserve(additional);
    }
}

#[derive(Error, Debug)]
pub enum PacketDecodeError {
    #[error("failed to decode packet ID")]
    DecodeID,
    #[error("failed to write into decoder: {0}")]
    FailedWrite(String),
    #[error("failed to flush decoder")]
    FailedFinish,
    #[error("packet exceeds maximum length")]
    TooLong,
    #[error("packet length is out of bounds")]
    OutOfBounds,
    #[error("malformed packet length VarInt")]
    MalformedLength,
}
