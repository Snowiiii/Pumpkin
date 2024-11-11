use pumpkin_macros::client_packet;

use crate::{bytebuf::ByteBuffer, ClientPacket, Property};

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
    // TODO: Make actions an enum
    pub fn new(actions: i8, players: &'a [Player]) -> Self {
        Self { actions, players }
    }
}

impl<'a> ClientPacket for CPlayerInfoUpdate<'a> {
    fn write(&self, bytebuf: &mut ByteBuffer) {
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
                    PlayerAction::InitializeChat(init_chat) => {
                        p.put_option(&init_chat, |p, v| {
                            p.put_uuid(&v.session_id);
                            p.put_i64(v.expires_at);
                            p.put_var_int(&v.public_key.len().into());
                            p.put_slice(&v.public_key);
                            p.put_var_int(&v.signature.len().into());
                            p.put_slice(&v.signature);
                        });
                    }
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
