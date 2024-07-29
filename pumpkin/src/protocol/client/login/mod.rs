use crate::protocol::{bytebuf::buffer::ByteBuffer, ClientPacket, VarInt};

pub struct CLoginDisconnect {
    reason: String,
}

impl CLoginDisconnect {
    pub fn new(reason: String) -> Self {
        Self { reason }
    }
}

impl ClientPacket for CLoginDisconnect {
    const PACKET_ID: VarInt = 0;

    fn write(&self, bytebuf: &mut ByteBuffer) {
        bytebuf.write_string(&serde_json::to_string_pretty(&self.reason).unwrap());
    }
}

pub struct CEncryptionRequest<'a> {
    server_id: String, // 20
    public_key_length: VarInt,
    public_key: &'a [u8],
    verify_token_length: VarInt,
    verify_token: &'a [u8],
    should_authenticate: bool,
}

impl<'a> CEncryptionRequest<'a> {
    pub fn new(
        server_id: String,
        public_key_length: VarInt,
        public_key: &'a [u8],
        verify_token_length: VarInt,
        verify_token: &'a [u8],
        should_authenticate: bool,
    ) -> Self {
        Self {
            server_id,
            public_key_length,
            public_key,
            verify_token_length,
            verify_token,
            should_authenticate,
        }
    }
}

impl<'a> ClientPacket for CEncryptionRequest<'a> {
    const PACKET_ID: VarInt = 1;

    fn write(&self, bytebuf: &mut ByteBuffer) {
        bytebuf.write_string_len(self.server_id.as_str(), 20);
        bytebuf.write_var_int(self.public_key_length);
        bytebuf.write_bytes(self.public_key);
        bytebuf.write_var_int(self.verify_token_length);
        bytebuf.write_bytes(self.verify_token);
        bytebuf.write_bool(self.should_authenticate);
    }
}

pub struct CSetCompression {
    threshold: VarInt,
}

pub struct CLoginSuccess {
    pub uuid: uuid::Uuid,
    pub username: String, // 16
    pub num_of_props: VarInt,
    // pub property: Property,
    pub strict_error_handling: bool,
}

impl CLoginSuccess {
    pub fn new(
        uuid: uuid::Uuid,
        username: String,
        num_of_props: VarInt,
        strict_error_handling: bool,
    ) -> Self {
        Self {
            uuid,
            username,
            num_of_props,
            strict_error_handling,
        }
    }
}

pub struct Property {
    name: String,
    value: String,
    is_signed: bool,
    signature: Option<String>,
}

impl ClientPacket for CLoginSuccess {
    const PACKET_ID: VarInt = 2;

    fn write(&self, bytebuf: &mut ByteBuffer) {
        bytebuf.write_uuid(self.uuid);
        bytebuf.write_string(&self.username);
        bytebuf.write_var_int(self.num_of_props);
        // Todo
        bytebuf.write_bool(self.strict_error_handling);
    }
}

impl CSetCompression {
    pub fn new(threshold: VarInt) -> Self {
        Self { threshold }
    }
}

impl ClientPacket for CSetCompression {
    const PACKET_ID: VarInt = 3;

    fn write(&self, bytebuf: &mut ByteBuffer) {
        bytebuf.write_var_int(self.threshold);
    }
}
