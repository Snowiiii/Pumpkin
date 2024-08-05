use pumpkin_macros::packet;

use crate::{bytebuf::ByteBuffer, text::TextComponent, ClientPacket, VarInt};


#[packet(0x6C)]
pub struct CSystemChatMessge {
  content: TextComponent,
  overlay: bool,
}

impl CSystemChatMessge {
  pub fn new(content: TextComponent, overlay: bool) -> Self {
      Self { content, overlay }
  }
}

impl ClientPacket for CSystemChatMessge {
  fn write(&self, bytebuf: &mut ByteBuffer) {
      bytebuf.put_slice(&self.content.encode());
      bytebuf.put_bool(self.overlay);
  }
}
