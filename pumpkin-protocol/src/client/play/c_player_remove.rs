use pumpkin_macros::client_packet;
use serde::{ser::SerializeSeq, Serialize};

use crate::VarInt;

use super::ClientboundPlayPackets;

#[derive(Serialize)]
#[client_packet(ClientboundPlayPackets::PlayerInfoRemove as i32)]
pub struct CRemovePlayerInfo<'a> {
    players_count: VarInt,
    #[serde(serialize_with = "serialize_slice_uuids")]
    players: &'a [uuid::Uuid],
}

impl<'a> CRemovePlayerInfo<'a> {
    pub fn new(players_count: VarInt, players: &'a [uuid::Uuid]) -> Self {
        Self {
            players_count,
            players,
        }
    }
}

fn serialize_slice_uuids<S: serde::Serializer>(
    uuids: &[uuid::Uuid],
    serializer: S,
) -> Result<S::Ok, S::Error> {
    let mut seq = serializer.serialize_seq(Some(uuids.len()))?;
    for uuid in uuids {
        seq.serialize_element(uuid.as_bytes())?;
    }
    seq.end()
}
