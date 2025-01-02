use pumpkin_core::text::TextComponent;

use pumpkin_macros::client_packet;
use serde::Serialize;

use crate::VarInt;

#[derive(Serialize)]
#[client_packet("play:disguised_chat")]
pub struct CDisguisedChatMessage<'a> {
    message: &'a TextComponent,
    chat_type: VarInt,
    sender_name: &'a TextComponent,
    target_name: Option<&'a TextComponent>,
}

impl<'a> CDisguisedChatMessage<'a> {
    pub fn new(
        message: &'a TextComponent,
        chat_type: VarInt,
        sender_name: &'a TextComponent,
        target_name: Option<&'a TextComponent>,
    ) -> Self {
        Self {
            message,
            chat_type,
            sender_name,
            target_name,
        }
    }
}
