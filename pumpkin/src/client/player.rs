use crate::protocol::RawPacket;

use super::Client;

pub struct Player {
    // All networking stuff
    pub client: Client,
}

impl Player {
    pub fn handle_packet(&mut self, _packet: &mut RawPacket) {}
}
