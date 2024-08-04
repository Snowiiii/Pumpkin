use crate::{
    bytebuf::{packet_id::Packet, ByteBuffer},
    ClientPacket, Property, VarInt,
};

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

impl<'a> Packet for CLoginSuccess<'a> {
    const PACKET_ID: VarInt = 0x02;
}

impl<'a> ClientPacket for CLoginSuccess<'a> {
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
