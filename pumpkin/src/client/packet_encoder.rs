use std::io::Write;

use aes::cipher::{generic_array::GenericArray, BlockEncryptMut, BlockSizeUser, KeyIvInit};
use anyhow::{ensure, Context};
use bytes::{BufMut, BytesMut};

use crate::{
    client::MAX_PACKET_SIZE,
    protocol::{bytebuf::ByteBuffer, ClientPacket, VarInt32},
};

type Cipher = cfb8::Encryptor<aes::Aes128>;

// Encoder: Server -> Client
#[derive(Default)]
pub struct PacketEncoder {
    buf: BytesMut,

    cipher: Option<Cipher>,
}

impl PacketEncoder {
    pub fn append_packet<P: ClientPacket>(&mut self, packet: P) -> anyhow::Result<()> {
        let start_len = self.buf.len();

        let mut writer = (&mut self.buf).writer();

        let mut packet_buf = ByteBuffer::empty();
        VarInt32(P::PACKET_ID)
            .encode(&mut writer)
            .context("failed to encode packet ID")?;
        packet.write(&mut packet_buf);

        writer.write(packet_buf.buf()).unwrap();

        let data_len = self.buf.len() - start_len;

        if false { // compression
        }
        let packet_len = data_len;

        ensure!(
            packet_len <= MAX_PACKET_SIZE as usize,
            "packet exceeds maximum length"
        );

        let packet_len_size = VarInt32(packet_len as i32).written_size();

        self.buf.put_bytes(0, packet_len_size);
        self.buf
            .copy_within(start_len..start_len + data_len, start_len + packet_len_size);

        let front = &mut self.buf[start_len..];
        VarInt32(packet_len as i32).encode(front)?;
        Ok(())
    }

    pub fn enable_encryption(&mut self, key: &[u8; 16]) {
        assert!(self.cipher.is_none(), "encryption is already enabled");
        self.cipher = Some(Cipher::new_from_slices(key, key).expect("invalid key"));
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
