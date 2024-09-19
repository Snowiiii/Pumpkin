use pumpkin_macros::client_packet;
use serde::Serialize;

use crate::VarInt;

use super::ClientboundPlayPackets;

#[derive(Serialize)]
#[client_packet(ClientboundPlayPackets::EntityMetadata as i32)]
pub struct CSetEntityMetadata<T: Serialize> {
    entity_id: VarInt,
    metadata: Metadata<T>,
    end: u8,
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
