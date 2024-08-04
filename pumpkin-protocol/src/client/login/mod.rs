use crate::{bytebuf::ByteBuffer, ClientPacket, Property, VarInt};

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

pub struct CLoginSuccess<'a> {
    pub uuid: uuid::Uuid,
    pub username: String, // 16
    pub properties: &'a [Property],
    pub strict_error_handling: bool,
}

impl<'a> CLoginSuccess<'a> {
    pub fn new(
        uuid: uuid::Uuid,
        username: String,
        properties: &'a [Property],
        strict_error_handling: bool,
    ) -> Self {
        Self {
            uuid,
            username,
            properties,
            strict_error_handling,
        }
    }
}

impl<'a> ClientPacket for CLoginSuccess<'a> {
    const PACKET_ID: VarInt = 0x02;

    fn write(&self, bytebuf: &mut ByteBuffer) {
        bytebuf.put_uuid(self.uuid);
        bytebuf.put_string(&self.username);
        bytebuf.put_list::<Property>(self.properties, |p, v| {
            p.put_string(&v.name);
            p.put_string(&v.value);
            // has signature ?
            // todo: for some reason we get "got too many bytes error when using a signature"
            p.put_bool(false);
            // p.put_option(&v.signature, |p,v| p.put_string(v));
        });
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
