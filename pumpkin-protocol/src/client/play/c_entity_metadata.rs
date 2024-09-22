use pumpkin_macros::packet;
use serde::Serialize;

use crate::VarInt;

#[derive(Serialize)]
#[packet(0x58)]
pub struct CSetEntityMetadata<T: Serialize> {
    pub entity_id: VarInt,
    pub metadata: Metadata<T>,
    pub end: u8,
}

impl<T: Serialize> CSetEntityMetadata<T> {
    pub fn new(entity_id: VarInt, metadata: Metadata<T>) -> Self {
        Self {
            entity_id,
            metadata,
            end: 255,
        }
    }
}

#[derive(Serialize)]
pub struct Metadata<T: Serialize> {
    index: u8,
    r#type: VarInt,
    value: T,
}

impl<T: Serialize> Metadata<T> {
    pub fn new(index: u8, r#type: VarInt, value: T) -> Self {
        Self {
            index,
            r#type,
            value,
        }
    }
}
