use crate::VarInt;

#[derive(serde::Deserialize)]
pub struct SConfirmTeleport {
    pub teleport_id: VarInt,
}
