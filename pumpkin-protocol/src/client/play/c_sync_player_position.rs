use pumpkin_core::math::vector3::Vector3;
use pumpkin_macros::client_packet;

use crate::{ClientPacket, PositionFlag, VarInt};

#[client_packet("play:player_position")]
pub struct CSyncPlayerPosition<'a> {
    teleport_id: VarInt,
    x: f64,
    y: f64,
    z: f64,
    delta_x: f64,
    delta_y: f64,
    delta_z: f64,
    yaw: f32,
    pitch: f32,
    releatives: &'a [PositionFlag],
}

impl<'a> CSyncPlayerPosition<'a> {
    pub fn new(
        teleport_id: VarInt,
        position: Vector3<f64>,
        delta: Vector3<f64>,
        yaw: f32,
        pitch: f32,
        releatives: &'a [PositionFlag],
    ) -> Self {
        Self {
            teleport_id,
            x: position.x,
            y: position.y,
            z: position.z,
            delta_x: delta.x,
            delta_y: delta.y,
            delta_z: delta.z,
            yaw,
            pitch,
            releatives,
        }
    }
}

impl<'a> ClientPacket for CSyncPlayerPosition<'a> {
    fn write(&self, bytebuf: &mut crate::bytebuf::ByteBuffer) {
        bytebuf.put_var_int(&self.teleport_id);
        bytebuf.put_f64(self.x);
        bytebuf.put_f64(self.y);
        bytebuf.put_f64(self.z);
        bytebuf.put_f64(self.delta_x);
        bytebuf.put_f64(self.delta_y);
        bytebuf.put_f64(self.delta_z);
        bytebuf.put_f32(self.yaw);
        bytebuf.put_f32(self.pitch);
        // not sure about that
        bytebuf.put_i32(PositionFlag::get_bitfield(self.releatives));
    }
}
