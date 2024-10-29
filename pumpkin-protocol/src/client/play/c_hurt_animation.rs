use pumpkin_macros::client_packet;
use serde::Serialize;

use crate::VarInt;

#[derive(Serialize)]
#[client_packet("play:hurt_animation")]
pub struct CHurtAnimation<'a> {
    entity_id: &'a VarInt,
    yaw: f32,
}

impl<'a> CHurtAnimation<'a> {
    pub fn new(entity_id: &'a VarInt, yaw: f32) -> Self {
        Self { entity_id, yaw }
    }
}
