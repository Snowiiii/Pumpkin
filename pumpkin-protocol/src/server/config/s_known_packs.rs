use pumpkin_macros::server_packet;

use crate::VarInt;

#[derive(serde::Deserialize)]
#[server_packet("configuration:select_known_packs")]
pub struct SKnownPacks {
    pub known_pack_count: VarInt,
    // known_packs: &'a [KnownPack]
}
