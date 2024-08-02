use crate::{bytebuf::ByteBuffer, ClientPacket, VarInt};

pub struct CLoginDisconnect<'a> {
    reason: &'a str,
}

impl<'a> CLoginDisconnect<'a> {
    pub fn new(reason: &'a str) -> Self {
        Self { reason }
    }
}

impl<'a> ClientPacket for CLoginDisconnect<'a> {
    const PACKET_ID: VarInt = 0x00;

    fn write(&self, bytebuf: &mut ByteBuffer) {
        bytebuf.put_string(&serde_json::to_string_pretty(&self.reason).unwrap());
    }
}

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
    const PACKET_ID: VarInt = 0x01;

    fn write(&self, bytebuf: &mut ByteBuffer) {
        bytebuf.put_string(self.server_id);
        bytebuf.put_var_int(self.public_key.len() as VarInt);
        bytebuf.put_slice(self.public_key);
        bytebuf.put_var_int(self.verify_token.len() as VarInt);
        bytebuf.put_slice(self.verify_token);
        bytebuf.put_bool(self.should_authenticate);
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
    const PACKET_ID: VarInt = 0x02;

    fn write(&self, bytebuf: &mut ByteBuffer) {
        bytebuf.put_uuid(self.uuid);
        bytebuf.put_string(&self.username);
        bytebuf.put_var_int(self.num_of_props);
        // Todo
        bytebuf.put_bool(self.strict_error_handling);
    }
}

impl CSetCompression {
    pub fn new(threshold: VarInt) -> Self {
        Self { threshold }
    }
}

impl ClientPacket for CSetCompression {
    const PACKET_ID: VarInt = 0x03;

    fn write(&self, bytebuf: &mut ByteBuffer) {
        bytebuf.put_var_int(self.threshold);
    }
}
