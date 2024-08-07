use pumpkin_macros::packet;

#[derive(serde::Deserialize)]
#[packet(0x1C)]
pub struct SPlayerRotation {
    pub yaw: f32,
    pub pitch: f32,
    pub ground: bool,
}
