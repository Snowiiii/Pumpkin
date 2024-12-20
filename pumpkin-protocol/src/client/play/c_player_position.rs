use bytes::{BufMut, BytesMut};
use pumpkin_core::math::vector3::Vector3;
use pumpkin_macros::client_packet;

use crate::{bytebuf::ByteBufMut, ClientPacket, PositionFlag, VarInt};

#[client_packet("play:player_position")]
pub struct CPlayerPosition<'a> {
    teleport_id: VarInt,
    position: Vector3<f64>,
    delta: Vector3<f64>,
    yaw: f32,
    pitch: f32,
    releatives: &'a [PositionFlag],
}

impl<'a> CPlayerPosition<'a> {
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
            position,
            delta,
            yaw,
            pitch,
            releatives,
        }
    }
}

impl ClientPacket for CPlayerPosition<'_> {
    fn write(&self, bytebuf: &mut BytesMut) {
        bytebuf.put_var_int(&self.teleport_id);
        bytebuf.put_f64(self.position.x);
        bytebuf.put_f64(self.position.y);
        bytebuf.put_f64(self.position.z);
        bytebuf.put_f64(self.delta.x);
        bytebuf.put_f64(self.delta.y);
        bytebuf.put_f64(self.delta.z);
        bytebuf.put_f32(self.yaw);
        bytebuf.put_f32(self.pitch);
        // not sure about that
        bytebuf.put_i32(PositionFlag::get_bitfield(self.releatives));
    }
}
