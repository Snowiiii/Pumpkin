use std::sync::Arc;

use super::PlayerConfig;
use crate::block::block_manager::BlockActionResult;
use crate::{
    command::CommandSender,
    entity::player::{ChatMode, Hand, Player},
    error::PumpkinError,
    server::Server,
    world::player_chunker,
};
use num_traits::FromPrimitive;
use pumpkin_config::ADVANCED_CONFIG;
use pumpkin_core::math::{boundingbox::BoundingBox, position::WorldPosition};
use pumpkin_core::{
    math::{vector3::Vector3, wrap_degrees},
    text::TextComponent,
    GameMode,
};
use pumpkin_inventory::{InventoryError, WindowType};
use pumpkin_protocol::server::play::SCookieResponse as SPCookieResponse;
use pumpkin_protocol::{
    client::play::CCommandSuggestions,
    server::play::{SCloseContainer, SCommandSuggestion, SKeepAlive, SSetPlayerGround, SUseItem},
    VarInt,
};
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
use pumpkin_world::item::item_registry::get_item_by_id;
use thiserror::Error;

fn modulus(a: f32, b: f32) -> f32 {
    ((a % b) + b) % b
}

#[derive(Debug, Error)]
pub enum BlockPlacingError {
    BlockOutOfReach,
    InvalidBlockFace,
    BlockOutOfWorld,
    InventoryInvalid,
}

impl std::fmt::Display for BlockPlacingError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

impl PumpkinError for BlockPlacingError {
    fn is_kick(&self) -> bool {
        match self {
            Self::BlockOutOfReach | Self::BlockOutOfWorld => false,
            Self::InvalidBlockFace | Self::InventoryInvalid => true,
        }
    }

    fn severity(&self) -> log::Level {
        match self {
            Self::BlockOutOfReach | Self::BlockOutOfWorld | Self::InvalidBlockFace => {
                log::Level::Warn
            }
            Self::InventoryInvalid => log::Level::Error,
        }
    }

    fn client_kick_reason(&self) -> Option<String> {
        match self {
            Self::BlockOutOfReach | Self::BlockOutOfWorld => None,
            Self::InvalidBlockFace => Some("Invalid block face".into()),
            Self::InventoryInvalid => Some("Held item invalid".into()),
        }
    }
}

