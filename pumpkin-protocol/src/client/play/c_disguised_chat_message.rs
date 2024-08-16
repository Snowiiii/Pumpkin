use pumpkin_macros::packet;
use pumpkin_text::TextComponent;
use serde::Serialize;

use crate::VarInt;

#[derive(Serialize)]
#[packet(0x1E)]
pub struct CDisguisedChatMessage {
    message: TextComponent,
    chat_type: VarInt,
    sender_name: TextComponent,
    target_name: Option<TextComponent>,
}

impl CDisguisedChatMessage {
    pub fn new(
        message: TextComponent,
        chat_type: VarInt,
        sender_name: TextComponent,
        target_name: Option<TextComponent>,
    ) -> Self {
        Self {
            message,
            chat_type,
            sender_name,
            target_name,
        }
    }
}
