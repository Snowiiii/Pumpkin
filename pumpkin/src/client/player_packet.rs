use pumpkin_protocol::server::play::{
    SChatCommand, SConfirmTeleport, SPlayerPosition, SPlayerPositionRotation, SPlayerRotation,
};

use crate::{
    commands::{handle_command, CommandSender},
    server::Server,
};

use super::Client;

// implement player packets
pub trait PlayerPacketProcessor {
    fn handle_confirm_teleport(&mut self, server: &mut Server, confirm_teleport: SConfirmTeleport);

    fn handle_chat_command(&mut self, server: &mut Server, command: SChatCommand);

    fn handle_position(&mut self, server: &mut Server, position: SPlayerPosition);

    fn handle_position_rotation(
        &mut self,
        server: &mut Server,
        position_rotation: SPlayerPositionRotation,
    );
    fn handle_rotation(&mut self, server: &mut Server, rotation: SPlayerRotation);
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

    fn handle_position(&mut self, _server: &mut Server, position: SPlayerPosition) {
        if position.x.is_nan() || position.feet_y.is_nan() || position.z.is_nan() {
            self.kick("Invalid movement");
        }
        let player = self.player.as_mut().unwrap();
        player.x = position.x;
        player.y = position.feet_y;
        player.z = position.z;
    }

    fn handle_position_rotation(
        &mut self,
        _server: &mut Server,
        position_rotation: SPlayerPositionRotation,
    ) {
        if position_rotation.x.is_nan()
            || position_rotation.feet_y.is_nan()
            || position_rotation.z.is_nan()
        {
            self.kick("Invalid movement");
        }
        if !position_rotation.yaw.is_finite() || !position_rotation.pitch.is_finite() {
            self.kick("Invalid rotation");
        }
        let player = self.player.as_mut().unwrap();
        player.x = position_rotation.x;
        player.y = position_rotation.feet_y;
        player.z = position_rotation.z;
        player.yaw = position_rotation.yaw;
        player.pitch = position_rotation.pitch;
    }

    fn handle_rotation(&mut self, _server: &mut Server, rotation: SPlayerRotation) {
        if !rotation.yaw.is_finite() || !rotation.pitch.is_finite() {
            self.kick("Invalid rotation");
        }
        let player = self.player.as_mut().unwrap();
        player.yaw = rotation.yaw;
        player.pitch = rotation.pitch;
    }

    fn handle_chat_command(&mut self, server: &mut Server, command: SChatCommand) {
        handle_command(&mut CommandSender::Player(self), command.command, server);
    }
}
