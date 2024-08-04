use crate::{
    bytebuf::{packet_id::Packet, ByteBuffer},
    ClientPacket, Property, VarInt,
};

use super::PlayerAction;

pub struct CPlayerInfoUpdate<'a> {
    pub actions: i8,
    pub players: &'a [Player<'a>],
}

impl<'a> Packet for CPlayerInfoUpdate<'a> {
    const PACKET_ID: VarInt = 0x3E;
}

pub struct Player<'a> {
    pub uuid: uuid::Uuid,
    pub actions: &'a [PlayerAction],
}

impl<'a> CPlayerInfoUpdate<'a> {
    pub fn new(actions: i8, players: &'a [Player]) -> Self {
        Self { actions, players }
    }
}

impl<'a> ClientPacket for CPlayerInfoUpdate<'a> {
    fn write(&self, bytebuf: &mut ByteBuffer) {
        bytebuf.put_i8(self.actions);
        bytebuf.put_list::<Player>(self.players, |p, v| {
            p.put_uuid(v.uuid);
            for action in v.actions {
                match action {
                    PlayerAction::AddPlayer { name, properties } => {
                        p.put_string(name);
                        p.put_list::<Property>(properties, |p, v| {
                            p.put_string(&v.name);
                            p.put_string(&v.value);
                            // has signature ?
                            // todo: for some reason we get "got too many bytes error when using a signature"
                            p.put_bool(false);
                            // todo signature
                        });
                    }
                    PlayerAction::InitializeChat(_) => todo!(),
                    PlayerAction::UpdateGameMode(_) => todo!(),
                    PlayerAction::UpdateListed { listed } => p.put_bool(*listed),
                    PlayerAction::UpdateLatency(_) => todo!(),
                    PlayerAction::UpdateDisplayName(_) => todo!(),
                }
            }
        });
    }
}
