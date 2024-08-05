use pumpkin_macros::packet;

use crate::{
    bytebuf::{packet_id::Packet, ByteBuffer},
    ClientPacket, VarInt,
};

#[packet(0x0B)]
pub struct CChangeDifficulty {
    difficulty: u8,
    locked: bool,
}

impl CChangeDifficulty {
    pub fn new(difficulty: u8, locked: bool) -> Self {
        Self { difficulty, locked }
    }
}

impl ClientPacket for CChangeDifficulty {
    fn write(&self, bytebuf: &mut ByteBuffer) {
        bytebuf.put_u8(self.difficulty);
        bytebuf.put_bool(self.locked);
    }
}
