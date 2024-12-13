use bytes::{BufMut, BytesMut};
use pumpkin_core::text::TextComponent;
use pumpkin_macros::client_packet;

use crate::{bytebuf::ByteBufMut, ClientPacket, VarInt};

#[client_packet("play:command_suggestions")]
pub struct CCommandSuggestions<'a> {
    id: VarInt,
    start: VarInt,
    length: VarInt,
    matches: Vec<CommandSuggestion<'a>>,
}

impl<'a> CCommandSuggestions<'a> {
    pub fn new(
        id: VarInt,
        start: VarInt,
        length: VarInt,
        matches: Vec<CommandSuggestion<'a>>,
    ) -> Self {
        Self {
            id,
            start,
            length,
            matches,
        }
    }
}

impl ClientPacket for CCommandSuggestions<'_> {
    fn write(&self, bytebuf: &mut BytesMut) {
        bytebuf.put_var_int(&self.id);
        bytebuf.put_var_int(&self.start);
        bytebuf.put_var_int(&self.length);

        bytebuf.put_list(&self.matches, |bytebuf, suggestion| {
            bytebuf.put_string(suggestion.suggestion);
            bytebuf.put_bool(suggestion.tooltip.is_some());
            if let Some(tooltip) = &suggestion.tooltip {
                bytebuf.put_slice(&tooltip.encode());
            }
        })
    }
}

#[derive(PartialEq, Eq, Hash, Debug)]
pub struct CommandSuggestion<'a> {
    pub suggestion: &'a str,
    pub tooltip: Option<TextComponent<'a>>,
}

impl<'a> CommandSuggestion<'a> {
    pub fn new(suggestion: &'a str, tooltip: Option<TextComponent<'a>>) -> Self {
        Self {
            suggestion,
            tooltip,
        }
    }
}
