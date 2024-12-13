use bytes::{BufMut, BytesMut};
use pumpkin_core::math::vector3::Vector3;
use pumpkin_macros::client_packet;

use crate::{bytebuf::ByteBufMut, ClientPacket, PositionFlag, VarInt};

#[client_packet("play:teleport_entity")]
pub struct CTeleportEntity<'a> {
    entity_id: VarInt,
    position: Vector3<f64>,
    delta: Vector3<f64>,
    yaw: f32,
    pitch: f32,
    releatives: &'a [PositionFlag],
    on_ground: bool,
}

impl<'a> CTeleportEntity<'a> {
    pub fn new(
        entity_id: VarInt,
        position: Vector3<f64>,
        delta: Vector3<f64>,
        yaw: f32,
        pitch: f32,
        releatives: &'a [PositionFlag],
        on_ground: bool,
    ) -> Self {
        Self {
            entity_id,
            position,
            delta,
            yaw,
            pitch,
            releatives,
            on_ground,
        }
    }
}

impl ClientPacket for CTeleportEntity<'_> {
    fn write(&self, bytebuf: &mut BytesMut) {
        bytebuf.put_var_int(&self.entity_id);
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
        bytebuf.put_bool(self.on_ground);
    }
}
