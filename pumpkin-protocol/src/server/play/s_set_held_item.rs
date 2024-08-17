use pumpkin_macros::packet;
use serde::Deserialize;

#[derive(Deserialize)]
#[packet(0x2F)]
pub struct SSetHeldItem {
    pub slot: i16,
}
