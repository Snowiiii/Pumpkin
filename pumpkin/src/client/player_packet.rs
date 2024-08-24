use std::f32::consts::PI;

use crate::{
    commands::{handle_command, CommandSender},
    entity::player::{ChatMode, GameMode, Hand, Player},
    server::Server,
    util::math::wrap_degrees,
};
use num_traits::FromPrimitive;
use pumpkin_core::text::TextComponent;
use pumpkin_entity::EntityId;
use pumpkin_protocol::{
    client::play::{
        Animation, CAcknowledgeBlockChange, CBlockUpdate, CEntityAnimation, CEntityVelocity,
        CHeadRot, CHurtAnimation, CPingResponse, CPlayerChatMessage, CUpdateEntityPos,
        CUpdateEntityPosRot, CUpdateEntityRot, CWorldEvent, FilterType,
    },
    position::WorldPosition,
    server::play::{
        Action, ActionType, SChatCommand, SChatMessage, SClientInformationPlay, SConfirmTeleport,
        SInteract, SPlayPingRequest, SPlayerAction, SPlayerCommand, SPlayerPosition,
        SPlayerPositionRotation, SPlayerRotation, SSetCreativeSlot, SSetHeldItem, SSwingArm,
        SUseItemOn, Status,
    },
};
use pumpkin_world::block::BlockFace;
use pumpkin_world::global_registry;

use super::PlayerConfig;

fn modulus(a: f32, b: f32) -> f32 {
    ((a % b) + b) % b
}

/// Handles all Play Packets send by a real Player
impl Player {
    pub fn handle_confirm_teleport(
        &mut self,
        _server: &mut Server,
        confirm_teleport: SConfirmTeleport,
    ) {
        if let Some(id) = self.awaiting_teleport.clone() {
            if id == confirm_teleport.teleport_id {
            } else {
                log::warn!("Teleport id does not match, Weird but okay");
            }
            self.awaiting_teleport = None;
        } else {
            self.client
                .kick("Send Teleport confirm, but we did not teleport")
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
            self.kick(TextComponent::text("Invalid movement"));
            return;
        }
        let entity = &mut self.entity;
        entity.lastx = entity.x;
        entity.lasty = entity.y;
        entity.lastz = entity.z;
        entity.x = Self::clamp_horizontal(position.x);
        entity.y = Self::clamp_vertical(position.feet_y);
        entity.z = Self::clamp_horizontal(position.z);
        // TODO: teleport when moving > 8 block

        // send new position to all other players
        let on_ground = self.on_ground;
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
            self.kick(TextComponent::text("Invalid movement"));
            return;
        }
        if !position_rotation.yaw.is_finite() || !position_rotation.pitch.is_finite() {
            self.kick(TextComponent::text("Invalid rotation"));
            return;
        }
        let entity = &mut self.entity;

        entity.lastx = entity.x;
        entity.lasty = entity.y;
        entity.lastz = entity.z;
        entity.x = Self::clamp_horizontal(position_rotation.x);
        entity.y = Self::clamp_vertical(position_rotation.feet_y);
        entity.z = Self::clamp_horizontal(position_rotation.z);
        entity.yaw = wrap_degrees(position_rotation.yaw) % 360.0;
        entity.pitch = wrap_degrees(position_rotation.pitch).clamp(-90.0, 90.0) % 360.0;

