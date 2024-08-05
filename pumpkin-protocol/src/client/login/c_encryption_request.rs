use pumpkin_macros::packet;

use crate::{
    bytebuf::{packet_id::Packet, ByteBuffer},
    ClientPacket, VarInt,
};

#[packet(0x01)]
pub struct CEncryptionRequest<'a> {
    server_id: &'a str, // 20
    public_key: &'a [u8],
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
            public_key,
            verify_token,
            should_authenticate,
        }
    }
}

impl<'a> ClientPacket for CEncryptionRequest<'a> {
    fn write(&self, bytebuf: &mut ByteBuffer) {
        bytebuf.put_string(self.server_id);
        bytebuf.put_var_int(self.public_key.len() as VarInt);
        bytebuf.put_slice(self.public_key);
        bytebuf.put_var_int(self.verify_token.len() as VarInt);
        bytebuf.put_slice(self.verify_token);
        bytebuf.put_bool(self.should_authenticate);
    }
}
