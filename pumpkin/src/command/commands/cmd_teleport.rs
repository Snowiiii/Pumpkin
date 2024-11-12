use async_trait::async_trait;
use pumpkin_core::math::vector3::Vector3;
use pumpkin_core::text::TextComponent;

use crate::command::args::arg_entities::EntitiesArgumentConsumer;
use crate::command::args::arg_entity::EntityArgumentConsumer;
use crate::command::args::arg_position_3d::Position3DArgumentConsumer;
use crate::command::args::arg_rotation::RotationArgumentConsumer;
use crate::command::args::ConsumedArgs;
use crate::command::args::FindArg;
use crate::command::tree::CommandTree;
use crate::command::tree_builder::{argument, literal, require};
use crate::command::CommandError;
use crate::command::{CommandExecutor, CommandSender};
use crate::entity::player::PermissionLvl;

const NAMES: [&str; 2] = ["teleport", "tp"];
const DESCRIPTION: &str = "Teleports entities, including players."; // todo

/// position
const ARG_LOCATION: &str = "location";

/// single entity
const ARG_DESTINATION: &str = "destination";

/// multiple entities
const ARG_TARGETS: &str = "targets";

/// rotation: yaw/pitch
const ARG_ROTATION: &str = "rotation";

/// single entity
const ARG_FACING_ENTITY: &str = "facingEntity";

/// position
const ARG_FACING_LOCATION: &str = "facingLocation";

fn yaw_pitch_facing_position(
    looking_from: &Vector3<f64>,
    looking_towards: &Vector3<f64>,
) -> (f32, f32) {
    let direction_vector = (looking_towards.sub(looking_from)).normalize();

    let yaw_radians = -direction_vector.x.atan2(direction_vector.z);
    let pitch_radians = (-direction_vector.y).asin();

    let yaw_degrees = yaw_radians * 180.0 / std::f64::consts::PI;
    let pitch_degrees = pitch_radians * 180.0 / std::f64::consts::PI;

    (yaw_degrees as f32, pitch_degrees as f32)
}

struct TpEntitiesToEntityExecutor;

#[async_trait]
impl CommandExecutor for TpEntitiesToEntityExecutor {
    async fn execute<'a>(
        &self,
        _sender: &mut CommandSender<'a>,
        _server: &crate::server::Server,
        args: &ConsumedArgs<'a>,
    ) -> Result<(), CommandError> {
        let targets = EntitiesArgumentConsumer::find_arg(args, ARG_TARGETS)?;

        let destination = EntityArgumentConsumer::find_arg(args, ARG_DESTINATION)?;
        let pos = destination.living_entity.entity.pos.load();

        for target in targets {
            let yaw = target.living_entity.entity.yaw.load();
            let pitch = target.living_entity.entity.pitch.load();
            target.teleport(pos, yaw, pitch).await;
        }

        Ok(())
    }
}

struct TpEntitiesToPosFacingPosExecutor;

#[async_trait]
impl CommandExecutor for TpEntitiesToPosFacingPosExecutor {
    async fn execute<'a>(
        &self,
        _sender: &mut CommandSender<'a>,
        _server: &crate::server::Server,
        args: &ConsumedArgs<'a>,
    ) -> Result<(), CommandError> {
        let targets = EntitiesArgumentConsumer::find_arg(args, ARG_TARGETS)?;

        let pos = Position3DArgumentConsumer::find_arg(args, ARG_LOCATION)?;

        let facing_pos = Position3DArgumentConsumer::find_arg(args, ARG_FACING_LOCATION)?;
        let (yaw, pitch) = yaw_pitch_facing_position(&pos, &facing_pos);

        for target in targets {
            target.teleport(pos, yaw, pitch).await;
        }

        Ok(())
    }
}

struct TpEntitiesToPosFacingEntityExecutor;

#[async_trait]
impl CommandExecutor for TpEntitiesToPosFacingEntityExecutor {
    async fn execute<'a>(
        &self,
        _sender: &mut CommandSender<'a>,
        _server: &crate::server::Server,
        args: &ConsumedArgs<'a>,
    ) -> Result<(), CommandError> {
        let targets = EntitiesArgumentConsumer::find_arg(args, ARG_TARGETS)?;

        let pos = Position3DArgumentConsumer::find_arg(args, ARG_LOCATION)?;

        let facing_entity = &EntityArgumentConsumer::find_arg(args, ARG_FACING_ENTITY)?
            .living_entity
            .entity;
        let (yaw, pitch) = yaw_pitch_facing_position(&pos, &facing_entity.pos.load());

        for target in targets {
            target.teleport(pos, yaw, pitch).await;
        }

        Ok(())
    }
}

struct TpEntitiesToPosWithRotationExecutor;

