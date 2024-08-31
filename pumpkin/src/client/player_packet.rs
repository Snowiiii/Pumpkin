use std::f32::consts::PI;

use crate::{
    commands::{handle_command, CommandSender},
    entity::player::{ChatMode, Hand, Player},
    server::Server,
    util::math::wrap_degrees,
};
use num_traits::FromPrimitive;
use pumpkin_config::ADVANCED_CONFIG;
use pumpkin_core::{text::TextComponent, GameMode};
use pumpkin_entity::EntityId;
use pumpkin_inventory::WindowType;
use pumpkin_protocol::server::play::{SCloseContainer, SSetPlayerGround, SUseItem};
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
use pumpkin_world::block::{BlockFace, BlockId};
use pumpkin_world::global_registry;

use super::PlayerConfig;

fn modulus(a: f32, b: f32) -> f32 {
    ((a % b) + b) % b
}

/// Handles all Play Packets send by a real Player
/// NEVER TRUST THE CLIENT. HANDLE EVERY ERROR, UNWRAP/EXPECT ARE FORBIDDEN
impl Player {
    pub fn handle_confirm_teleport(
        &mut self,
        _server: &mut Server,
        confirm_teleport: SConfirmTeleport,
    ) {
        if let Some((id, position)) = self.awaiting_teleport.as_ref() {
            if id == &confirm_teleport.teleport_id {
                // we should set the pos now to that we requested in the teleport packet, Is may fixed issues when the client sended position packets while being teleported
                self.entity.x = position.x;
                self.entity.y = position.y;
                self.entity.z = position.z;

                self.awaiting_teleport = None;
            } else {
                self.kick(TextComponent::text("Wrong teleport id"))
            }
        } else {
            self.kick(TextComponent::text(
                "Send Teleport confirm, but we did not teleport",
            ))
        }
    }

    fn clamp_horizontal(pos: f64) -> f64 {
        pos.clamp(-3.0E7, 3.0E7)
    }

    fn clamp_vertical(pos: f64) -> f64 {
        pos.clamp(-2.0E7, 2.0E7)
    }

    pub async fn handle_position(&mut self, _server: &mut Server, position: SPlayerPosition) {
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
        let world = self.world.lock().await;
        world.broadcast_packet(
            &[&self.client.token],
            &CUpdateEntityPos::new(
                entity_id.into(),
                (x * 4096.0 - lastx * 4096.0) as i16,
                (y * 4096.0 - lasty * 4096.0) as i16,
                (z * 4096.0 - lastz * 4096.0) as i16,
                on_ground,
            ),
        );
    }

