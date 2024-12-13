use bytes::Bytes;
use pumpkin_macros::server_packet;

use crate::{
    bytebuf::{ByteBuf, ReadingError},
    ServerPacket, VarInt,
};

#[server_packet("login:key")]
pub struct SEncryptionResponse {
    pub shared_secret_length: VarInt,
    pub shared_secret: bytes::Bytes,
    pub verify_token_length: VarInt,
    pub verify_token: bytes::Bytes,
}

impl ServerPacket for SEncryptionResponse {
    fn read(bytebuf: &mut Bytes) -> Result<Self, ReadingError> {
        let shared_secret_length = bytebuf.try_get_var_int()?;
        let shared_secret = bytebuf.try_copy_to_bytes(shared_secret_length.0 as usize)?;
        let verify_token_length = bytebuf.try_get_var_int()?;
        let verify_token = bytebuf.try_copy_to_bytes(shared_secret_length.0 as usize)?;
        Ok(Self {
            shared_secret_length,
            shared_secret,
            verify_token_length,
            verify_token,
        })
    }
}
