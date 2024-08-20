use pumpkin_macros::packet;
use pumpkin_text::TextComponent;
use serde::Serialize;

use crate::VarInt;

#[derive(Serialize)]
#[packet(0x1E)]
pub struct CDisguisedChatMessage<'a> {
    message: TextComponent<'a>,
    chat_type: VarInt,
    sender_name: TextComponent<'a>,
    target_name: Option<TextComponent<'a>>,
}

impl<'a> CDisguisedChatMessage<'a> {
    pub fn new(
        message: TextComponent<'a>,
        chat_type: VarInt,
        sender_name: TextComponent<'a>,
        target_name: Option<TextComponent<'a>>,
    ) -> Self {
        Self {
            message,
            chat_type,
            sender_name,
            target_name,
        }
    }
}
