use pumpkin_macros::packet;

use crate::{ClientPacket, VarInt};

#[derive(Clone)]
#[packet(0x01)]
pub struct CSpawnEntity {
    entity_id: VarInt,
    entity_uuid: uuid::Uuid,
    typ: VarInt,
    x: f64,
    y: f64,
    z: f64,
    pitch: u8,    // angle
    yaw: u8,      // angle
    head_yaw: u8, // angle
    data: VarInt,
    velocity_x: i16,
    velocity_y: i16,
    velocity_z: i16,
}

impl CSpawnEntity {
    pub fn new(
        entity_id: VarInt,
        entity_uuid: uuid::Uuid,
        typ: VarInt,
        x: f64,
        y: f64,
        z: f64,
        pitch: u8,    // angle
        yaw: u8,      // angle
        head_yaw: u8, // angle
        data: VarInt,
        velocity_x: i16,
        velocity_y: i16,
        velocity_z: i16,
    ) -> Self {
        Self {
            entity_id,
            entity_uuid,
            typ,
            x,
            y,
            z,
            pitch,
            yaw,
            head_yaw,
            data,
            velocity_x,
            velocity_y,
            velocity_z,
        }
    }
}

impl ClientPacket for CSpawnEntity {
    fn write(&self, bytebuf: &mut crate::bytebuf::ByteBuffer) {
        bytebuf.put_var_int(&self.entity_id);
        bytebuf.put_uuid(self.entity_uuid);
        bytebuf.put_var_int(&self.typ);
        bytebuf.put_f64(self.x);
        bytebuf.put_f64(self.y);
        bytebuf.put_f64(self.z);
        bytebuf.put_u8(self.pitch);
        bytebuf.put_u8(self.yaw);
        bytebuf.put_u8(self.head_yaw);
        bytebuf.put_var_int(&self.data);
        bytebuf.put_i16(self.velocity_x);
        bytebuf.put_i16(self.velocity_y);
        bytebuf.put_i16(self.velocity_z);
    }
}