        // send new position to all other players
        let on_ground = self.on_ground;
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
            self.kick(TextComponent::text("Invalid rotation"));
            return;
        }
        let entity = &mut self.entity;
        entity.yaw = wrap_degrees(rotation.yaw) % 360.0;
        entity.pitch = wrap_degrees(rotation.pitch).clamp(-90.0, 90.0) % 360.0;
        // send new position to all other players
        let on_ground = self.on_ground;
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
        if command.entity_id != self.entity.entity_id.into() {
            return;
        }

        if let Some(action) = Action::from_i32(command.action.0) {
            match action {
                pumpkin_protocol::server::play::Action::StartSneaking => self.sneaking = true,
                pumpkin_protocol::server::play::Action::StopSneaking => self.sneaking = false,
                pumpkin_protocol::server::play::Action::LeaveBed => todo!(),
                pumpkin_protocol::server::play::Action::StartSprinting => self.sprinting = true,
                pumpkin_protocol::server::play::Action::StopSprinting => self.sprinting = false,
                pumpkin_protocol::server::play::Action::StartHorseJump => todo!(),
                pumpkin_protocol::server::play::Action::StopHorseJump => todo!(),
                pumpkin_protocol::server::play::Action::OpenVehicleInventory => todo!(),
                pumpkin_protocol::server::play::Action::StartFlyingElytra => {} // TODO
            }
        } else {
            self.kick(TextComponent::text("Invalid player command"))
        }
    }

    pub fn handle_swing_arm(&mut self, server: &mut Server, swing_arm: SSwingArm) {
        let animation = match Hand::from_i32(swing_arm.hand.0).unwrap() {
            Hand::Main => Animation::SwingMainArm,
            Hand::Off => Animation::SwingOffhand,
        };
        let id = self.entity_id();
        server.broadcast_packet_except(
            &[&self.client.token],
            &CEntityAnimation::new(id.into(), animation as u8),
        )
    }

    pub fn handle_chat_message(&mut self, server: &mut Server, chat_message: SChatMessage) {
        dbg!("got message");

        let message = chat_message.message;
        if message.len() > 256 {
            self.kick(TextComponent::text("Oversized message"));
            return;
        }

        // TODO: filter message & validation
        let gameprofile = self.client.gameprofile.as_ref().unwrap();

        server.broadcast_packet(
            self,
            &CPlayerChatMessage::new(
                pumpkin_protocol::uuid::UUID(gameprofile.id),
                1.into(),
                chat_message.signature.as_deref(),
                &message,
                chat_message.timestamp,
                chat_message.salt,
                &[],
                Some(TextComponent::text(&message)),
                FilterType::PassThrough,
                1.into(),
                TextComponent::text(&gameprofile.name.clone()),
                None,
            ),
        )

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
        self.client.config = Some(PlayerConfig {
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
        let action = ActionType::from_i32(interact.typ.0).unwrap();
        if action == ActionType::Attack {
            let entity_id = interact.entity_id;
            // TODO: do validation and stuff
            let config = &server.advanced_config.pvp;
            if config.enabled {
                let attacked_player = server.get_by_entityid(self, entity_id.0 as EntityId);
                self.sneaking = interact.sneaking;
                if let Some(mut player) = attacked_player {
                    let token = player.client.token.clone();
                    let velo = player.velocity;
                    if config.protect_creative && player.gamemode == GameMode::Creative {
                        return;
                    }
                    if config.knockback {
                        let yaw = self.entity.yaw;
                        let strength = 1.0;
                        player.knockback(
                            strength * 0.5,
                            (yaw * (PI / 180.0)).sin() as f64,
                            -(yaw * (PI / 180.0)).cos() as f64,
                        );
                        let packet = &CEntityVelocity::new(
                            &entity_id,
                            player.velocity.x as f32,
                            player.velocity.y as f32,
                            player.velocity.z as f32,
                        );
                        self.velocity = self.velocity.multiply(0.6, 1.0, 0.6);

                        player.velocity = velo;
                        player.client.send_packet(packet);
                    }
                    if config.hurt_animation {
                        // TODO
                        // thats how we prevent borrow errors :c
                        let packet = &CHurtAnimation::new(&entity_id, self.entity.yaw);
                        self.client.send_packet(packet);
                        player.client.send_packet(packet);
                        server.broadcast_packet_except(
                            &[self.client.token.as_ref(), token.as_ref()],
                            &CHurtAnimation::new(&entity_id, 10.0),
                        )
                    }
                    if config.swing {}
                } else {
                    self.kick(TextComponent::text("Interacted with invalid entity id"))
                }
            }
        }
    }
    pub fn handle_player_action(&mut self, server: &mut Server, player_action: SPlayerAction) {
        match Status::from_i32(player_action.status.0).unwrap() {
            Status::StartedDigging => {
                // TODO: do validation
                // TODO: Config
                if self.gamemode == GameMode::Creative {
                    let location = player_action.location;
                    // Block break & block break sound
                    // TODO: currently this is always dirt replace it
                    server.broadcast_packet(self, &CWorldEvent::new(2001, &location, 11, false));
                    // AIR
                    server.broadcast_packet(self, &CBlockUpdate::new(location, 0.into()));
                }
            }
            Status::CancelledDigging => {
                self.current_block_destroy_stage = 0;
            }
            Status::FinishedDigging => {
                // TODO: do validation
                let location = player_action.location;
                // Block break & block break sound
                // TODO: currently this is always dirt replace it
                server.broadcast_packet(self, &CWorldEvent::new(2001, &location, 11, false));
                // AIR
                server.broadcast_packet(self, &CBlockUpdate::new(location, 0.into()));
                // TODO: Send this every tick
                self.client
                    .send_packet(&CAcknowledgeBlockChange::new(player_action.sequence));
            }
            Status::DropItemStack => {
                dbg!("todo");
            }
            Status::DropItem => {
                dbg!("todo");
            }
            Status::ShootArrowOrFinishEating => {
                dbg!("todo");
            }
            Status::SwapItem => {
                dbg!("todo");
            }
        }
    }

    pub fn handle_play_ping_request(&mut self, _server: &mut Server, request: SPlayPingRequest) {
        self.client
            .send_packet(&CPingResponse::new(request.payload));
    }

    pub fn handle_use_item_on(&mut self, server: &mut Server, use_item_on: SUseItemOn) {
        self.client
            .send_packet(&CAcknowledgeBlockChange::new(use_item_on.sequence));

        let location = use_item_on.location;
        let face = BlockFace::from_i32(use_item_on.face.0).unwrap();
        let location = WorldPosition(location.0 + face.to_offset());
        if let Some(item) = self.inventory.held_item() {
            let minecraft_id =
                global_registry::find_minecraft_id(global_registry::ITEM_REGISTRY, item.item_id)
                    .expect("All item ids are in the global registry");
            if let Ok(block_state_id) =
                pumpkin_world::block::block_registry::block_id_and_properties_to_block_state_id(
                    minecraft_id,
                    None,
                )
            {
                server.broadcast_packet(
                    self,
                    &CBlockUpdate::new(location, (block_state_id as i32).into()),
                );
            }
        }
    }

    pub fn handle_set_held_item(&mut self, _server: &mut Server, held: SSetHeldItem) {
        let slot = held.slot;
        if !(0..=8).contains(&slot) {
            self.kick(TextComponent::text("Invalid held slot"))
        }
        self.inventory.set_selected(slot as usize);
    }

    pub fn handle_set_creative_slot(&mut self, _server: &mut Server, packet: SSetCreativeSlot) {
        if self.gamemode != GameMode::Creative {
            self.kick(TextComponent::text(
                "Invalid action, you can only do that if you are in creative",
            ));
            return;
        }
        self.inventory
            .set_slot(packet.slot as usize, packet.clicked_item.to_item(), false);
    }
}
