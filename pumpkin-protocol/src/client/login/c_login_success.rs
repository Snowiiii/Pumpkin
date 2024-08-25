use pumpkin_macros::packet;

use crate::{bytebuf::ByteBuffer, ClientPacket, Property};

#[packet(0x02)]
pub struct CLoginSuccess<'a> {
    pub uuid: &'a uuid::Uuid,
    pub username: &'a str, // 16
    pub properties: &'a [Property],
    pub strict_error_handling: bool,
}

impl<'a> CLoginSuccess<'a> {
    pub fn new(
        uuid: &'a uuid::Uuid,
        username: &'a str,
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
    fn write(&self, bytebuf: &mut ByteBuffer) {
        bytebuf.put_uuid(self.uuid);
        bytebuf.put_string(self.username);
        bytebuf.put_list::<Property>(self.properties, |p, v| {
            p.put_string(&v.name);
            p.put_string(&v.value);
            p.put_option(&v.signature, |p, v| p.put_string(v));
        });
        bytebuf.put_bool(self.strict_error_handling);
    }
}
