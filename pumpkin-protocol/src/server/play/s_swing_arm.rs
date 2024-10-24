use crate::VarInt;

#[derive(serde::Deserialize)]
pub struct SSwingArm {
    pub hand: VarInt,
}
