use async_trait::async_trait;
use pumpkin_core::text::TextComponent;

use crate::command::arg_entities::{parse_arg_entities, EntitiesArgumentConsumer};
use crate::command::arg_entity::{parse_arg_entity, EntityArgumentConsumer};
use crate::command::arg_position_3d::{parse_arg_position_3d, Position3DArgumentConsumer};
use crate::command::arg_rotation::{parse_arg_rotation, RotationArgumentConsumer};
use crate::command::tree::CommandTree;
use crate::command::tree_builder::{argument, literal, require};
use crate::command::InvalidTreeError;
use crate::command::{tree::ConsumedArgs, CommandExecutor, CommandSender};

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

struct TpEntitiesToEntityExecutor;

#[async_trait]
impl CommandExecutor for TpEntitiesToEntityExecutor {
    async fn execute<'a>(
        &self,
        sender: &mut CommandSender<'a>,
        server: &crate::server::Server,
        args: &ConsumedArgs<'a>,
    ) -> Result<(), InvalidTreeError> {
        let targets = parse_arg_entities(sender, server, ARG_TARGETS, args).await?;

        let destination = parse_arg_entity(sender, server, ARG_DESTINATION, args).await?;
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
        sender: &mut CommandSender<'a>,
        server: &crate::server::Server,
        args: &ConsumedArgs<'a>,
    ) -> Result<(), InvalidTreeError> {
        let targets = parse_arg_entities(sender, server, ARG_TARGETS, args).await?;

        let pos = parse_arg_position_3d(ARG_LOCATION, args)?;

        // TODO: calculate yaw/pitch based on position
        // let facing_position = parse_arg_position_3d(ARG_FACING_LOCATION, args)?;

        for target in targets {
            let yaw = target.living_entity.entity.yaw.load();
            let pitch = target.living_entity.entity.pitch.load();
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
        sender: &mut CommandSender<'a>,
        server: &crate::server::Server,
        args: &ConsumedArgs<'a>,
    ) -> Result<(), InvalidTreeError> {
        let targets = parse_arg_entities(sender, server, ARG_TARGETS, args).await?;

        let pos = parse_arg_position_3d(ARG_LOCATION, args)?;

        // TODO: calculate yaw/pitch based on entity
        // let facing_entity = parse_arg_entity(sender, server, ARG_FACING_ENTITY, args).await?;

        for target in targets {
            let yaw = target.living_entity.entity.yaw.load();
            let pitch = target.living_entity.entity.pitch.load();
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
        sender: &mut CommandSender<'a>,
        server: &crate::server::Server,
        args: &ConsumedArgs<'a>,
    ) -> Result<(), InvalidTreeError> {
        let targets = parse_arg_entities(sender, server, ARG_TARGETS, args).await?;

        let pos = parse_arg_position_3d(ARG_LOCATION, args)?;

        let (yaw, pitch) = parse_arg_rotation(ARG_ROTATION, args)?;

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
        sender: &mut CommandSender<'a>,
        server: &crate::server::Server,
        args: &ConsumedArgs<'a>,
    ) -> Result<(), InvalidTreeError> {
        let targets = parse_arg_entities(sender, server, ARG_TARGETS, args).await?;

        let pos = parse_arg_position_3d(ARG_LOCATION, args)?;

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
        server: &crate::server::Server,
        args: &ConsumedArgs<'a>,
    ) -> Result<(), InvalidTreeError> {
        let destination = parse_arg_entity(sender, server, ARG_DESTINATION, args).await?;
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
    ) -> Result<(), InvalidTreeError> {
        match sender {
            CommandSender::Player(player) => {
                let pos = parse_arg_position_3d(ARG_LOCATION, args)?;
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
        require(&|sender| sender.permission_lvl() >= 2)
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
            )
            .with_child(
                argument(ARG_LOCATION, &Position3DArgumentConsumer).execute(&TpSelfToPosExecutor),
            )
            .with_child(
                argument(ARG_DESTINATION, &EntityArgumentConsumer).execute(&TpSelfToEntityExecutor),
            ),
    )
}