    pub async fn handle_position_rotation(
        &mut self,
        _server: &mut Server,
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
        let world = self.world.lock().await;

        world.broadcast_packet(
            &[&self.client.token],
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
        world.broadcast_packet(
            &[&self.client.token],
            &CHeadRot::new(entity_id.into(), yaw as u8),
        );
    }

    pub async fn handle_rotation(&mut self, _server: &mut Server, rotation: SPlayerRotation) {
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

        let world = self.world.lock().await;
        let packet = CUpdateEntityRot::new(entity_id.into(), yaw as u8, pitch as u8, on_ground);
        // self.client.send_packet(&packet);
        world.broadcast_packet(&[&self.client.token], &packet);
        let packet = CHeadRot::new(entity_id.into(), yaw as u8);
        //        self.client.send_packet(&packet);
        world.broadcast_packet(&[&self.client.token], &packet);
    }

    pub fn handle_chat_command(&mut self, server: &mut Server, command: SChatCommand) {
        handle_command(&mut CommandSender::Player(self), server, &command.command);
    }

    pub fn handle_player_ground(&mut self, _server: &mut Server, ground: SSetPlayerGround) {
        self.on_ground = ground.on_ground;
    }

    pub async fn handle_player_command(&mut self, _server: &mut Server, command: SPlayerCommand) {
        if command.entity_id != self.entity.entity_id.into() {
            return;
        }

        if let Some(action) = Action::from_i32(command.action.0) {
            match action {
                pumpkin_protocol::server::play::Action::StartSneaking => {
                    if !self.sneaking {
                        self.set_sneaking(true).await
                    }
                }
                pumpkin_protocol::server::play::Action::StopSneaking => {
                    if self.sneaking {
                        self.set_sneaking(false).await
                    }
                }
                pumpkin_protocol::server::play::Action::LeaveBed => todo!(),
                pumpkin_protocol::server::play::Action::StartSprinting => {
                    if !self.sprinting {
                        self.set_sprinting(true).await
                    }
                }
                pumpkin_protocol::server::play::Action::StopSprinting => {
                    if self.sprinting {
                        self.set_sprinting(false).await
                    }
                }
                pumpkin_protocol::server::play::Action::StartHorseJump => todo!(),
                pumpkin_protocol::server::play::Action::StopHorseJump => todo!(),
                pumpkin_protocol::server::play::Action::OpenVehicleInventory => todo!(),
                pumpkin_protocol::server::play::Action::StartFlyingElytra => {} // TODO
            }
        } else {
            self.kick(TextComponent::text("Invalid player command"))
        }
    }

    pub async fn handle_swing_arm(&mut self, _server: &mut Server, swing_arm: SSwingArm) {
        match Hand::from_i32(swing_arm.hand.0) {
            Some(hand) => {
                let animation = match hand {
                    Hand::Main => Animation::SwingMainArm,
                    Hand::Off => Animation::SwingOffhand,
                };
                let id = self.entity_id();
                let world = self.world.lock().await;
                world.broadcast_packet(
                    &[&self.client.token],
                    &CEntityAnimation::new(id.into(), animation as u8),
                )
            }
            None => {
                self.kick(TextComponent::text("Invalid hand"));
            }
        };
    }

    pub async fn handle_chat_message(&mut self, _server: &mut Server, chat_message: SChatMessage) {
        dbg!("got message");

        let message = chat_message.message;
        if message.len() > 256 {
            self.kick(TextComponent::text("Oversized message"));
            return;
        }

        // TODO: filter message & validation
        let gameprofile = &self.gameprofile;

        let world = self.world.lock().await;
        world.broadcast_packet(
            &[&self.client.token],
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
        if let (Some(main_hand), Some(chat_mode)) = (
            Hand::from_i32(client_information.main_hand.into()),
            ChatMode::from_i32(client_information.chat_mode.into()),
        ) {
            self.client.config = Some(PlayerConfig {
                locale: client_information.locale,
                view_distance: client_information.view_distance,
                chat_mode,
                chat_colors: client_information.chat_colors,
                skin_parts: client_information.skin_parts,
                main_hand,
                text_filtering: client_information.text_filtering,
                server_listing: client_information.server_listing,
            });
        } else {
            self.kick(TextComponent::text("Invalid hand or chat type"))
        }
    }

    pub async fn handle_interact(&mut self, _: &mut Server, interact: SInteract) {
        let sneaking = interact.sneaking;
        if self.sneaking != sneaking {
            self.set_sneaking(sneaking).await;
        }
        match ActionType::from_i32(interact.typ.0) {
            Some(action) => match action {
                ActionType::Attack => {
                    let entity_id = interact.entity_id;
                    // TODO: do validation and stuff
                    let config = &ADVANCED_CONFIG.pvp;
                    if config.enabled {
                        let world = self.world.clone();
                        let world = world.lock().await;
                        let attacked_player = world.get_by_entityid(self, entity_id.0 as EntityId);
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
                                world.broadcast_packet(
                                    &[&self.client.token, &token],
                                    &CHurtAnimation::new(&entity_id, 10.0),
                                )
                            }
                            if config.swing {}
                        } else {
                            self.kick(TextComponent::text("Interacted with invalid entity id"))
                        }
                    }
                }
                ActionType::Interact => {
                    dbg!("todo");
                }
                ActionType::InteractAt => {
                    dbg!("todo");
                }
            },
            None => self.kick(TextComponent::text("Invalid action type")),
        }
    }
    pub async fn handle_player_action(
        &mut self,
        _server: &mut Server,
        player_action: SPlayerAction,
    ) {
        match Status::from_i32(player_action.status.0) {
            Some(status) => match status {
                Status::StartedDigging => {
                    if !self.can_interact_with_block_at(&player_action.location, 1.0) {
                        // TODO: maybe log?
                        return;
                    }
                    // TODO: do validation
                    // TODO: Config
                    if self.gamemode == GameMode::Creative {
                        let location = player_action.location;
                        // Block break & block break sound
                        // TODO: currently this is always dirt replace it
                        let world = self.world.lock().await;
                        world.broadcast_packet(
                            &[&self.client.token],
                            &CWorldEvent::new(2001, &location, 11, false),
                        );
                        // AIR
                        world.broadcast_packet(
                            &[&self.client.token],
                            &CBlockUpdate::new(&location, 0.into()),
                        );
                    }
                }
                Status::CancelledDigging => {
                    if !self.can_interact_with_block_at(&player_action.location, 1.0) {
                        // TODO: maybe log?
                        return;
                    }
                    self.current_block_destroy_stage = 0;
                }
                Status::FinishedDigging => {
                    // TODO: do validation
                    let location = player_action.location;
                    if !self.can_interact_with_block_at(&location, 1.0) {
                        // TODO: maybe log?
                        return;
                    }
                    // Block break & block break sound
                    // TODO: currently this is always dirt replace it
                    let world = self.world.lock().await;
                    world.broadcast_packet(
                        &[&self.client.token],
                        &CWorldEvent::new(2001, &location, 11, false),
                    );
                    // AIR
                    world.broadcast_packet(
                        &[&self.client.token],
                        &CBlockUpdate::new(&location, 0.into()),
                    );
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
            },
            None => self.kick(TextComponent::text("Invalid status")),
        }
    }

    pub fn handle_play_ping_request(&mut self, _server: &mut Server, request: SPlayPingRequest) {
        self.client
            .send_packet(&CPingResponse::new(request.payload));
    }

    pub async fn handle_use_item_on(&mut self, _server: &mut Server, use_item_on: SUseItemOn) {
        let location = use_item_on.location;

        if !self.can_interact_with_block_at(&location, 1.0) {
            // TODO: maybe log?
            return;
        }

        if let Some(face) = BlockFace::from_i32(use_item_on.face.0) {
            if let Some(item) = self.inventory.held_item() {
                let minecraft_id = global_registry::find_minecraft_id(
                    global_registry::ITEM_REGISTRY,
                    item.item_id,
                )
                .expect("All item ids are in the global registry");
                if let Ok(block_state_id) = BlockId::new(minecraft_id, None) {
                    let world = self.world.lock().await;
                    world.broadcast_packet(
                        &[&self.client.token],
                        &CBlockUpdate::new(&location, block_state_id.get_id_mojang_repr().into()),
                    );
                    world.broadcast_packet(
                        &[&self.client.token],
                        &CBlockUpdate::new(
                            &WorldPosition(location.0 + face.to_offset()),
                            block_state_id.get_id_mojang_repr().into(),
                        ),
                    );
                }
            }
            self.client
                .send_packet(&CAcknowledgeBlockChange::new(use_item_on.sequence));
        } else {
            self.kick(TextComponent::text("Invalid block face"))
        }
    }

    pub fn handle_use_item(&mut self, _server: &mut Server, _use_item: SUseItem) {
        // TODO: handle packet correctly
        log::error!("An item was used(SUseItem), but the packet is not implemented yet");
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

    // TODO:
    // This function will in the future be used to keep track of if the client is in a valid state.
    // But this is not possible yet
    pub fn handle_close_container(&mut self, _server: &mut Server, packet: SCloseContainer) {
        // window_id 0 represents both 9x1 Generic AND inventory here
        let Some(_window_type) = WindowType::from_u8(packet.window_id) else {
            self.kick(TextComponent::text("Invalid window ID"));
            return;
        };
    }
}
