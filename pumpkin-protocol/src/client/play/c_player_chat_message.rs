use pumpkin_core::text::TextComponent;

use pumpkin_macros::client_packet;
use serde::Serialize;

use crate::{BitSet, VarInt};

#[derive(Serialize)]
#[client_packet("play:player_chat")]
pub struct CPlayerChatMessage<'a> {
    #[serde(with = "uuid::serde::compact")]
    sender: uuid::Uuid,
    index: VarInt,
    message_signature: Option<&'a [u8]>,
    message: &'a str,
    timestamp: i64,
    salt: i64,
    previous_messages_count: VarInt,
    previous_messages: &'a [PreviousMessage<'a>], // max 20
    unsigned_content: Option<TextComponent<'a>>,
    filter_type: FilterType<'a>,
    chat_type: VarInt,
    sender_name: TextComponent<'a>,
    target_name: Option<TextComponent<'a>>,
}

impl<'a> CPlayerChatMessage<'a> {
    #[expect(clippy::too_many_arguments)]
    pub fn new(
        sender: uuid::Uuid,
        index: VarInt,
        message_signature: Option<&'a [u8]>,
        message: &'a str,
        timestamp: i64,
        salt: i64,
        previous_messages: &'a [PreviousMessage<'a>],
        unsigned_content: Option<TextComponent<'a>>,
        filter_type: FilterType<'a>,
        chat_type: VarInt,
        sender_name: TextComponent<'a>,
        target_name: Option<TextComponent<'a>>,
    ) -> Self {
        Self {
            sender,
            index,
            message_signature,
            message,
            timestamp,
            salt,
            previous_messages_count: previous_messages.len().try_into().unwrap(),
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

#[derive(Serialize)]
#[repr(i32)]
pub enum FilterType<'a> {
    /// Message is not filtered at all
    PassThrough = 0,
    /// Message is fully filtered
    FullyFiltered = 1,
    /// Only some characters in the message are filtered
    PartiallyFiltered(BitSet<'a>) = 2,
}
