use bytes::{BufMut, BytesMut};
use pumpkin_macros::client_packet;

use crate::{bytebuf::ByteBufMut, ClientPacket, Property};

use super::PlayerAction;

#[client_packet("play:player_info_update")]
pub struct CPlayerInfoUpdate<'a> {
    pub actions: i8,
    pub players: &'a [Player<'a>],
}

pub struct Player<'a> {
    pub uuid: uuid::Uuid,
    pub actions: Vec<PlayerAction<'a>>,
}

impl<'a> CPlayerInfoUpdate<'a> {
    pub fn new(actions: i8, players: &'a [Player]) -> Self {
        Self { actions, players }
    }
}

impl ClientPacket for CPlayerInfoUpdate<'_> {
    fn write(&self, bytebuf: &mut BytesMut) {
        bytebuf.put_i8(self.actions);
        bytebuf.put_list::<Player>(self.players, |p, v| {
            p.put_uuid(&v.uuid);
            for action in &v.actions {
                match action {
                    PlayerAction::AddPlayer { name, properties } => {
                        p.put_string(name);
                        p.put_list::<Property>(properties, |p, v| {
                            p.put_string(&v.name);
                            p.put_string(&v.value);
                            p.put_option(&v.signature, |p, v| p.put_string(v));
                        });
                    }
                    PlayerAction::InitializeChat(_) => todo!(),
                    PlayerAction::UpdateGameMode(gamemode) => p.put_var_int(gamemode),
                    PlayerAction::UpdateListed(listed) => p.put_bool(*listed),
                    PlayerAction::UpdateLatency(_) => todo!(),
                    PlayerAction::UpdateDisplayName(_) => todo!(),
                    PlayerAction::UpdateListOrder => todo!(),
                }
            }
        });
    }
}
