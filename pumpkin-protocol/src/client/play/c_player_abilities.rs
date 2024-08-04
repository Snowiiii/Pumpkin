use crate::{
    bytebuf::{packet_id::Packet, ByteBuffer},
    ClientPacket, VarInt,
};

pub struct CPlayerAbilities {
    flags: i8,
    flying_speed: f32,
    field_of_view: f32,
}

impl CPlayerAbilities {
    pub fn new(flags: i8, flying_speed: f32, field_of_view: f32) -> Self {
        Self {
            flags,
            flying_speed,
            field_of_view,
        }
    }
}

impl Packet for CPlayerAbilities {
    const PACKET_ID: VarInt = 0x38;
}

impl ClientPacket for CPlayerAbilities {
    fn write(&self, bytebuf: &mut ByteBuffer) {
        bytebuf.put_i8(self.flags);
        bytebuf.put_f32(self.flying_speed);
        bytebuf.put_f32(self.field_of_view);
    }
}