#[async_trait]
impl CommandExecutor for TpEntitiesToPosWithRotationExecutor {
    async fn execute<'a>(
        &self,
        _sender: &mut CommandSender<'a>,
        _server: &crate::server::Server,
        args: &ConsumedArgs<'a>,
    ) -> Result<(), CommandError> {
        let targets = EntitiesArgumentConsumer::find_arg(args, ARG_TARGETS)?;

        let pos = Position3DArgumentConsumer::find_arg(args, ARG_LOCATION)?;

        let (yaw, pitch) = RotationArgumentConsumer::find_arg(args, ARG_ROTATION)?;

        for target in targets {
            target.teleport(pos, yaw, pitch).await;
        }

        Ok(())
    }
}

struct TpEntitiesToPosExecutor;

#[async_trait]
impl CommandExecutor for TpEntitiesToPosExecutor {
    async fn execute<'a>(
        &self,
        _sender: &mut CommandSender<'a>,
        _server: &crate::server::Server,
        args: &ConsumedArgs<'a>,
    ) -> Result<(), CommandError> {
        let targets = EntitiesArgumentConsumer::find_arg(args, ARG_TARGETS)?;

        let pos = Position3DArgumentConsumer::find_arg(args, ARG_LOCATION)?;

        for target in targets {
            let yaw = target.living_entity.entity.yaw.load();
            let pitch = target.living_entity.entity.pitch.load();
            target.teleport(pos, yaw, pitch).await;
        }

        Ok(())
    }
}

struct TpSelfToEntityExecutor;

#[async_trait]
impl CommandExecutor for TpSelfToEntityExecutor {
    async fn execute<'a>(
        &self,
        sender: &mut CommandSender<'a>,
        _server: &crate::server::Server,
        args: &ConsumedArgs<'a>,
    ) -> Result<(), CommandError> {
        let destination = EntityArgumentConsumer::find_arg(args, ARG_DESTINATION)?;
        let pos = destination.living_entity.entity.pos.load();

        match sender {
            CommandSender::Player(player) => {
                let yaw = player.living_entity.entity.yaw.load();
                let pitch = player.living_entity.entity.pitch.load();
                player.teleport(pos, yaw, pitch).await;
            }
            _ => {
                sender
                    .send_message(TextComponent::text(
                        "Only players may execute this command.",
                    ))
                    .await;
            }
        };

        Ok(())
    }
}

struct TpSelfToPosExecutor;

#[async_trait]
impl CommandExecutor for TpSelfToPosExecutor {
    async fn execute<'a>(
        &self,
        sender: &mut CommandSender<'a>,
        _server: &crate::server::Server,
        args: &ConsumedArgs<'a>,
    ) -> Result<(), CommandError> {
        match sender {
            CommandSender::Player(player) => {
                let pos = Position3DArgumentConsumer::find_arg(args, ARG_LOCATION)?;
                let yaw = player.living_entity.entity.yaw.load();
                let pitch = player.living_entity.entity.pitch.load();
                player.teleport(pos, yaw, pitch).await;
            }
            _ => {
                sender
                    .send_message(TextComponent::text(
                        "Only players may execute this command.",
                    ))
                    .await;
            }
        };

        Ok(())
    }
}

pub fn init_command_tree<'a>() -> CommandTree<'a> {
    CommandTree::new(NAMES, DESCRIPTION).with_child(
        require(&|sender| sender.has_permission_lvl(PermissionLvl::Two))
            .with_child(
                argument(ARG_LOCATION, &Position3DArgumentConsumer).execute(&TpSelfToPosExecutor),
            )
            .with_child(
                argument(ARG_DESTINATION, &EntityArgumentConsumer).execute(&TpSelfToEntityExecutor),
            )
            .with_child(
                argument(ARG_TARGETS, &EntitiesArgumentConsumer)
                    .with_child(
                        argument(ARG_LOCATION, &Position3DArgumentConsumer)
                            .execute(&TpEntitiesToPosExecutor)
                            .with_child(
                                argument(ARG_ROTATION, &RotationArgumentConsumer)
                                    .execute(&TpEntitiesToPosWithRotationExecutor),
                            )
                            .with_child(
                                literal("facing")
                                    .with_child(
                                        literal("entity").with_child(
                                            argument(ARG_FACING_ENTITY, &EntityArgumentConsumer)
                                                .execute(&TpEntitiesToPosFacingEntityExecutor),
                                        ),
                                    )
                                    .with_child(
                                        argument(ARG_FACING_LOCATION, &Position3DArgumentConsumer)
                                            .execute(&TpEntitiesToPosFacingPosExecutor),
                                    ),
                            ),
                    )
                    .with_child(
                        argument(ARG_DESTINATION, &EntityArgumentConsumer)
                            .execute(&TpEntitiesToEntityExecutor),
                    ),
            ),
    )
}
