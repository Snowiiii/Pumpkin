use std::io::Write;

use aes::cipher::{generic_array::GenericArray, BlockEncryptMut, BlockSizeUser, KeyIvInit};
use bytes::{BufMut, BytesMut};

use std::io::Read;

use flate2::bufread::ZlibEncoder;
use flate2::Compression;

use crate::{bytebuf::ByteBuffer, ClientPacket, PacketError, VarInt, MAX_PACKET_SIZE};

type Cipher = cfb8::Encryptor<aes::Aes128>;

// Encoder: Server -> Client
#[derive(Default)]
pub struct PacketEncoder {
    buf: BytesMut,
    compress_buf: Vec<u8>,
    compression: Option<(u32, u32)>,
    cipher: Option<Cipher>,
}

impl PacketEncoder {
    pub fn append_packet<P: ClientPacket>(&mut self, packet: &P) -> Result<(), PacketError> {
        let start_len = self.buf.len();

        let mut writer = (&mut self.buf).writer();

        let mut packet_buf = ByteBuffer::empty();
        VarInt(P::PACKET_ID)
            .encode(&mut writer)
            .map_err(|_| PacketError::EncodeID)?;
        packet.write(&mut packet_buf);

        writer
            .write(packet_buf.buf())
            .map_err(|_| PacketError::EncodeFailedWrite)?;

        let data_len = self.buf.len() - start_len;

        if let Some((threshold, compression_level)) = self.compression {
            if data_len > threshold as usize {
                let mut z =
                    ZlibEncoder::new(&self.buf[start_len..], Compression::new(compression_level));

                self.compress_buf.clear();

                let data_len_size = VarInt(data_len as i32).written_size();

                let packet_len = data_len_size + z.read_to_end(&mut self.compress_buf).unwrap();

                if packet_len >= MAX_PACKET_SIZE as usize {
                    Err(PacketError::TooLong)?
                }

                drop(z);

                self.buf.truncate(start_len);

                let mut writer = (&mut self.buf).writer();

                VarInt(packet_len as i32)
                    .encode(&mut writer)
                    .map_err(|_| PacketError::EncodeLength)?;
                VarInt(data_len as i32)
                    .encode(&mut writer)
                    .map_err(|_| PacketError::EncodeData)?;
                self.buf.extend_from_slice(&self.compress_buf);
            } else {
                let data_len_size = 1;
                let packet_len = data_len_size + data_len;

                if packet_len >= MAX_PACKET_SIZE as usize {
                    Err(PacketError::TooLong)?
                }

                let packet_len_size = VarInt(packet_len as i32).written_size();

                let data_prefix_len = packet_len_size + data_len_size;

                self.buf.put_bytes(0, data_prefix_len);
                self.buf
                    .copy_within(start_len..start_len + data_len, start_len + data_prefix_len);

                let mut front = &mut self.buf[start_len..];

                #[allow(clippy::needless_borrows_for_generic_args)]
                VarInt(packet_len as i32)
                    .encode(&mut front)
                    .map_err(|_| PacketError::EncodeLength)?;
                // Zero for no compression on this packet.
                VarInt(0)
                    .encode(front)
                    .map_err(|_| PacketError::EncodeData)?;
            }

            return Ok(());
        }

        let packet_len = data_len;

        if packet_len >= MAX_PACKET_SIZE as usize {
            Err(PacketError::TooLong)?
        }

        let packet_len_size = VarInt(packet_len as i32).written_size();

        self.buf.put_bytes(0, packet_len_size);
        self.buf
            .copy_within(start_len..start_len + data_len, start_len + packet_len_size);

        let front = &mut self.buf[start_len..];
        VarInt(packet_len as i32)
            .encode(front)
            .map_err(|_| PacketError::EncodeID)?;
        Ok(())
    }

    pub fn enable_encryption(&mut self, key: &[u8; 16]) {
        assert!(self.cipher.is_none(), "encryption is already enabled");
        self.cipher = Some(Cipher::new_from_slices(key, key).expect("invalid key"));
    }

    pub fn set_compression(&mut self, compression: Option<(u32, u32)>) {
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
