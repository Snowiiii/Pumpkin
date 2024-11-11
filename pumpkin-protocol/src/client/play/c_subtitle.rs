use pumpkin_core::text::TextComponent;

use pumpkin_macros::client_packet;
use serde::Serialize;

#[derive(Serialize)]
#[client_packet("play:set_subtitle_text")]
pub struct CSubtitle<'a> {
    subtitle: TextComponent<'a>,
}

impl<'a> CSubtitle<'a> {
    pub fn new(subtitle: TextComponent<'a>) -> Self {
        Self { subtitle }
    }
}
