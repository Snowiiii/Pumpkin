use pumpkin_core::text::TextComponent;

use pumpkin_macros::client_packet;
use serde::Serialize;

#[derive(Serialize)]
#[client_packet("play:set_title_text")]
pub struct CTitleText<'a> {
    title: TextComponent<'a>,
}

impl<'a> CTitleText<'a> {
    pub fn new(title: TextComponent<'a>) -> Self {
        Self { title }
    }
}
