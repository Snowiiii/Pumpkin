use pumpkin_macros::packet;

use crate::{text::TextComponent, ClientPacket, VarInt};

#[derive(Clone)]
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

impl ClientPacket for CDisguisedChatMessage {
    fn write(&self, bytebuf: &mut crate::bytebuf::ByteBuffer) {
        bytebuf.put_slice(&self.message.encode());
        bytebuf.put_var_int(&self.chat_type);
        bytebuf.put_slice(&self.sender_name.encode());
        bytebuf.put_option(&self.target_name, |p, v| p.put_slice(&v.encode()));
    }
}
