use bytes::Bytes;
use pumpkin_macros::server_packet;

use crate::{
    bytebuf::{ByteBuffer, DeserializerError},
    ServerPacket, VarInt,
};

#[server_packet("play:chat_session_update")]
pub struct SPlayerSession {
    pub session_id: uuid::Uuid,
    pub expires_at: i64,
    pub public_key_len: VarInt,
    pub public_key: Bytes,
    pub signature_len: VarInt,
    pub signature: Bytes,
}

// TODO
impl ServerPacket for SPlayerSession {
    fn read(bytebuf: &mut ByteBuffer) -> Result<Self, DeserializerError> {
        let session_id = bytebuf.get_uuid()?;
        let expires_at = bytebuf.get_i64()?;
        let public_key_len = bytebuf.get_var_int()?;
        let public_key = bytebuf.copy_to_bytes(public_key_len.0 as usize)?;
        let signature_len = bytebuf.get_var_int()?;
        let signature = bytebuf.copy_to_bytes(signature_len.0 as usize)?;
        Ok(Self {
            session_id,
            expires_at,
            public_key_len,
            public_key,
            signature_len,
            signature,
        })
    }
}
