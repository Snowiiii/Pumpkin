use pumpkin_macros::client_packet;
use serde::Serialize;

use crate::VarInt;

use super::ClientboundPlayPackets;

#[derive(Serialize)]
#[client_packet(ClientboundPlayPackets::EntityMetadata as i32)]
pub struct CSetEntityMetadata<T> {
    entity_id: VarInt,
    metadata: Metadata<T>,
    end: u8,
}

impl<T> CSetEntityMetadata<T> {
    pub fn new(entity_id: VarInt, metadata: Metadata<T>) -> Self {
        Self {
            entity_id,
            metadata,
            end: 255,
        }
    }
}

#[derive(Serialize)]
pub struct Metadata<T> {
    index: u8,
    typ: VarInt,
    value: T,
}

impl<T> Metadata<T> {
    pub fn new(index: u8, typ: VarInt, value: T) -> Self {
        Self { index, typ, value }
    }
}
