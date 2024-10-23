use serde::Deserialize;

#[derive(Deserialize)]
pub struct SCloseContainer {
    pub window_id: u8,
}
