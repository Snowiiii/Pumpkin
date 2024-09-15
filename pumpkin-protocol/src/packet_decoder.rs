use aes::cipher::{generic_array::GenericArray, BlockDecryptMut, BlockSizeUser, KeyIvInit};
use parking_lot::Mutex;

use std::{io::Write, sync::{atomic::{AtomicBool, Ordering}, OnceLock}};

use flate2::write::ZlibDecoder;

use crate::{
    bytebuf::ByteBuffer, PacketError, RawPacket, VarInt,
};

type Cipher = cfb8::Decryptor<aes::Aes128>;

// Decoder: Client -> Server
// Supports ZLib decoding/decompression
// Supports Aes128 Encyption
#[derive(Default)]
pub struct PacketDecoder {
    compression: AtomicBool,
    cipher: OnceLock<Mutex<Cipher>>,
}

impl PacketDecoder {
    /// Enable encryption with the provided key.
    /// Does nothing if already enabled
    pub fn enable_encryption(&self, key: &[u8; 16]) {
        self.cipher.get_or_init(|| {
            Mutex::new(Cipher::new_from_slices(key, key).expect("Invalid Key"))
        });
    }

    /// Enables ZLib Deompression
    pub fn set_compression(&self, compression: bool) {
        self.compression.store(compression, Ordering::Relaxed);
    }

    /// Decrypt and decompress the given bytes (if enabled)
    /// Returns a Raw Packet if succesfull
    /// Caller is responsible for validating packet size
    pub async fn decode(&self, bytes: &mut [u8]) -> Result<RawPacket, PacketError> {
        // If cipher is not initialized, assume encryption is disabled
        if let Some(cipher) = &mut self.cipher.get() {
            let mut cipher = cipher.lock();
            for chunk in bytes.chunks_mut(Cipher::block_size()) {
                let gen_arr = GenericArray::from_mut_slice(chunk);
                cipher.decrypt_block_mut(gen_arr);
            }
        }

        let decompressed_data = if self.compression.load(Ordering::Relaxed) {
            let mut data = vec![];
            let mut decompressor = ZlibDecoder::new(&mut data);

            decompressor
                .write_all(bytes)
                .map_err(|_| PacketError::FailedWrite)?;
            decompressor
                .finish()
                .map_err(|_| PacketError::FailedFinish)?;

            data
        } else {
            bytes.to_vec()
        };
        let mut data_slice = &decompressed_data[..];

        let packet_id = VarInt::decode_with_reader(&mut data_slice)
            .await
            .map_err(|_| PacketError::DecodeID)?;

        Ok(RawPacket {
            id: packet_id,
            bytebuf: ByteBuffer::new(data_slice.into()),
        })
    }
}
