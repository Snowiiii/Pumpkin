use crate::bytebuf::ByteBufMut;
use crate::client::play::bossevent_action::BosseventAction;
use crate::{ClientPacket, VarInt};
use bytes::{BufMut, BytesMut};
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

impl ClientPacket for CBossEvent<'_> {
    fn write(&self, bytebuf: &mut BytesMut) {
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
                bytebuf.put_var_int(&VarInt::from(0u8));
                bytebuf.put_slice(&title.encode());
                bytebuf.put_f32(*health);
                bytebuf.put_var_int(color);
                bytebuf.put_var_int(division);
                bytebuf.put_u8(*flags);
            }
            BosseventAction::Remove => {
                bytebuf.put_var_int(&VarInt::from(1u8));
            }
            BosseventAction::UpdateHealth(health) => {
                bytebuf.put_var_int(&VarInt::from(2u8));
                bytebuf.put_f32(*health);
            }
            BosseventAction::UpdateTile(title) => {
                bytebuf.put_var_int(&VarInt::from(3u8));
                bytebuf.put_slice(&title.encode());
            }
            BosseventAction::UpdateStyle { color, dividers } => {
                bytebuf.put_var_int(&VarInt::from(4u8));
                bytebuf.put_var_int(color);
                bytebuf.put_var_int(dividers);
            }
            BosseventAction::UpdateFlags(flags) => {
                bytebuf.put_var_int(&VarInt::from(5u8));
                bytebuf.put_u8(*flags);
            }
        }
    }
}
