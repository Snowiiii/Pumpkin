use pumpkin_macros::packet;

use crate::{
    bytebuf::{packet_id::Packet, ByteBuffer},
    ClientPacket, VarInt,
};

#[packet(0x40)]
pub struct CSyncPlayerPostion {
    x: f64,
    y: f64,
    z: f64,
    yaw: f32,
    pitch: f32,
    flags: i8,
    teleport_id: VarInt,
}

impl CSyncPlayerPostion {
    pub fn new(
        x: f64,
        y: f64,
        z: f64,
        yaw: f32,
        pitch: f32,
        flags: i8,
        teleport_id: VarInt,
    ) -> Self {
        Self {
            x,
            y,
            z,
            yaw,
            pitch,
            flags,
            teleport_id,
        }
    }
}

impl ClientPacket for CSyncPlayerPostion {
    fn write(&self, bytebuf: &mut ByteBuffer) {
        bytebuf.put_f64(self.x);
        bytebuf.put_f64(self.y);
        bytebuf.put_f64(self.z);
        bytebuf.put_f32(self.yaw.to_degrees());
        bytebuf.put_f32(self.pitch.to_degrees());
        bytebuf.put_i8(self.flags);
        bytebuf.put_var_int(self.teleport_id);
    }
}
