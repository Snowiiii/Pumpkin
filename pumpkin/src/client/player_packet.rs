use std::sync::Arc;

use crate::{
    command::CommandSender,
    entity::player::{ChatMode, Hand, Player},
    server::Server,
    world::player_chunker,
};
use num_traits::FromPrimitive;
use pumpkin_config::ADVANCED_CONFIG;
use pumpkin_core::math::position::WorldPosition;
use pumpkin_core::{
    math::{vector3::Vector3, wrap_degrees},
    text::TextComponent,
    GameMode,
};
use pumpkin_inventory::{InventoryError, WindowType};
use pumpkin_protocol::server::play::{SCloseContainer, SKeepAlive, SSetPlayerGround, SUseItem};
use pumpkin_protocol::{
    client::play::{
        Animation, CAcknowledgeBlockChange, CEntityAnimation, CHeadRot, CPingResponse,
        CPlayerChatMessage, CUpdateEntityPos, CUpdateEntityPosRot, CUpdateEntityRot, FilterType,
    },
    server::play::{
        Action, ActionType, SChatCommand, SChatMessage, SClientCommand, SClientInformationPlay,
        SConfirmTeleport, SInteract, SPlayPingRequest, SPlayerAbilities, SPlayerAction,
        SPlayerCommand, SPlayerPosition, SPlayerPositionRotation, SPlayerRotation,
        SSetCreativeSlot, SSetHeldItem, SSwingArm, SUseItemOn, Status,
    },
};
use pumpkin_world::block::{block_registry::get_block_by_item, BlockFace};

use super::PlayerConfig;

fn modulus(a: f32, b: f32) -> f32 {
    ((a % b) + b) % b
}

/// Handles all Play Packets send by a real Player
/// NEVER TRUST THE CLIENT. HANDLE EVERY ERROR, UNWRAP/EXPECT ARE FORBIDDEN
impl Player {
    pub async fn handle_confirm_teleport(&self, confirm_teleport: SConfirmTeleport) {
        let mut awaiting_teleport = self.awaiting_teleport.lock().await;
        if let Some((id, position)) = awaiting_teleport.as_ref() {
            if id == &confirm_teleport.teleport_id {
                // we should set the pos now to that we requested in the teleport packet, Is may fixed issues when the client sended position packets while being teleported
                self.living_entity
                    .set_pos(position.x, position.y, position.z);

                *awaiting_teleport = None;
            } else {
                self.kick(TextComponent::text("Wrong teleport id")).await;
            }
        } else {
            self.kick(TextComponent::text(
                "Send Teleport confirm, but we did not teleport",
            ))
            .await;
        }
    }

    fn clamp_horizontal(pos: f64) -> f64 {
        pos.clamp(-3.0E7, 3.0E7)
    }

    fn clamp_vertical(pos: f64) -> f64 {
        pos.clamp(-2.0E7, 2.0E7)
    }

    pub async fn handle_position(self: &Arc<Self>, position: SPlayerPosition) {
        if position.x.is_nan() || position.feet_y.is_nan() || position.z.is_nan() {
            self.kick(TextComponent::text("Invalid movement")).await;
            return;
        }

        let entity = &self.living_entity.entity;
        self.living_entity.set_pos(
            Self::clamp_horizontal(position.x),
            Self::clamp_vertical(position.feet_y),
            Self::clamp_horizontal(position.z),
        );

        let pos = entity.pos.load();
        let last_pos = self.living_entity.last_pos.load();

        entity
            .on_ground
            .store(position.ground, std::sync::atomic::Ordering::Relaxed);

        let entity_id = entity.entity_id;
        let Vector3 { x, y, z } = pos;
        let (last_x, last_y, last_z) = (last_pos.x, last_pos.y, last_pos.z);
        let world = &entity.world;

        // let delta = Vector3::new(x - lastx, y - lasty, z - lastz);
        // let velocity = self.velocity;

        // // Player is falling down fast, we should account for that
        // let max_speed = if self.fall_flying { 300.0 } else { 100.0 };

        // teleport when more than 8 blocks (i guess 8 blocks)
        // TODO: REPLACE * 2.0 by movement packets. see vanilla for details
        // if delta.length_squared() - velocity.length_squared() > max_speed * 2.0 {
        //     self.teleport(x, y, z, self.entity.yaw, self.entity.pitch);
        //     return;
        // }
        // send new position to all other players
        world
            .broadcast_packet_except(
                &[self.gameprofile.id],
                &CUpdateEntityPos::new(
                    entity_id.into(),
                    x.mul_add(4096.0, -(last_x * 4096.0)) as i16,
                    y.mul_add(4096.0, -(last_y * 4096.0)) as i16,
                    z.mul_add(4096.0, -(last_z * 4096.0)) as i16,
                    position.ground,
                ),
            )
            .await;
        player_chunker::update_position(self).await;
    }

