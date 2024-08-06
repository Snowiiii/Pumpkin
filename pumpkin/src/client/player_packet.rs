use pumpkin_protocol::server::play::{
    SChatCommand, SConfirmTeleport, SPlayerCommand, SPlayerPosition, SPlayerPositionRotation,
    SPlayerRotation,
};

use crate::{
    commands::{handle_command, CommandSender},
    server::Server,
};

use super::Client;

/// Handles all Play Packets send by a real Player
impl Client {
    pub fn handle_confirm_teleport(
        &mut self,
        _server: &mut Server,
        confirm_teleport: SConfirmTeleport,
    ) {
        let player = self.player.as_mut().unwrap();
        if let Some(id) = player.awaiting_teleport.clone() {
            if id == confirm_teleport.teleport_id {
            } else {
                log::warn!("Teleport id does not match, Weird but okay");
            }
            player.awaiting_teleport = None;
        } else {
            self.kick("Send Teleport confirm, but we did not teleport")
        }
    }

    pub fn handle_position(&mut self, _server: &mut Server, position: SPlayerPosition) {
        if position.x.is_nan() || position.feet_y.is_nan() || position.z.is_nan() {
            self.kick("Invalid movement");
        }
        let player = self.player.as_mut().unwrap();
        player.x = position.x;
        player.y = position.feet_y;
        player.z = position.z;
    }

    pub fn handle_position_rotation(
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

    pub fn handle_rotation(&mut self, _server: &mut Server, rotation: SPlayerRotation) {
        if !rotation.yaw.is_finite() || !rotation.pitch.is_finite() {
            self.kick("Invalid rotation");
        }
        let player = self.player.as_mut().unwrap();
        player.yaw = rotation.yaw;
        player.pitch = rotation.pitch;
    }

    pub fn handle_chat_command(&mut self, server: &mut Server, command: SChatCommand) {
        handle_command(&mut CommandSender::Player(self), command.command);
    }

    pub fn handle_player_command(&mut self, _server: &mut Server, command: SPlayerCommand) {
        let player = self.player.as_mut().unwrap();

        if command.entitiy_id != player.entity.entity_id.into() {
            return;
        }

        match command.action {
            pumpkin_protocol::server::play::Action::StartSneaking => player.sneaking = true,
            pumpkin_protocol::server::play::Action::StopSneaking => player.sneaking = false,
            pumpkin_protocol::server::play::Action::LeaveBed => todo!(),
            pumpkin_protocol::server::play::Action::StartSprinting => player.sprinting = true,
            pumpkin_protocol::server::play::Action::StopSprinting => player.sprinting = false,
            pumpkin_protocol::server::play::Action::StartHourseJump => todo!(),
            pumpkin_protocol::server::play::Action::OpenVehicleInventory => todo!(),
            pumpkin_protocol::server::play::Action::StartFlyingElytra => todo!(),
        }
    }
}
