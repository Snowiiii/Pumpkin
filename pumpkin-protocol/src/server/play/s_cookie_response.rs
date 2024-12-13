use crate::{
    bytebuf::{ByteBuf, ReadingError},
    Identifier, ServerPacket, VarInt,
};
use bytes::Bytes;
use pumpkin_macros::server_packet;
use serde::de;

#[server_packet("play:cookie_response")]
/// Response to a Cookie Request (play) from the server.
/// The Notchian (vanilla) server only accepts responses of up to 5 kiB in size.
pub struct SCookieResponse {
    pub key: Identifier,
    pub has_payload: bool,
    pub payload_length: Option<VarInt>,
    pub payload: Option<bytes::Bytes>, // 5120,
}

const MAX_PAYLOAD_SIZE: i32 = 5120;

impl ServerPacket for SCookieResponse {
    fn read(bytebuf: &mut Bytes) -> Result<Self, ReadingError> {
        let key = bytebuf.try_get_string()?;
        let has_payload = bytebuf.try_get_bool()?;

        if !has_payload {
            return Ok(Self {
                key,
                has_payload,
                payload_length: None,
                payload: None,
            });
        }

        let payload_length = bytebuf.try_get_var_int()?;
        let length = payload_length.0;

        if length > MAX_PAYLOAD_SIZE {
            return Err(de::Error::custom(
                "Payload exceeds the maximum allowed size (5120 bytes)",
            ));
        }

        let payload = bytebuf.try_copy_to_bytes(length as usize)?;

        Ok(Self {
            key,
            has_payload,
            payload_length: Some(payload_length),
            payload: Some(payload),
        })
    }
}
