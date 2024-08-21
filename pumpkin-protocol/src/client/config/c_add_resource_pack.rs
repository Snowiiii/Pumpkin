use pumpkin_core::text::TextComponent;
use pumpkin_macros::packet;
use serde::Serialize;

use crate::uuid::UUID;

#[derive(Serialize)]
#[packet(0x09)]
pub struct CConfigAddResourcePack<'a> {
    uuid: UUID,
    url: &'a str,
    hash: &'a str, // max 40
    forced: bool,
    prompt_message: Option<TextComponent<'a>>,
}

impl<'a> CConfigAddResourcePack<'a> {
    pub fn new(
        uuid: UUID,
        url: &'a str,
        hash: &'a str,
        forced: bool,
        prompt_message: Option<TextComponent<'a>>,
    ) -> Self {
        Self {
            uuid,
            url,
            hash,
            forced,
            prompt_message,
        }
    }
}
