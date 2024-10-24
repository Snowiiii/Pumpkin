use crate::VarInt;

#[derive(serde::Deserialize)]
pub struct SKnownPacks {
    pub known_pack_count: VarInt,
    // known_packs: &'a [KnownPack]
}
