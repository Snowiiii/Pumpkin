use num_derive::{FromPrimitive, ToPrimitive};
use num_traits::FromPrimitive;
use pumpkin_core::text::TextComponent;
use pumpkin_macros::packet;
use serde::Serialize;

use crate::{bytebuf::ByteBuffer, uuid::UUID, BitSet, ClientPacket, VarInt};

#[packet(0x39)]
pub struct CPlayerChatMessage<'a> {
    sender: UUID,
    index: VarInt,
    message_signature: Option<&'a [u8]>,
    message: &'a str,
    timestamp: i64,
    salt: i64,
    previous_messages_count: VarInt,
    previous_messages: &'a [PreviousMessage<'a>], // max 20
    unsigned_content: Option<TextComponent<'a>>,
    /// See `FilterType`
    filter_type: VarInt,

    // TODO: Implement
    #[allow(dead_code)]
    filter_type_bits: Option<BitSet<'a>>,
    chat_type: VarInt,
    sender_name: TextComponent<'a>,
    target_name: Option<TextComponent<'a>>,
}

impl<'a> CPlayerChatMessage<'a> {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        sender: UUID,
        index: VarInt,
        message_signature: Option<&'a [u8]>,
        message: &'a str,
        timestamp: i64,
        salt: i64,
        previous_messages: &'a [PreviousMessage<'a>],
        unsigned_content: Option<TextComponent<'a>>,
        filter_type: VarInt,
        filter_type_bits: Option<BitSet<'a>>,
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
            previous_messages_count: previous_messages.len().into(),
            previous_messages,
            unsigned_content,
            filter_type,
            filter_type_bits,
            chat_type,
            sender_name,
            target_name,
        }
    }
}

impl<'a> ClientPacket for CPlayerChatMessage<'a> {
    fn write(&self, bytebuf: &mut ByteBuffer) {
        bytebuf.put_uuid(self.sender.0);
        bytebuf.put_var_int(&self.index);
        bytebuf.put_option(&self.message_signature, |p, v| p.put_slice(v));
        bytebuf.put_string(self.message);
        bytebuf.put_i64(self.timestamp);
        bytebuf.put_i64(self.salt);

        if self.previous_messages_count.0 > 20 {
            // TODO: Assert this is <=20
        }

        bytebuf.put_var_int(&self.previous_messages_count);
        for previous_message in self.previous_messages {
            bytebuf.put_var_int(&previous_message.message_id);
            if let Some(prev_sig) = previous_message.signature {
                // TODO: validate whether this should be None or not
                bytebuf.put_slice(prev_sig);
            }
        }

        bytebuf.put_option(&self.unsigned_content, |p, v| {
            p.put_slice(v.encode().as_slice())
        });

        bytebuf.put_var_int(&self.filter_type);
        match FilterType::from_i32(self.filter_type.0) {
            Some(FilterType::PassThrough) => (),
            Some(FilterType::FullyFiltered) => {
                // TODO: Implement
            }
            Some(FilterType::PartiallyFiltered) => {
                // TODO: Implement
            }
            None => {
                // TODO: Implement
            }
        }

        bytebuf.put_var_int(&self.chat_type);
        bytebuf.put_slice(self.sender_name.encode().as_slice());
        bytebuf.put_option(&self.target_name, |p, v| p.put_slice(v.encode().as_slice()));
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
