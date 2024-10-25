#[derive(serde::Deserialize)]
pub struct SPlayerRotation {
    pub yaw: f32,
    pub pitch: f32,
    pub ground: bool,
}
