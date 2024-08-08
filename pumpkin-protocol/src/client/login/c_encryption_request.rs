use pumpkin_macros::packet;
use serde::Serialize;

use crate::{RawBytes, VarInt};

#[derive(Serialize)]
#[packet(0x01)]
pub struct CEncryptionRequest<'a> {
    server_id: &'a str, // 20
    public_key_length: VarInt,
    public_key: RawBytes<'a>,
    verify_token_length: VarInt,
    verify_token: RawBytes<'a>,
    should_authenticate: bool,
}

impl<'a> CEncryptionRequest<'a> {
    pub fn new(
        server_id: &'a str,
        public_key: RawBytes<'a>,
        verify_token: RawBytes<'a>,
        should_authenticate: bool,
    ) -> Self {
        Self {
            server_id,
            public_key_length: public_key.0.len().into(),
            public_key,
            verify_token_length: verify_token.0.len().into(),
            verify_token,
            should_authenticate,
        }
    }
}
