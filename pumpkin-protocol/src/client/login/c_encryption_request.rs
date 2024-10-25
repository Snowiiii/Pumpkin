use pumpkin_macros::client_packet;
use serde::Serialize;

use crate::VarInt;

use super::ClientboundLoginPackets;

#[derive(Serialize)]
#[client_packet(ClientboundLoginPackets::EncryptionRequest as i32)]
pub struct CEncryptionRequest<'a> {
    server_id: &'a str, // 20
    public_key_length: VarInt,
    public_key: &'a [u8],
    verify_token_length: VarInt,
    verify_token: &'a [u8],
    should_authenticate: bool,
}

impl<'a> CEncryptionRequest<'a> {
    pub fn new(
        server_id: &'a str,
        public_key: &'a [u8],
        verify_token: &'a [u8],
        should_authenticate: bool,
    ) -> Self {
        Self {
            server_id,
            public_key_length: public_key.len().into(),
            public_key,
            verify_token_length: verify_token.len().into(),
            verify_token,
            should_authenticate,
        }
    }
}
