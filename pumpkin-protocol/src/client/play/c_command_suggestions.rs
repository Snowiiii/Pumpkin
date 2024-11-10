use std::{borrow::Cow, hash::Hash};

use pumpkin_core::text::TextComponent;
use pumpkin_macros::client_packet;

use crate::{ClientPacket, VarInt};

#[client_packet("play:command_suggestions")]
pub struct CCommandSuggestions<'a> {
    id: VarInt,
    start: VarInt,
    length: VarInt,
    matches: Vec<CommandSuggestion<'a>>,
}

impl<'a> CCommandSuggestions<'a> {
    pub fn new(
        id: impl Into<VarInt>,
        start: impl Into<VarInt>,
        length: impl Into<VarInt>,
        matches: Vec<CommandSuggestion<'a>>,
    ) -> Self {
        Self {
            id: id.into(),
            start: start.into(),
            length: length.into(),
            matches,
        }
    }
}

impl<'a> ClientPacket for CCommandSuggestions<'a> {
    fn write(&self, bytebuf: &mut crate::bytebuf::ByteBuffer) {
        bytebuf.put_var_int(&self.id);
        bytebuf.put_var_int(&self.start);
        bytebuf.put_var_int(&self.length);

        bytebuf.put_list(&self.matches, |bytebuf, suggestion| {
            bytebuf.put_string_len(&suggestion.suggestion, 32767);
            bytebuf.put_bool(suggestion.tooltip.is_some());
            if let Some(tooltip) = &suggestion.tooltip {
                bytebuf.put_slice(&tooltip.encode());
            }
        })
    }
}

#[derive(PartialEq, Eq, Hash, Debug)]
pub struct CommandSuggestion<'a> {
    pub suggestion: Cow<'a, str>,
    pub tooltip: Option<TextComponent<'a>>,
}

impl<'a> CommandSuggestion<'a> {
    pub fn new(suggestion: Cow<'a, str>, tooltip: Option<TextComponent<'a>>) -> Self {
        Self {
            suggestion,
            tooltip,
        }
    }
}
