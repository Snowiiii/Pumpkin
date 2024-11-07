use pumpkin_core::text::TextComponent;
use pumpkin_macros::client_packet;

use crate::{ClientPacket, VarInt};

#[client_packet("play:command_suggestions")]
pub struct CCommandSuggestions<'a> {
    id: VarInt,
    start: VarInt,
    length: VarInt,
    matches: Vec<(String, Option<TextComponent<'a>>)>,
}

impl<'a> CCommandSuggestions<'a> {
    pub fn new(
        id: impl Into<VarInt>,
        start: impl Into<VarInt>,
        length: impl Into<VarInt>,
        matches: Vec<(String, Option<TextComponent<'a>>)>,
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

        bytebuf.put_list(&self.matches, |bytebuf, (suggestion, tooltip)| {
            bytebuf.put_string_len(suggestion, 32767);
            bytebuf.put_bool(tooltip.is_some());
            if let Some(tooltip) = tooltip {
                bytebuf.put_slice(&tooltip.encode());
            }
        })
    }
}
