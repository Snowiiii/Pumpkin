use pumpkin_macros::server_packet;

use crate::{
    bytebuf::{ByteBuffer, DeserializerError},
    ServerPacket, VarInt,
};

#[server_packet("login:key")]
pub struct SEncryptionResponse {
    pub shared_secret_length: VarInt,
    pub shared_secret: Vec<u8>,
    pub verify_token_length: VarInt,
    pub verify_token: Vec<u8>,
}

impl ServerPacket for SEncryptionResponse {
    fn read(bytebuf: &mut ByteBuffer) -> Result<Self, DeserializerError> {
        let shared_secret_length = bytebuf.get_var_int()?;
        let shared_secret_length_usize = usize::try_from(shared_secret_length.get())
            .map_err(|_| DeserializerError::Message("invalid length returned by client".into()))?;
        let shared_secret = bytebuf.copy_to_bytes(shared_secret_length_usize)?;
        let verify_token_length = bytebuf.get_var_int()?;
        let verify_token = bytebuf.copy_to_bytes(shared_secret_length_usize)?;
        Ok(Self {
            shared_secret_length,
            shared_secret: shared_secret.to_vec(),
            verify_token_length,
            verify_token: verify_token.to_vec(),
        })
    }
}
