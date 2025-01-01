use pumpkin_core::text::TextComponent;

use pumpkin_macros::client_packet;
use serde::Serialize;

#[derive(Serialize)]
#[client_packet("play:set_subtitle_text")]
pub struct CSubtitle<'a> {
    subtitle: &'a TextComponent,
}

impl<'a> CSubtitle<'a> {
    pub fn new(subtitle: &'a TextComponent) -> Self {
        Self { subtitle }
    }
}
