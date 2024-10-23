use serde::Deserialize;

use crate::VarInt;

#[derive(Deserialize)]
pub struct SCloseContainer {
    pub window_id: VarInt,
}
