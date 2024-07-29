
use aes::cipher::{generic_array::GenericArray, BlockDecryptMut, BlockSizeUser, KeyIvInit};
use anyhow::{bail, ensure, Context};
use bytes::{Buf, BytesMut};

use crate::{
    client::MAX_PACKET_SIZE,
    protocol::{bytebuf::buffer::ByteBuffer, RawPacket, VarInt32, VarIntDecodeError},
};

type Cipher = cfb8::Decryptor<aes::Aes128>;

// Decoder: Client -> Server
#[derive(Default)]
pub struct PacketDecoder {
    buf: BytesMut,

    cipher: Option<Cipher>,
}

impl PacketDecoder {
    pub fn decode(&mut self) -> anyhow::Result<Option<RawPacket>> {
        let mut r = &self.buf[..];

        let packet_len = match VarInt32::decode_partial(&mut r) {
            Ok(len) => len,
            Err(VarIntDecodeError::Incomplete) => return Ok(None),
            Err(VarIntDecodeError::TooLarge) => bail!("malformed packet length VarInt"),
        };

        ensure!(
            (0..=MAX_PACKET_SIZE).contains(&packet_len),
            "packet length of {packet_len} is out of bounds"
        );

        if r.len() < packet_len as usize {
            // Not enough data arrived yet.
            return Ok(None);
        }

        let packet_len_len = VarInt32(packet_len).written_size();

        let mut data;

        // no compression

        self.buf.advance(packet_len_len);
        data = self.buf.split_to(packet_len as usize);

        r = &data[..];
        let packet_id = VarInt32::decode(&mut r)
            .context("failed to decode packet ID")?
            .0;

        data.advance(data.len() - r.len());
        dbg!(packet_id);
        Ok(Some(RawPacket {
            len: packet_len,
            id: packet_id,
            bytebuf: ByteBuffer::from_bytes(&data),
        }))
    }

    pub fn enable_encryption(&mut self, key: &[u8; 16]) {
        assert!(self.cipher.is_none(), "encryption is already enabled");

        let mut cipher = Cipher::new_from_slices(key, key).expect("invalid key");

        // Don't forget to decrypt the data we already have.
        Self::decrypt_bytes(&mut cipher, &mut self.buf);

        self.cipher = Some(cipher);
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
