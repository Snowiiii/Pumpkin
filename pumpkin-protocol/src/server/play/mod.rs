use crate::{ServerPacket, VarInt};

pub struct SConfirmTeleport {
    pub teleport_id: VarInt,
}

impl ServerPacket for SConfirmTeleport {
    const PACKET_ID: VarInt = 0x00;

    fn read(bytebuf: &mut crate::bytebuf::ByteBuffer) -> Self {
        Self {
            teleport_id: bytebuf.get_var_int(),
        }
    }
}
