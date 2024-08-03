use pumpkin_protocol::server::play::SConfirmTeleport;

use crate::{entity::player::Player, server::Server};

// implement player packets
pub trait PlayerPacketProcessor {
    fn handle_confirm_teleport(&mut self, server: &mut Server, confirm_teleport: SConfirmTeleport);
}

impl PlayerPacketProcessor for Player {
    fn handle_confirm_teleport(&mut self, server: &mut Server, confirm_teleport: SConfirmTeleport) {
    }
}
