use pumpkin_macros::packet;
use pumpkin_text::TextComponent;
use serde::Serialize;

#[derive(Serialize)]
#[packet(0x09)]
pub struct CConfigAddResourcePack {
    uuid: uuid::Uuid,
    url: String,
    hash: String,
    forced: bool,
    prompt_message: Option<TextComponent>,
}

impl CConfigAddResourcePack {
    pub fn new(
        uuid: uuid::Uuid,
        url: String,
        hash: String,
        forced: bool,
        prompt_message: Option<TextComponent>,
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
