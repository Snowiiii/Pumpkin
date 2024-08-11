use num_traits::FromPrimitive;
use pumpkin_inventory::WindowType;
use pumpkin_protocol::{
    client::play::{
        Animation, CEntityAnimation, CHeadRot, COpenScreen, CPlayerChatMessage, CUpdateEntityPos,
        CUpdateEntityPosRot, CUpdateEntityRot, FilterType,
    },
    server::play::{
        SChatCommand, SChatMessage, SConfirmTeleport, SPlayerCommand, SPlayerPosition,
        SPlayerPositionRotation, SPlayerRotation, SSwingArm,
    },
    VarInt,
};
use pumpkin_text::TextComponent;

use crate::{
    commands::{handle_command, CommandSender},
    entity::player::Hand,
    server::Server,
    util::math::wrap_degrees,
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

    fn clamp_horizontal(pos: f64) -> f64 {
        pos.clamp(-3.0E7, 3.0E7)
    }

    fn clamp_vertical(pos: f64) -> f64 {
        pos.clamp(-2.0E7, 2.0E7)
    }

    pub fn handle_position(&mut self, server: &mut Server, position: SPlayerPosition) {
        if position.x.is_nan() || position.feet_y.is_nan() || position.z.is_nan() {
            self.kick("Invalid movement");
            return;
        }
        let player = self.player.as_mut().unwrap();
        let entity = &mut player.entity;
        entity.lastx = entity.x;
        entity.lasty = entity.y;
        entity.lastz = entity.z;
        entity.x = Self::clamp_horizontal(position.x);
        entity.y = Self::clamp_vertical(position.feet_y);
        entity.z = Self::clamp_horizontal(position.z);
        // TODO: teleport when moving > 8 block

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
            return;
        }
        if !position_rotation.yaw.is_finite() || !position_rotation.pitch.is_finite() {
            self.kick("Invalid rotation");
            return;
        }
        let player = self.player.as_mut().unwrap();
        let entity = &mut player.entity;

        entity.x = Self::clamp_horizontal(position_rotation.x);
        entity.y = Self::clamp_vertical(position_rotation.feet_y);
        entity.z = Self::clamp_horizontal(position_rotation.z);
        entity.yaw = wrap_degrees(position_rotation.yaw);
        entity.pitch = wrap_degrees(position_rotation.pitch);

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
            return;
        }
        let player = self.player.as_mut().unwrap();
        let entity = &mut player.entity;
        entity.yaw = wrap_degrees(rotation.yaw);
        entity.pitch = wrap_degrees(rotation.pitch);
        // send new position to all other players
        let on_ground = player.on_ground;
        let entity_id = entity.entity_id;
        let yaw = (entity.yaw * 256.0 / 360.0).floor();
        let pitch = (entity.pitch * 256.0 / 360.0).floor();
        let head_yaw = (entity.head_yaw * 256.0 / 360.0).floor();

        server.broadcast_packet(
            self,
            CUpdateEntityRot::new(entity_id.into(), yaw as u8, pitch as u8, on_ground),
        );
        server.broadcast_packet(self, CHeadRot::new(entity_id.into(), head_yaw as u8));
    }

    pub fn handle_chat_command(&mut self, _server: &mut Server, command: SChatCommand) {
        handle_command(&mut CommandSender::Player(self), &command.command);
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

    pub fn handle_swing_arm(&mut self, server: &mut Server, swing_arm: SSwingArm) {
        let animation = match Hand::from_i32(swing_arm.hand.0).unwrap() {
            Hand::Main => Animation::SwingMainArm,
            Hand::Off => Animation::SwingOffhand,
        };
        let player = self.player.as_mut().unwrap();
        let id = player.entity_id();
        server.broadcast_packet_expect(self, CEntityAnimation::new(id.into(), animation as u8))
    }

    pub fn handle_chat_message(&mut self, server: &mut Server, chat_message: SChatMessage) {
        let message = chat_message.message;
        server.broadcast_packet(
            self,
            COpenScreen::new(
                VarInt(0),
                VarInt(WindowType::CraftingTable as i32),
                TextComponent::from("Test Crafter"),
            ),
        );
        // TODO: filter message & validation
        let gameprofile = self.gameprofile.as_ref().unwrap();
        dbg!("got message");
        // yeah a "raw system message", the ugly way to do that, but it works
        // server.broadcast_packet(
        //     self,
        //     CSystemChatMessge::new(
        //         TextComponent::from(format!("{}: {}", gameprofile.name, message)),
        //         false,
        //     ),
        // );
        server.broadcast_packet(
            self,
            CPlayerChatMessage::new(
                gameprofile.id,
                0.into(),
                None,
                message.clone(),
                chat_message.timestamp,
                chat_message.salt,
                &[],
                Some(TextComponent::from(message.clone())),
                pumpkin_protocol::VarInt(FilterType::PassThrough as i32),
                0.into(),
                TextComponent::from(gameprofile.name.clone()),
                None,
            ),
        )
        /* server.broadcast_packet(
            self,
            CDisguisedChatMessage::new(
                TextComponent::from(message.clone()),
                VarInt(0),
                gameprofile.name.clone().into(),
                None,
            ),
        ) */
    }
}
