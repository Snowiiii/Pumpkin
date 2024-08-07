use num_derive::FromPrimitive;
use serde::Serialize;

use crate::{text::TextComponent, VarInt};

#[derive(Serialize)]
pub struct PlayerChatMessage<'a> {
    sender: uuid::Uuid,
    index: VarInt,
    message_signature: Option<&'a [u8]>,
    message: String,
    timestamp: i64,
    salt: i64,
    previous_messages: &'a [PreviousMessage<'a>], // max 20
    unsigned_content: Option<TextComponent>,
    /// See `FilterType`
    filter_type: VarInt,
    chat_type: VarInt,
    sender_name: TextComponent,
    target_name: Option<TextComponent>,
}

impl<'a> PlayerChatMessage<'a> {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        sender: uuid::Uuid,
        index: VarInt,
        message_signature: Option<&'a [u8]>,
        message: String,
        timestamp: i64,
        salt: i64,
        previous_messages: &'a [PreviousMessage<'a>],
        unsigned_content: Option<TextComponent>,
        filter_type: VarInt,
        chat_type: VarInt,
        sender_name: TextComponent,
        target_name: Option<TextComponent>,
    ) -> Self {
        Self {
            sender,
            index,
            message_signature,
            message,
            timestamp,
            salt,
            previous_messages,
            unsigned_content,
            filter_type,
            chat_type,
            sender_name,
            target_name,
        }
    }
}

#[derive(Serialize)]
pub struct PreviousMessage<'a> {
    message_id: VarInt,
    signature: Option<&'a [u8]>,
}

#[derive(FromPrimitive)]
pub enum FilterType {
    /// Message is not filtered at all
    PassThrough,
    /// Message is fully filtered
    FullyFiltered,
    /// Only some characters in the message are filtered
    PartiallyFiltered,
}