    pub async fn handle_position_rotation(
        self: &Arc<Self>,
        position_rotation: SPlayerPositionRotation,
    ) {
        if position_rotation.x.is_nan()
            || position_rotation.feet_y.is_nan()
            || position_rotation.z.is_nan()
        {
            self.kick(TextComponent::text("Invalid movement")).await;
            return;
        }

        if position_rotation.yaw.is_infinite() || position_rotation.pitch.is_infinite() {
            self.kick(TextComponent::text("Invalid rotation")).await;
            return;
        }

        let entity = &self.living_entity.entity;
        self.living_entity.set_pos(
            Self::clamp_horizontal(position_rotation.x),
            Self::clamp_vertical(position_rotation.feet_y),
            Self::clamp_horizontal(position_rotation.z),
        );

        let pos = entity.pos.load();
        let last_pos = self.living_entity.last_pos.load();

        entity.on_ground.store(
            position_rotation.ground,
            std::sync::atomic::Ordering::Relaxed,
        );

        entity.set_rotation(
            wrap_degrees(position_rotation.yaw) % 360.0,
            wrap_degrees(position_rotation.pitch).clamp(-90.0, 90.0) % 360.0,
        );

        let entity_id = entity.entity_id;
        let Vector3 { x, y, z } = pos;
        let (last_x, last_y, last_z) = (last_pos.x, last_pos.y, last_pos.z);

        let yaw = modulus(entity.yaw.load() * 256.0 / 360.0, 256.0);
        let pitch = modulus(entity.pitch.load() * 256.0 / 360.0, 256.0);
        // let head_yaw = (entity.head_yaw * 256.0 / 360.0).floor();
        let world = &entity.world;

        // let delta = Vector3::new(x - lastx, y - lasty, z - lastz);
        // let velocity = self.velocity;

        // // Player is falling down fast, we should account for that
        // let max_speed = if self.fall_flying { 300.0 } else { 100.0 };

        // // teleport when more than 8 blocks (i guess 8 blocks)
        // // TODO: REPLACE * 2.0 by movement packets. see vanilla for details
        // if delta.length_squared() - velocity.length_squared() > max_speed * 2.0 {
        //     self.teleport(x, y, z, yaw, pitch);
        //     return;
        // }
        // send new position to all other players

        world
            .broadcast_packet_except(
                &[self.gameprofile.id],
                &CUpdateEntityPosRot::new(
                    entity_id.into(),
                    x.mul_add(4096.0, -(last_x * 4096.0)) as i16,
                    y.mul_add(4096.0, -(last_y * 4096.0)) as i16,
                    z.mul_add(4096.0, -(last_z * 4096.0)) as i16,
                    yaw as u8,
                    pitch as u8,
                    position_rotation.ground,
                ),
            )
            .await;
        world
            .broadcast_packet_except(
                &[self.gameprofile.id],
                &CHeadRot::new(entity_id.into(), yaw as u8),
            )
            .await;
        player_chunker::update_position(self).await;
    }

    pub async fn handle_rotation(&self, rotation: SPlayerRotation) {
        if !rotation.yaw.is_finite() || !rotation.pitch.is_finite() {
            self.kick(TextComponent::text("Invalid rotation")).await;
            return;
        }
        let entity = &self.living_entity.entity;
        entity
            .on_ground
            .store(rotation.ground, std::sync::atomic::Ordering::Relaxed);
        entity.set_rotation(
            wrap_degrees(rotation.yaw) % 360.0,
            wrap_degrees(rotation.pitch).clamp(-90.0, 90.0) % 360.0,
        );
        // send new position to all other players
        let entity_id = entity.entity_id;
        let yaw = modulus(entity.yaw.load() * 256.0 / 360.0, 256.0);
        let pitch = modulus(entity.pitch.load() * 256.0 / 360.0, 256.0);
        // let head_yaw = modulus(entity.head_yaw * 256.0 / 360.0, 256.0);

        let world = &entity.world;
        let packet =
            CUpdateEntityRot::new(entity_id.into(), yaw as u8, pitch as u8, rotation.ground);
        world
            .broadcast_packet_except(&[self.gameprofile.id], &packet)
            .await;
        let packet = CHeadRot::new(entity_id.into(), yaw as u8);
        world
            .broadcast_packet_except(&[self.gameprofile.id], &packet)
            .await;
    }

