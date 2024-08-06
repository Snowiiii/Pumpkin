use pumpkin_protocol::{
    client::play::{CHeadRot, CUpdateEntityPos, CUpdateEntityPosRot, CUpdateEntityRot},
    server::play::{
        SChatCommand, SConfirmTeleport, SPlayerCommand, SPlayerPosition, SPlayerPositionRotation,
        SPlayerRotation,
    },
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

    pub fn handle_position(&mut self, server: &mut Server, position: SPlayerPosition) {
        if position.x.is_nan() || position.feet_y.is_nan() || position.z.is_nan() {
            self.kick("Invalid movement");
        }
        let player = self.player.as_mut().unwrap();
        let entity = &mut player.entity;
        entity.lastx = entity.x;
        entity.lasty = entity.y;
        entity.lastz = entity.z;
        entity.x = position.x;
        entity.y = position.feet_y;
        entity.z = position.z;
        // todo: teleport when moving > 8 block

        // send new position to all other players
        let on_ground = player.on_ground;
        let entity_id = entity.entity_id;
        let (x, lastx) = (entity.x, entity.lastx);
        let (y, lasty) = (entity.y, entity.lasty);
        let (z, lastz) = (entity.z, entity.lastz);

        server.broadcast_packet(
            self,
            CUpdateEntityPos::new(
                entity_id.into(),
                (x * 4096.0 - lastx * 4096.0) as i16,
                (y * 4096.0 - lasty * 4096.0) as i16,
                (z * 4096.0 - lastz * 4096.0) as i16,
                on_ground,
            ),
        );
    }

    pub fn handle_position_rotation(
        &mut self,
        server: &mut Server,
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
        let entity = &mut player.entity;

        entity.x = position_rotation.x;
        entity.y = position_rotation.feet_y;
        entity.z = position_rotation.z;
        entity.yaw = position_rotation.yaw % 360.0;
        entity.pitch = position_rotation.pitch.clamp(-90.0, 90.0) % 360.0;

        // send new position to all other players
        let on_ground = player.on_ground;
        let entity_id = entity.entity_id;
        let (x, lastx) = (entity.x, entity.lastx);
        let (y, lasty) = (entity.y, entity.lasty);
        let (z, lastz) = (entity.z, entity.lastz);
        let yaw = (entity.yaw * 256.0 / 360.0).floor();
        let pitch = (entity.pitch * 256.0 / 360.0).floor();
        let head_yaw = (entity.head_yaw * 256.0 / 360.0).floor();

        server.broadcast_packet(
            self,
            CUpdateEntityPosRot::new(
                entity_id.into(),
                (x * 4096.0 - lastx * 4096.0) as i16,
                (y * 4096.0 - lasty * 4096.0) as i16,
                (z * 4096.0 - lastz * 4096.0) as i16,
                yaw as u8,
                pitch as u8,
                on_ground,
            ),
        );
        server.broadcast_packet(self, CHeadRot::new(entity_id.into(), head_yaw as u8));
    }

    pub fn handle_rotation(&mut self, server: &mut Server, rotation: SPlayerRotation) {
        if !rotation.yaw.is_finite() || !rotation.pitch.is_finite() {
            self.kick("Invalid rotation");
        }
        let player = self.player.as_mut().unwrap();
        let entity = &mut player.entity;
        entity.yaw = rotation.yaw % 360.0;
        entity.pitch = rotation.pitch.clamp(-90.0, 90.0) % 360.0;
        // send new position to all other players
        let on_ground = player.on_ground;
        let entity_id = entity.entity_id;
        let yaw = entity.yaw;
        let pitch = entity.pitch;

        server.broadcast_packet(
            self,
            CUpdateEntityRot::new(entity_id.into(), yaw as u8, pitch as u8, on_ground),
        );
        server.broadcast_packet(self, CHeadRot::new(entity_id.into(), yaw as u8));
    }

    pub fn handle_chat_command(&mut self, _server: &mut Server, command: SChatCommand) {
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
