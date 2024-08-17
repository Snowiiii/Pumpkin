use num_traits::FromPrimitive;
use pumpkin_entity::EntityId;
use pumpkin_protocol::{
    client::play::{
        Animation, CBlockUpdate, CEntityAnimation, CEntityVelocity, CHeadRot, CHurtAnimation,
        CSystemChatMessge, CUpdateEntityPos, CUpdateEntityPosRot, CUpdateEntityRot,
    },
    server::play::{
        Action, SChatCommand, SChatMessage, SClientInformationPlay, SConfirmTeleport, SInteract,
        SPlayerAction, SPlayerCommand, SPlayerPosition, SPlayerPositionRotation, SPlayerRotation,
        SSwingArm, SUseItemOn,
    },
};
use pumpkin_text::TextComponent;

use crate::{
    commands::{handle_command, CommandSender},
    entity::player::{ChatMode, GameMode, Hand},
    server::Server,
    util::math::wrap_degrees,
};

use super::{Client, PlayerConfig};

fn modulus(a: f32, b: f32) -> f32 {
    ((a % b) + b) % b
}

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
            &CUpdateEntityPos::new(
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

        entity.lastx = entity.x;
        entity.lasty = entity.y;
        entity.lastz = entity.z;
        entity.x = Self::clamp_horizontal(position_rotation.x);
        entity.y = Self::clamp_vertical(position_rotation.feet_y);
        entity.z = Self::clamp_horizontal(position_rotation.z);
        entity.yaw = wrap_degrees(position_rotation.yaw).clamp(-90.0, 90.0) % 360.0;
        entity.pitch = wrap_degrees(position_rotation.pitch) % 360.0;

        // send new position to all other players
        let on_ground = player.on_ground;
        let entity_id = entity.entity_id;
        let (x, lastx) = (entity.x, entity.lastx);
        let (y, lasty) = (entity.y, entity.lasty);
        let (z, lastz) = (entity.z, entity.lastz);
        let yaw = modulus(entity.yaw * 256.0 / 360.0, 256.0);
        let pitch = modulus(entity.pitch * 256.0 / 360.0, 256.0);
        // let head_yaw = (entity.head_yaw * 256.0 / 360.0).floor();

        server.broadcast_packet(
            self,
            &CUpdateEntityPosRot::new(
                entity_id.into(),
                (x * 4096.0 - lastx * 4096.0) as i16,
                (y * 4096.0 - lasty * 4096.0) as i16,
                (z * 4096.0 - lastz * 4096.0) as i16,
                yaw as u8,
                pitch as u8,
                on_ground,
            ),
        );
        server.broadcast_packet(self, &CHeadRot::new(entity_id.into(), yaw as u8));
    }

    pub fn handle_rotation(&mut self, server: &mut Server, rotation: SPlayerRotation) {
        if !rotation.yaw.is_finite() || !rotation.pitch.is_finite() {
            self.kick("Invalid rotation");
            return;
        }
        let player = self.player.as_mut().unwrap();
        let entity = &mut player.entity;
        entity.yaw = wrap_degrees(rotation.yaw).clamp(-90.0, 90.0) % 360.0;
        entity.pitch = wrap_degrees(rotation.pitch) % 360.0;
        // send new position to all other players
        let on_ground = player.on_ground;
        let entity_id = entity.entity_id;
        let yaw = modulus(entity.yaw * 256.0 / 360.0, 256.0);
        let pitch = modulus(entity.pitch * 256.0 / 360.0, 256.0);
        // let head_yaw = modulus(entity.head_yaw * 256.0 / 360.0, 256.0);

        server.broadcast_packet(
            self,
            &CUpdateEntityRot::new(entity_id.into(), yaw as u8, pitch as u8, on_ground),
        );
        server.broadcast_packet(self, &CHeadRot::new(entity_id.into(), yaw as u8));
    }

    pub fn handle_chat_command(&mut self, _server: &mut Server, command: SChatCommand) {
        handle_command(&mut CommandSender::Player(self), &command.command);
    }

    pub fn handle_player_command(&mut self, _server: &mut Server, command: SPlayerCommand) {
        let player = self.player.as_mut().unwrap();

        if command.entitiy_id != player.entity.entity_id.into() {
            return;
        }

        if let Some(action) = Action::from_i32(command.action.0) {
            match action {
                pumpkin_protocol::server::play::Action::StartSneaking => player.sneaking = true,
                pumpkin_protocol::server::play::Action::StopSneaking => player.sneaking = false,
                pumpkin_protocol::server::play::Action::LeaveBed => todo!(),
                pumpkin_protocol::server::play::Action::StartSprinting => player.sprinting = true,
                pumpkin_protocol::server::play::Action::StopSprinting => player.sprinting = false,
                pumpkin_protocol::server::play::Action::StartHourseJump => todo!(),
                pumpkin_protocol::server::play::Action::StopHourseJump => todo!(),
                pumpkin_protocol::server::play::Action::OpenVehicleInventory => todo!(),
                pumpkin_protocol::server::play::Action::StartFlyingElytra => {} // TODO
            }
        } else {
            self.kick("Invalid player command")
        }
    }

    pub fn handle_swing_arm(&mut self, server: &mut Server, swing_arm: SSwingArm) {
        let animation = match Hand::from_i32(swing_arm.hand.0).unwrap() {
            Hand::Main => Animation::SwingMainArm,
            Hand::Off => Animation::SwingOffhand,
        };
        let player = self.player.as_mut().unwrap();
        let id = player.entity_id();
        server.broadcast_packet_expect(
            &[&self.token],
            &CEntityAnimation::new(id.into(), animation as u8),
        )
    }

    pub fn handle_chat_message(&mut self, server: &mut Server, chat_message: SChatMessage) {
        let message = chat_message.message;
        // TODO: filter message & validation
        let gameprofile = self.gameprofile.as_ref().unwrap();
        dbg!("got message");
        // yeah a "raw system message", the ugly way to do that, but it works
        server.broadcast_packet(
            self,
            &CSystemChatMessge::new(
                TextComponent::from(format!("{}: {}", gameprofile.name, message)),
                false,
            ),
        );

        /*   server.broadcast_packet(
            self,
            CPlayerChatMessage::new(
                pumpkin_protocol::uuid::UUID(gameprofile.id),
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

        */
        /* server.broadcast_packet(
            self,
            &CDisguisedChatMessage::new(
                TextComponent::from(message.clone()),
                VarInt(0),
                gameprofile.name.clone().into(),
                None,
            ),
        ) */
    }

    pub fn handle_client_information_play(
        &mut self,
        _server: &mut Server,
        client_information: SClientInformationPlay,
    ) {
        self.config = Some(PlayerConfig {
            locale: client_information.locale,
            view_distance: client_information.view_distance,
            chat_mode: ChatMode::from_i32(client_information.chat_mode.into()).unwrap(),
            chat_colors: client_information.chat_colors,
            skin_parts: client_information.skin_parts,
            main_hand: Hand::from_i32(client_information.main_hand.into()).unwrap(),
            text_filtering: client_information.text_filtering,
            server_listing: client_information.server_listing,
        });
    }

    pub fn handle_interact(&mut self, server: &mut Server, interact: SInteract) {
        let attacker_player = self.player.as_ref().unwrap();
        let entity_id = interact.entity_id;
        // TODO: do validation and stuff
        let config = &server.advanced_config.pvp;
        if config.enabled {
            let attacked_client = server.get_by_entityid(self, entity_id.0 as EntityId);
            if let Some(mut client) = attacked_client {
                let token = client.token.clone();
                let player = client.player.as_mut().unwrap();
                let velo = player.velocity;
                if config.protect_creative && player.gamemode == GameMode::Creative {
                    return;
                }
                if config.knockback {
                    let pitch = attacker_player.entity.pitch;
                    let strength = 1.0;
                    player.knockback(
                        strength * 0.5,
                        (pitch * 0.017453292).sin() as f64,
                        -(pitch * 0.017453292).cos() as f64,
                    );
                    let packet = &CEntityVelocity::new(
                        &entity_id,
                        player.velocity.x as f32,
                        player.velocity.y as f32,
                        player.velocity.z as f32,
                    );
                    player.velocity = velo;
                    client.send_packet(packet);
                    //  attacker_player.velocity = attacker_player.velocity.multiply(0.6, 1.0, 0.6);
                }
                if config.hurt_animation {
                    // TODO
                    // thats how we prevent borrow errors :c
                    let packet = &CHurtAnimation::new(&entity_id, 10.0);
                    self.send_packet(packet);
                    client.send_packet(packet);
                    server.broadcast_packet_expect(
                        &[self.token.as_ref(), token.as_ref()],
                        &CHurtAnimation::new(&entity_id, 10.0),
                    )
                }
            } else {
                self.kick("Interacted with invalid entitiy id")
            }
        }
    }
    pub fn handle_player_action(&mut self, _server: &mut Server, player_action: SPlayerAction) {}

    pub fn handle_use_item_on(&mut self, server: &mut Server, use_item_on: SUseItemOn) {
        let mut location = use_item_on.location;
        location.y += 2;
        server.broadcast_packet(self, &CBlockUpdate::new(location, 11.into()));
    }
}