/// Handles all Play Packets send by a real Player
/// NEVER TRUST THE CLIENT. HANDLE EVERY ERROR, UNWRAP/EXPECT ARE FORBIDDEN
impl Player {
    pub async fn handle_confirm_teleport(&self, confirm_teleport: SConfirmTeleport) {
        let mut awaiting_teleport = self.awaiting_teleport.lock().await;
        if let Some((id, position)) = awaiting_teleport.as_ref() {
            if id == &confirm_teleport.teleport_id {
                // we should set the pos now to that we requested in the teleport packet, Is may fixed issues when the client sended position packets while being teleported
                self.living_entity.set_pos(*position);

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

    pub async fn handle_position(self: &Arc<Self>, packet: SPlayerPosition) {
        // y = feet Y
        let position = packet.position;
        if position.x.is_nan() || position.y.is_nan() || position.z.is_nan() {
            self.kick(TextComponent::text("Invalid movement")).await;
            return;
        }
        let position = Vector3::new(
            Self::clamp_horizontal(position.x),
            Self::clamp_vertical(position.y),
            Self::clamp_horizontal(position.z),
        );
        let entity = &self.living_entity.entity;
        self.living_entity.set_pos(position);

        let pos = entity.pos.load();
        let last_pos = self.living_entity.last_pos.load();

        entity
            .on_ground
            .store(packet.ground, std::sync::atomic::Ordering::Relaxed);

        let entity_id = entity.entity_id;
        let Vector3 { x, y, z } = pos;
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
                    Vector3::new(
                        x.mul_add(4096.0, -(last_pos.x * 4096.0)) as i16,
                        y.mul_add(4096.0, -(last_pos.y * 4096.0)) as i16,
                        z.mul_add(4096.0, -(last_pos.z * 4096.0)) as i16,
                    ),
                    packet.ground,
                ),
            )
            .await;
        player_chunker::update_position(self).await;
    }

    pub async fn handle_position_rotation(self: &Arc<Self>, packet: SPlayerPositionRotation) {
        // y = feet Y
        let position = packet.position;
        if position.x.is_nan() || position.y.is_nan() || position.z.is_nan() {
            self.kick(TextComponent::text("Invalid movement")).await;
            return;
        }

        if packet.yaw.is_infinite() || packet.pitch.is_infinite() {
            self.kick(TextComponent::text("Invalid rotation")).await;
            return;
        }
        let position = Vector3::new(
            Self::clamp_horizontal(position.x),
            Self::clamp_vertical(position.y),
            Self::clamp_horizontal(position.z),
        );
        let entity = &self.living_entity.entity;
        self.living_entity.set_pos(position);

        let pos = entity.pos.load();
        let last_pos = self.living_entity.last_pos.load();

        entity
            .on_ground
            .store(packet.ground, std::sync::atomic::Ordering::Relaxed);

        entity.set_rotation(
            wrap_degrees(packet.yaw) % 360.0,
            wrap_degrees(packet.pitch).clamp(-90.0, 90.0) % 360.0,
        );

        let entity_id = entity.entity_id;
        let Vector3 { x, y, z } = pos;

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
                    Vector3::new(
                        x.mul_add(4096.0, -(last_pos.x * 4096.0)) as i16,
                        y.mul_add(4096.0, -(last_pos.y * 4096.0)) as i16,
                        z.mul_add(4096.0, -(last_pos.z * 4096.0)) as i16,
                    ),
                    yaw as u8,
                    pitch as u8,
                    packet.ground,
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

    pub async fn handle_chat_command(
        self: &Arc<Self>,
        server: &Arc<Server>,
        command: SChatCommand,
    ) {
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
        let animation = match swing_arm.hand.0 {
            0 => Animation::SwingMainArm,
            1 => Animation::SwingOffhand,
            _ => {
                self.kick(TextComponent::text("Invalid hand")).await;
                return;
            }
        };
        // Invert hand if player is left handed
        let animation = match self.config.lock().await.main_hand {
            Hand::Left => match animation {
                Animation::SwingMainArm => Animation::SwingOffhand,
                Animation::SwingOffhand => Animation::SwingMainArm,
                _ => unreachable!(),
            },
            Hand::Right => animation,
        };

        let id = self.entity_id();
        let world = self.world();
        world
            .broadcast_packet_except(
                &[self.gameprofile.id],
                &CEntityAnimation::new(id.into(), animation as u8),
            )
            .await;
    }

    pub async fn handle_chat_message(&self, chat_message: SChatMessage) {
        let message = chat_message.message;
        if message.len() > 256 {
            self.kick(TextComponent::text("Oversized message")).await;
            return;
        }

        if message.chars().any(|c| c == '§' || c < ' ' || c == '\x7F') {
            self.kick(TextComponent::text("Illegal characters in chat"))
                .await;
            return;
        }

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

    pub async fn handle_client_information(
        self: &Arc<Player>,
        client_information: SClientInformationPlay,
    ) {
        if let (Some(main_hand), Some(chat_mode)) = (
            Hand::from_i32(client_information.main_hand.into()),
            ChatMode::from_i32(client_information.chat_mode.into()),
        ) {
            if client_information.view_distance <= 0 {
                self.kick(TextComponent::text(
                    "Cannot have zero or negative view distance!",
                ))
                .await;
                return;
            }

            let (update_skin, update_watched) = {
                let mut config = self.config.lock().await;
                let update_skin = config.main_hand != main_hand
                    || config.skin_parts != client_information.skin_parts;

                let old_view_distance = config.view_distance;

                let update_watched = if old_view_distance == client_information.view_distance as u8
                {
                    false
                } else {
                    log::debug!(
                        "Player {} ({}) updated render distance: {} -> {}.",
                        self.gameprofile.name,
                        self.client.id,
                        old_view_distance,
                        client_information.view_distance
                    );

                    true
                };

                *config = PlayerConfig {
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
                (update_skin, update_watched)
            };

            if update_watched {
                player_chunker::update_position(self).await;
            }

            if update_skin {
                log::debug!(
                    "Player {} ({}) updated their skin.",
                    self.gameprofile.name,
                    self.client.id,
                );
                self.update_client_information().await;
            }
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
                self.world().respawn_player(self, false).await;
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
                if victim.living_entity.health.load() <= 0.0 {
                    // you can trigger this from a non-modded / innocent client client,
                    // so we shouldn't kick the player
                    return;
                }
                self.attack(&victim).await;
            }
            ActionType::Interact | ActionType::InteractAt => {
                log::debug!("todo");
            }
        }
    }

    pub async fn handle_player_action(&self, player_action: SPlayerAction, server: &Server) {
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
                        let entity = &self.living_entity.entity;
                        let world = &entity.world;
                        let block = world.get_block(location).await;

                        world.break_block(location, Some(self)).await;

                        if let Ok(block) = block {
                            server
                                .block_manager
                                .on_broken(block, self, location, server)
                                .await;
                        }
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
                    let entity = &self.living_entity.entity;
                    let world = &entity.world;
                    let block = world.get_block(location).await;

                    world.break_block(location, Some(self)).await;

                    if let Ok(block) = block {
                        server
                            .block_manager
                            .on_broken(block, self, location, server)
                            .await;
                    }
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

    pub async fn handle_use_item_on(
        &self,
        use_item_on: SUseItemOn,
        server: &Server,
    ) -> Result<(), Box<dyn PumpkinError>> {
        let location = use_item_on.location;

        if !self.can_interact_with_block_at(&location, 1.0) {
            // TODO: maybe log?
            return Err(BlockPlacingError::BlockOutOfReach.into());
        }

        if let Some(face) = BlockFace::from_i32(use_item_on.face.0) {
            let inventory = self.inventory().lock().await;
            let entity = &self.living_entity.entity;
            let world = &entity.world;
            let item_slot = inventory.held_item();

            if let Some(item_stack) = item_slot {
                let item_stack = *item_stack;
                drop(inventory);

                if let Some(item) = get_item_by_id(item_stack.item_id) {
                    if let Ok(block) = world.get_block(location).await {
                        let result = server
                            .block_manager
                            .on_use_with_item(block, self, location, item, server)
                            .await;
                        match result {
                            BlockActionResult::Continue => {}
                            BlockActionResult::Consume => {
                                return Ok(());
                            }
                        }
                    }
                }

                // check if item is a block, Because Not every item can be placed :D
                if let Some(block) = get_block_by_item(item_stack.item_id) {
                    // TODO: Config
                    // Decrease Block count
                    if self.gamemode.load() != GameMode::Creative {
                        let mut inventory = self.inventory().lock().await;
                        let item_slot = inventory.held_item_mut();
                        // This should never be possible
                        let Some(item_stack) = item_slot else {
                            return Err(BlockPlacingError::InventoryInvalid.into());
                        };
                        item_stack.item_count -= 1;
                        if item_stack.item_count == 0 {
                            *item_slot = None;
                        }
                        drop(inventory);
                    }

                    let clicked_world_pos = WorldPosition(location.0);
                    let clicked_block_state = world.get_block_state(clicked_world_pos).await?;

                    let world_pos = if clicked_block_state.replaceable {
                        clicked_world_pos
                    } else {
                        let world_pos = WorldPosition(location.0 + face.to_offset());
                        let previous_block_state = world.get_block_state(world_pos).await?;

                        if !previous_block_state.replaceable {
                            return Ok(());
                        }

                        world_pos
                    };

                    //check max world build height
                    if world_pos.0.y > 319 {
                        self.client
                            .send_packet(&CAcknowledgeBlockChange::new(use_item_on.sequence))
                            .await;
                        return Err(BlockPlacingError::BlockOutOfWorld.into());
                    }

                    let block_bounding_box = BoundingBox::from_block(&world_pos);
                    let bounding_box = entity.bounding_box.load();
                    //TODO: Make this check for every entity in that position
                    if !bounding_box.intersects(&block_bounding_box) {
                        world
                            .set_block_state(world_pos, block.default_state_id)
                            .await;
                        server
                            .block_manager
                            .on_placed(block, self, world_pos, server)
                            .await;
                    }

                    self.client
                        .send_packet(&CAcknowledgeBlockChange::new(use_item_on.sequence))
                        .await;
                }
            } else {
                drop(inventory);
                let block = world.get_block(location).await;
                if let Ok(block) = block {
                    server
                        .block_manager
                        .on_use(block, self, location, server)
                        .await;
                }
            }

            Ok(())
        } else {
            Err(BlockPlacingError::InvalidBlockFace.into())
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
        self.inventory().lock().await.set_selected(slot as usize);
    }

    pub async fn handle_set_creative_slot(
        &self,
        packet: SSetCreativeSlot,
    ) -> Result<(), InventoryError> {
        if self.gamemode.load() != GameMode::Creative {
            return Err(InventoryError::PermissionError);
        }
        let valid_slot = packet.slot >= 0 && packet.slot <= 45;
        if valid_slot {
            self.inventory().lock().await.set_slot(
                packet.slot as usize,
                packet.clicked_item.to_item(),
                true,
            )?;
        };
        // TODO: The Item was dropped per drag and drop,
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
        let mut inventory = self.inventory().lock().await;

        inventory.state_id = 0;
        let open_container = self.open_container.load();
        if let Some(id) = open_container {
            let mut open_containers = server.open_containers.write().await;
            if let Some(container) = open_containers.get_mut(&id) {
                // If container contains both a location and a type, run the on_close block_manager handler
                if let Some(pos) = container.get_location() {
                    if let Some(block) = container.get_block() {
                        server
                            .block_manager
                            .on_close(&block, self, pos, server) //block, self, location, server)
                            .await;
                    }
                }
                // Remove the player from the container
                container.remove_player(self.entity_id());
            }
            self.open_container.store(None);
        }
    }

    pub async fn handle_command_suggestion(
        self: &Arc<Self>,
        packet: SCommandSuggestion,
        server: &Arc<Server>,
    ) {
        let mut src = CommandSender::Player(self.clone());
        let Some(cmd) = &packet.command.get(1..) else {
            return;
        };

        let Some((last_word_start, _)) = cmd.char_indices().rfind(|(_, c)| c.is_whitespace())
        else {
            return;
        };

        let suggestions = server
            .command_dispatcher
            .find_suggestions(&mut src, server, cmd)
            .await;

        let response = CCommandSuggestions::new(
            packet.id,
            (last_word_start + 2).into(),
            (cmd.len() - last_word_start - 1).into(),
            suggestions,
        );

        self.client.send_packet(&response).await;
    }

    pub fn handle_cookie_response(&self, packet: SPCookieResponse) {
        // TODO: allow plugins to access this
        log::debug!(
            "Received cookie_response[play]: key: \"{}\", has_payload: \"{}\", payload_length: \"{}\"",
            packet.key,
            packet.has_payload,
            packet.payload_length.unwrap_or(VarInt::from(0)).0
        );
    }
}