    pub async fn handle_chat_command(self: &Arc<Self>, server: &Server, command: SChatCommand) {
        let dispatcher = server.command_dispatcher.clone();
        dispatcher
            .handle_command(
                &mut CommandSender::Player(self.clone()),
                server,
                &command.command,
            )
            .await;
        if ADVANCED_CONFIG.commands.log_console {
            log::info!(
                "Player ({}): executed command /{}",
                self.gameprofile.name,
                command.command
            );
        }
    }

    pub fn handle_player_ground(&self, ground: &SSetPlayerGround) {
        self.living_entity
            .entity
            .on_ground
            .store(ground.on_ground, std::sync::atomic::Ordering::Relaxed);
    }

    pub async fn handle_player_command(&self, command: SPlayerCommand) {
        if command.entity_id != self.entity_id().into() {
            return;
        }

        if let Some(action) = Action::from_i32(command.action.0) {
            let entity = &self.living_entity.entity;
            match action {
                pumpkin_protocol::server::play::Action::StartSneaking => {
                    if !entity.sneaking.load(std::sync::atomic::Ordering::Relaxed) {
                        entity.set_sneaking(true).await;
                    }
                }
                pumpkin_protocol::server::play::Action::StopSneaking => {
                    if entity.sneaking.load(std::sync::atomic::Ordering::Relaxed) {
                        entity.set_sneaking(false).await;
                    }
                }
                pumpkin_protocol::server::play::Action::StartSprinting => {
                    if !entity.sprinting.load(std::sync::atomic::Ordering::Relaxed) {
                        entity.set_sprinting(true).await;
                    }
                }
                pumpkin_protocol::server::play::Action::StopSprinting => {
                    if entity.sprinting.load(std::sync::atomic::Ordering::Relaxed) {
                        entity.set_sprinting(false).await;
                    }
                }
                pumpkin_protocol::server::play::Action::LeaveBed
                | pumpkin_protocol::server::play::Action::StartHorseJump
                | pumpkin_protocol::server::play::Action::StopHorseJump
                | pumpkin_protocol::server::play::Action::OpenVehicleInventory => {
                    log::debug!("todo");
                }
                pumpkin_protocol::server::play::Action::StartFlyingElytra => {
                    let fall_flying = entity.check_fall_flying();
                    if entity
                        .fall_flying
                        .load(std::sync::atomic::Ordering::Relaxed)
                        != fall_flying
                    {
                        entity.set_fall_flying(fall_flying).await;
                    }
                } // TODO
            }
        } else {
            self.kick(TextComponent::text("Invalid player command"))
                .await;
        }
    }

    pub async fn handle_swing_arm(&self, swing_arm: SSwingArm) {
        match Hand::from_i32(swing_arm.hand.0) {
            Some(hand) => {
                let animation = match hand {
                    Hand::Main => Animation::SwingMainArm,
                    Hand::Off => Animation::SwingOffhand,
                };
                let id = self.entity_id();
                let world = &self.living_entity.entity.world;
                world
                    .broadcast_packet_except(
                        &[self.gameprofile.id],
                        &CEntityAnimation::new(id.into(), animation as u8),
                    )
                    .await;
            }
            None => {
                self.kick(TextComponent::text("Invalid hand")).await;
            }
        };
    }

