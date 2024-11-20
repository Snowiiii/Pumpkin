use crate::bytebuf::ByteBuffer;
use crate::client::play::bossevent_action::BosseventAction;
use crate::{ClientPacket, VarInt};
use pumpkin_macros::client_packet;

#[client_packet("play:boss_event")]
pub struct CBossEvent<'a> {
    pub uuid: uuid::Uuid,
    pub action: BosseventAction<'a>,
}

impl<'a> CBossEvent<'a> {
    pub fn new(uuid: uuid::Uuid, action: BosseventAction<'a>) -> Self {
        Self { uuid, action }
    }
}

impl<'a> ClientPacket for CBossEvent<'a> {
    fn write(&self, bytebuf: &mut ByteBuffer) {
        bytebuf.put_uuid(&self.uuid);
        let action = &self.action;
        match action {
            BosseventAction::Add {
                title,
                health,
                color,
                division,
                flags,
            } => {
                bytebuf.put_var_int(&VarInt::from(0 as u8));
                bytebuf.put_slice(title.encode().as_slice());
                bytebuf.put_f32(*health);
                bytebuf.put_var_int(&color);
                bytebuf.put_var_int(&division);
                bytebuf.put_u8(*flags);
            }
            BosseventAction::Remove => todo!(),
            BosseventAction::UpdateHealth(health) => {
                bytebuf.put_var_int(&VarInt::from(2 as u8));
                bytebuf.put_f32(*health);
            }
            BosseventAction::UpdateTile(_) => todo!(),
            BosseventAction::UpdateStyle { .. } => todo!(),
            BosseventAction::UpdateFlags(_) => todo!(),
        }
    }
}
