use pumpkin_macros::client_packet;

use crate::{bytebuf::ByteBuffer, ClientPacket};

/// this is a bit ugly, but ClientPacket depends on CommandTree in pumpkin(bin), from where ClientPacket cannot be implemented
#[client_packet("play:commands")]
pub struct CCommands<T> {
    pub data: T,
    pub write_fn: fn(&T, &mut ByteBuffer),
}

impl <T>ClientPacket for CCommands<T> {
    fn write(&self, bytebuf: &mut ByteBuffer) {
        let write_fn = self.write_fn;
        write_fn(&self.data, bytebuf);
    }
}