    pub async fn handle_chat_message(&self, chat_message: SChatMessage) {
        let message = chat_message.message;
        if message.len() > 256 {
            self.kick(TextComponent::text("Oversized message")).await;
            return;
        }

        // TODO: filter message & validation
        let gameprofile = &self.gameprofile;
        log::info!("<chat>{}: {}", gameprofile.name, message);

        let entity = &self.living_entity.entity;
        let world = &entity.world;
        world
            .broadcast_packet_all(&CPlayerChatMessage::new(
                gameprofile.id,
                1.into(),
                chat_message.signature.as_deref(),
                &message,
                chat_message.timestamp,
                chat_message.salt,
                &[],
                Some(TextComponent::text(&message)),
                FilterType::PassThrough,
                1.into(),
                TextComponent::text(&gameprofile.name),
                None,
            ))
            .await;

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

    pub async fn handle_client_information(&self, client_information: SClientInformationPlay) {
        if let (Some(main_hand), Some(chat_mode)) = (
            Hand::from_i32(client_information.main_hand.into()),
            ChatMode::from_i32(client_information.chat_mode.into()),
        ) {
            *self.config.lock().await = PlayerConfig {
                locale: client_information.locale,
                // A Negative view distance would be impossible and make no sense right ?, Mojang: Lets make is signed :D
                view_distance: client_information.view_distance as u8,
                chat_mode,
                chat_colors: client_information.chat_colors,
                skin_parts: client_information.skin_parts,
                main_hand,
                text_filtering: client_information.text_filtering,
                server_listing: client_information.server_listing,
            };
        } else {
            self.kick(TextComponent::text("Invalid hand or chat type"))
                .await;
        }
    }

    pub async fn handle_client_status(self: &Arc<Self>, client_status: SClientCommand) {
        match client_status.action_id.0 {
            0 => {
                if self.living_entity.health.load() > 0.0 {
                    return;
                }
                self.respawn(false).await;
                // TODO: hardcore set spectator
            }
            1 => {
                // request stats
                log::debug!("todo");
            }
            _ => {
                self.kick(TextComponent::text("Invalid client status"))
                    .await;
            }
        };
    }

    pub async fn handle_interact(&self, interact: SInteract) {
        let sneaking = interact.sneaking;
        let entity = &self.living_entity.entity;
        if entity.sneaking.load(std::sync::atomic::Ordering::Relaxed) != sneaking {
            entity.set_sneaking(sneaking).await;
        }
        let Some(action) = ActionType::from_i32(interact.typ.0) else {
            self.kick(TextComponent::text("Invalid action type")).await;
            return;
        };

        match action {
            ActionType::Attack => {
                let entity_id = interact.entity_id;
                let config = &ADVANCED_CONFIG.pvp;
                // TODO: do validation and stuff
                if !config.enabled {
                    return;
                }

                let world = &entity.world;
                let victim = world.get_player_by_entityid(entity_id.0).await;
                let Some(victim) = victim else {
                    self.kick(TextComponent::text("Interacted with invalid entity id"))
                        .await;
                    return;
                };

                self.attack(&victim).await;
            }
            ActionType::Interact | ActionType::InteractAt => {
                log::debug!("todo");
            }
        }
    }

    pub async fn handle_player_action(&self, player_action: SPlayerAction) {
        match Status::from_i32(player_action.status.0) {
            Some(status) => match status {
                Status::StartedDigging => {
                    if !self.can_interact_with_block_at(&player_action.location, 1.0) {
                        log::warn!(
                            "Player {0} tried to interact with block out of reach at {1}",
                            self.gameprofile.name,
                            player_action.location
                        );
                        return;
                    }
                    // TODO: do validation
                    // TODO: Config
                    if self.gamemode.load() == GameMode::Creative {
                        let location = player_action.location;
                        // Block break & block break sound
                        // TODO: currently this is always dirt replace it
                        let entity = &self.living_entity.entity;
                        let world = &entity.world;
                        world.break_block(location).await;
                    }
                }
                Status::CancelledDigging => {
                    if !self.can_interact_with_block_at(&player_action.location, 1.0) {
                        log::warn!(
                            "Player {0} tried to interact with block out of reach at {1}",
                            self.gameprofile.name,
                            player_action.location
                        );
                        return;
                    }
                    self.current_block_destroy_stage
                        .store(0, std::sync::atomic::Ordering::Relaxed);
                }
                Status::FinishedDigging => {
                    // TODO: do validation
                    let location = player_action.location;
                    if !self.can_interact_with_block_at(&location, 1.0) {
                        log::warn!(
                            "Player {0} tried to interact with block out of reach at {1}",
                            self.gameprofile.name,
                            player_action.location
                        );
                        return;
                    }
                    // Block break & block break sound
                    // TODO: currently this is always dirt replace it
                    let entity = &self.living_entity.entity;
                    let world = &entity.world;
                    world.break_block(location).await;
                    // TODO: Send this every tick
                    self.client
                        .send_packet(&CAcknowledgeBlockChange::new(player_action.sequence))
                        .await;
                }
                Status::DropItemStack
                | Status::DropItem
                | Status::ShootArrowOrFinishEating
                | Status::SwapItem => {
                    log::debug!("todo");
                }
            },
            None => self.kick(TextComponent::text("Invalid status")).await,
        }
    }

    pub async fn handle_keep_alive(&self, keep_alive: SKeepAlive) {
        if self
            .wait_for_keep_alive
            .load(std::sync::atomic::Ordering::Relaxed)
            && keep_alive.keep_alive_id
                == self
                    .keep_alive_id
                    .load(std::sync::atomic::Ordering::Relaxed)
        {
            self.wait_for_keep_alive
                .store(false, std::sync::atomic::Ordering::Relaxed);
        } else {
            self.kick(TextComponent::text("Timeout")).await;
        }
    }

    pub async fn handle_player_abilities(&self, player_abilities: SPlayerAbilities) {
        let mut abilities = self.abilities.lock().await;

        // Set the flying ability
        let flying = player_abilities.flags & 0x02 != 0 && abilities.allow_flying;
        if flying {
            self.living_entity.fall_distance.store(0.0);
        }
        abilities.flying = flying;
    }

    pub async fn handle_play_ping_request(&self, request: SPlayPingRequest) {
        self.client
            .send_packet(&CPingResponse::new(request.payload))
            .await;
    }

    pub async fn handle_use_item_on(&self, use_item_on: SUseItemOn) {
        let location = use_item_on.location;

        if !self.can_interact_with_block_at(&location, 1.0) {
            // TODO: maybe log?
            return;
        }

        if let Some(face) = BlockFace::from_i32(use_item_on.face.0) {
            if let Some(item) = self.inventory.lock().await.held_item() {
                let block = get_block_by_item(item.item_id);
                // check if item is a block, Because Not every item can be placed :D
                if let Some(block) = block {
                    let entity = &self.living_entity.entity;
                    let world = &entity.world;

                    world
                        .set_block(
                            WorldPosition(location.0 + face.to_offset()),
                            block.default_state_id,
                        )
                        .await;
                }
                self.client
                    .send_packet(&CAcknowledgeBlockChange::new(use_item_on.sequence))
                    .await;
            }
        } else {
            self.kick(TextComponent::text("Invalid block face")).await;
        }
    }

    pub fn handle_use_item(&self, _use_item: &SUseItem) {
        // TODO: handle packet correctly
        log::error!("An item was used(SUseItem), but the packet is not implemented yet");
    }

    pub async fn handle_set_held_item(&self, held: SSetHeldItem) {
        let slot = held.slot;
        if !(0..=8).contains(&slot) {
            self.kick(TextComponent::text("Invalid held slot")).await;
            return;
        }
        self.inventory.lock().await.set_selected(slot as usize);
    }

    pub async fn handle_set_creative_slot(
        &self,
        packet: SSetCreativeSlot,
    ) -> Result<(), InventoryError> {
        if self.gamemode.load() != GameMode::Creative {
            return Err(InventoryError::PermissionError);
        }
        let valid_slot = packet.slot >= 1 && packet.slot <= 45;
        if valid_slot {
            self.inventory.lock().await.set_slot(
                packet.slot as u16,
                packet.clicked_item.to_item(),
                true,
            )?;
        };
        // TODO: The Item was droped per drag and drop,
        Ok(())
    }

    // TODO:
    // This function will in the future be used to keep track of if the client is in a valid state.
    // But this is not possible yet
    pub async fn handle_close_container(&self, server: &Server, packet: SCloseContainer) {
        let Some(_window_type) = WindowType::from_i32(packet.window_id.0) else {
            self.kick(TextComponent::text("Invalid window ID")).await;
            return;
        };
        // window_id 0 represents both 9x1 Generic AND inventory here
        self.inventory
            .lock()
            .await
            .state_id
            .store(0, std::sync::atomic::Ordering::Relaxed);
        let open_container = self.open_container.load();
        if let Some(id) = open_container {
            let mut open_containers = server.open_containers.write().await;
            if let Some(container) = open_containers.get_mut(&id) {
                container.remove_player(self.entity_id());
            }
            self.open_container.store(None);
        }
    }
}
