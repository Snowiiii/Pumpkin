use serde::Deserialize;

#[derive(Deserialize)]
pub struct SSetPlayerGround {
    pub on_ground: bool,
}
