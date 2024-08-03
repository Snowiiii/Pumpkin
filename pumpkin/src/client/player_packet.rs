use pumpkin_protocol::server::play::SConfirmTeleport;

use crate::server::Server;

use super::Client;

// implement player packets
pub trait PlayerPacketProcessor {
    fn handle_confirm_teleport(&mut self, server: &mut Server, confirm_teleport: SConfirmTeleport);
}

impl PlayerPacketProcessor for Client {
    fn handle_confirm_teleport(
        &mut self,
        _server: &mut Server,
        confirm_teleport: SConfirmTeleport,
    ) {
        let player = self.player.as_mut().unwrap();
        if let Some(id) = player.awaiting_teleport {
            if id == confirm_teleport.teleport_id {
            } else {
                log::warn!("Teleport id does not match, Weird but okay");
            }
            player.awaiting_teleport = None;
        } else {
            self.kick("Send Teleport confirm, but we did not teleport")
        }
    }
}
