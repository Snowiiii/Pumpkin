use pumpkin_core::math::vector3::Vector3;
use pumpkin_macros::client_packet;
use serde::Serialize;

use crate::VarInt;

#[derive(Serialize)]
#[client_packet("play:move_entity_pos_rot")]
pub struct CUpdateEntityPosRot {
    entity_id: VarInt,
    delta: Vector3<i16>,
    yaw: u8,
    pitch: u8,
    on_ground: bool,
}

impl CUpdateEntityPosRot {
    pub fn new(
        entity_id: VarInt,
        delta: Vector3<i16>,
        yaw: u8,
        pitch: u8,
        on_ground: bool,
    ) -> Self {
        Self {
            entity_id,
            delta,
            yaw,
            pitch,
            on_ground,
        }
    }
}
