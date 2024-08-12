use num_derive::{FromPrimitive, ToPrimitive};
use pumpkin_macros::packet;
use pumpkin_text::TextComponent;
use serde::Serialize;

use crate::{uuid::UUID, BitSet, VarInt};

#[derive(Serialize, Clone)]
#[packet(0x39)]
pub struct CPlayerChatMessage<'a> {
    sender: UUID,
    index: VarInt,
    message_signature: Option<&'a [u8]>,
    message: String,
    timestamp: i64,
    salt: i64,
    previous_messages_count: VarInt,
    previous_messages: &'a [PreviousMessage<'a>], // max 20
    unsigned_content: Option<TextComponent>,
    /// See `FilterType`
    filter_type: VarInt,
    // TODO: THIS IS A HACK, We currently don't support writing or reading bitsets
    filter_type_bits: bool,
    chat_type: VarInt,
    sender_name: TextComponent,
    target_name: Option<TextComponent>,
}

impl<'a> CPlayerChatMessage<'a> {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        sender: UUID,
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
            previous_messages_count: previous_messages.len().into(),
            previous_messages,
            unsigned_content,
            filter_type,
            filter_type_bits: false,
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

#[derive(FromPrimitive, ToPrimitive)]
pub enum FilterType {
    /// Message is not filtered at all
    PassThrough,
    /// Message is fully filtered
    FullyFiltered,
    /// Only some characters in the message are filtered
    PartiallyFiltered,
}
