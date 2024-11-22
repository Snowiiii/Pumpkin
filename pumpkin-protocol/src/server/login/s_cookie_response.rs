use crate::bytebuf::{ByteBuffer, DeserializerError};
use crate::{Identifier, ServerPacket, VarInt};
use pumpkin_macros::server_packet;
use serde::de;

#[server_packet("login:cookie_response")]
/// Response to a Cookie Request (login) from the server.
/// The Notchian server only accepts responses of up to 5 kiB in size.
pub struct SCookieResponse {
    pub key: Identifier,
    pub has_payload: bool,
    pub payload_length: Option<VarInt>,
    pub payload: Option<Vec<u8>>, // 5120,
}

impl ServerPacket for SCookieResponse {
    fn read(bytebuf: &mut ByteBuffer) -> Result<Self, DeserializerError> {
        let key = bytebuf.get_string()?;
        let has_payload = bytebuf.get_bool()?;

        if !has_payload {
            return Ok(Self {
                key,
                has_payload,
                payload_length: None,
                payload: None,
            });
        }

        let payload_length = bytebuf
            .get_option(|v| v.get_var_int())?
            .ok_or_else(|| de::Error::custom("Failed to parse payload length"))?;

        let length = payload_length.0;

        if length > 5120 {
            return Err(de::Error::custom(
                "Payload exceeds the maximum allowed size (5120 bytes)",
            ));
        }

        let payload = bytebuf
            .get_option(|v| v.copy_to_bytes(length as usize))?
            .ok_or_else(|| de::Error::custom("Payload missing"))?
            .to_vec();

        Ok(Self {
            key,
            has_payload,
            payload_length: Some(payload_length),
            payload: Some(payload),
        })
    }
}
