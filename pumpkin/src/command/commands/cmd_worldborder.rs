use async_trait::async_trait;
use pumpkin_core::text::{
    color::{Color, NamedColor},
    TextComponent,
};

use crate::{
    command::{
        arg_position::{parse_arg_position, PositionArgumentConsumer},
        arg_simple::SimpleArgConsumer,
        tree::{CommandTree, ConsumedArgs},
        tree_builder::{argument, literal},
        CommandExecutor, CommandSender, InvalidTreeError,
    },
    server::Server,
};

const NAMES: [&str; 1] = ["worldborder"];

const DESCRIPTION: &str = "Worldborder command.";

struct WorldborderGetExecutor;

#[async_trait]
impl CommandExecutor for WorldborderGetExecutor {
    async fn execute<'a>(
        &self,
        sender: &mut CommandSender<'a>,
        server: &Server,
        _args: &ConsumedArgs<'a>,
    ) -> Result<(), InvalidTreeError> {
        let world = server
            .worlds
            .first()
            .expect("There should always be atleast one world");
        let border = world.worldborder.lock().await;

        let diameter = border.new_diameter.round() as i32;
        sender
            .send_message(TextComponent::text(&format!(
                "The world border is currently {diameter} block(s) wide"
            )))
            .await;
        Ok(())
    }
}

struct WorldborderSetExecutor;

#[async_trait]
impl CommandExecutor for WorldborderSetExecutor {
    async fn execute<'a>(
        &self,
        sender: &mut CommandSender<'a>,
        server: &Server,
        args: &ConsumedArgs<'a>,
    ) -> Result<(), InvalidTreeError> {
        let world = server
            .worlds
            .first()
            .expect("There should always be atleast one world");
        let mut border = world.worldborder.lock().await;

        let distance = args
            .get("distance")
            .ok_or(InvalidTreeError::InvalidConsumptionError(None))?
            .parse::<f64>()
            .map_err(|err| InvalidTreeError::InvalidConsumptionError(Some(err.to_string())))?;

        if (distance - border.new_diameter).abs() < f64::EPSILON {
            sender
                .send_message(
                    TextComponent::text("Nothing changed. The world border is already that size")
                        .color(Color::Named(NamedColor::Red)),
                )
                .await;
            return Ok(());
        }

        sender
            .send_message(TextComponent::text(&format!(
                "Set the world border to {distance:.1} block(s) wide"
            )))
            .await;
        border.set_diameter(world, distance, None).await;
        Ok(())
    }
}

struct WorldborderSetTimeExecutor;

#[async_trait]
impl CommandExecutor for WorldborderSetTimeExecutor {
    async fn execute<'a>(
        &self,
        sender: &mut CommandSender<'a>,
        server: &Server,
        args: &ConsumedArgs<'a>,
    ) -> Result<(), InvalidTreeError> {
        let world = server
            .worlds
            .first()
            .expect("There should always be atleast one world");
        let mut border = world.worldborder.lock().await;

        let distance = args
            .get("distance")
            .ok_or(InvalidTreeError::InvalidConsumptionError(None))?
            .parse::<f64>()
            .map_err(|err| InvalidTreeError::InvalidConsumptionError(Some(err.to_string())))?;
        let time = args
            .get("time")
            .ok_or(InvalidTreeError::InvalidConsumptionError(None))?
            .parse::<i32>()
            .map_err(|err| InvalidTreeError::InvalidConsumptionError(Some(err.to_string())))?;

        match distance.total_cmp(&border.new_diameter) {
            std::cmp::Ordering::Equal => {
                sender
                    .send_message(
                        TextComponent::text(
                            "Nothing changed. The world border is already that size",
                        )
                        .color(Color::Named(NamedColor::Red)),
                    )
                    .await;
                return Ok(());
            }
            std::cmp::Ordering::Less => {
                sender.send_message(TextComponent::text(&format!("Shrinking the world border to {distance:.2} blocks wide over {time} second(s)"))).await;
            }
            std::cmp::Ordering::Greater => {
                sender
                    .send_message(TextComponent::text(&format!(
                        "Growing the world border to {distance:.2} blocks wide over {time} seconds"
                    )))
                    .await;
            }
        }

        border
            .set_diameter(world, distance, Some(i64::from(time) * 1000))
            .await;
        Ok(())
    }
}

struct WorldborderAddExecutor;

#[async_trait]
impl CommandExecutor for WorldborderAddExecutor {
    async fn execute<'a>(
        &self,
        sender: &mut CommandSender<'a>,
        server: &Server,
        args: &ConsumedArgs<'a>,
    ) -> Result<(), InvalidTreeError> {
        let world = server
            .worlds
            .first()
            .expect("There should always be atleast one world");
        let mut border = world.worldborder.lock().await;

        let distance = args
            .get("distance")
            .ok_or(InvalidTreeError::InvalidConsumptionError(None))?
            .parse::<f64>()
            .map_err(|err| InvalidTreeError::InvalidConsumptionError(Some(err.to_string())))?;

        if distance == 0.0 {
            sender
                .send_message(
                    TextComponent::text("Nothing changed. The world border is already that size")
                        .color(Color::Named(NamedColor::Red)),
                )
                .await;
            return Ok(());
        }

        let distance = border.new_diameter + distance;

        sender
            .send_message(TextComponent::text(&format!(
                "Set the world border to {distance:.1} block(s) wide"
            )))
            .await;
        border.set_diameter(world, distance, None).await;
        Ok(())
    }
}

struct WorldborderAddTimeExecutor;

#[async_trait]
impl CommandExecutor for WorldborderAddTimeExecutor {
    async fn execute<'a>(
        &self,
        sender: &mut CommandSender<'a>,
        server: &Server,
        args: &ConsumedArgs<'a>,
    ) -> Result<(), InvalidTreeError> {
        let world = server
            .worlds
            .first()
            .expect("There should always be atleast one world");
        let mut border = world.worldborder.lock().await;

        let distance = args
            .get("distance")
            .ok_or(InvalidTreeError::InvalidConsumptionError(None))?
            .parse::<f64>()
            .map_err(|err| InvalidTreeError::InvalidConsumptionError(Some(err.to_string())))?;
        let time = args
            .get("time")
            .ok_or(InvalidTreeError::InvalidConsumptionError(None))?
            .parse::<i32>()
            .map_err(|err| InvalidTreeError::InvalidConsumptionError(Some(err.to_string())))?;

        let distance = distance + border.new_diameter;

        match distance.total_cmp(&border.new_diameter) {
            std::cmp::Ordering::Equal => {
                sender
                    .send_message(
                        TextComponent::text(
                            "Nothing changed. The world border is already that size",
                        )
                        .color(Color::Named(NamedColor::Red)),
                    )
                    .await;
                return Ok(());
            }
            std::cmp::Ordering::Less => {
                sender.send_message(TextComponent::text(&format!("Shrinking the world border to {distance:.2} blocks wide over {time} second(s)"))).await;
            }
            std::cmp::Ordering::Greater => {
                sender
                    .send_message(TextComponent::text(&format!(
                        "Growing the world border to {distance:.2} blocks wide over {time} seconds"
                    )))
                    .await;
            }
        }

        border
            .set_diameter(world, distance, Some(i64::from(time) * 1000))
            .await;
        Ok(())
    }
}

struct WorldborderCenterExecutor;

#[async_trait]
impl CommandExecutor for WorldborderCenterExecutor {
    async fn execute<'a>(
        &self,
        sender: &mut CommandSender<'a>,
        server: &Server,
        args: &ConsumedArgs<'a>,
    ) -> Result<(), InvalidTreeError> {
        let world = server
            .worlds
            .first()
            .expect("There should always be atleast one world");
        let mut border = world.worldborder.lock().await;

        let x = parse_arg_position("x", args)?;
        let z = parse_arg_position("z", args)?;

        sender
            .send_message(TextComponent::text(&format!(
                "Set the center of world border to {x:.2}, {z:.2}"
            )))
            .await;
        border.set_center(world, x, z).await;
        Ok(())
    }
}

struct WorldborderDamageAmountExecutor;

#[async_trait]
impl CommandExecutor for WorldborderDamageAmountExecutor {
    async fn execute<'a>(
        &self,
        sender: &mut CommandSender<'a>,
        server: &Server,
        args: &ConsumedArgs<'a>,
    ) -> Result<(), InvalidTreeError> {
        let world = server
            .worlds
            .first()
            .expect("There should always be atleast one world");
        let mut border = world.worldborder.lock().await;

        let damage_per_block = args
            .get("damage_per_block")
            .ok_or(InvalidTreeError::InvalidConsumptionError(None))?
            .parse::<f32>()
            .map_err(|err| InvalidTreeError::InvalidConsumptionError(Some(err.to_string())))?;

        if (damage_per_block - border.damage_per_block).abs() < f32::EPSILON {
            sender
                .send_message(
                    TextComponent::text(
                        "Nothing changed. The world border damage is already that amount",
                    )
                    .color(Color::Named(NamedColor::Red)),
                )
                .await;
            return Ok(());
        }

        sender
            .send_message(TextComponent::text(&format!(
                "Set the world border damage to {damage_per_block:.2} per block each second"
            )))
            .await;
        border.damage_per_block = damage_per_block;
        Ok(())
    }
}

struct WorldborderDamageBufferExecutor;

#[async_trait]
impl CommandExecutor for WorldborderDamageBufferExecutor {
    async fn execute<'a>(
        &self,
        sender: &mut CommandSender<'a>,
        server: &Server,
        args: &ConsumedArgs<'a>,
    ) -> Result<(), InvalidTreeError> {
        let world = server
            .worlds
            .first()
            .expect("There should always be atleast one world");
        let mut border = world.worldborder.lock().await;

        let buffer = args
            .get("distance")
            .ok_or(InvalidTreeError::InvalidConsumptionError(None))?
            .parse::<f32>()
            .map_err(|err| InvalidTreeError::InvalidConsumptionError(Some(err.to_string())))?;

        if (buffer - border.buffer).abs() < f32::EPSILON {
            sender
                .send_message(
                    TextComponent::text(
                        "Nothing changed. The world border damage buffer is already that distance",
                    )
                    .color(Color::Named(NamedColor::Red)),
                )
                .await;
            return Ok(());
        }

        sender
            .send_message(TextComponent::text(&format!(
                "Set the world border damage buffer to {buffer:.2} block(s)"
            )))
            .await;
        border.buffer = buffer;
        Ok(())
    }
}

struct WorldborderWarningDistanceExecutor;

#[async_trait]
impl CommandExecutor for WorldborderWarningDistanceExecutor {
    async fn execute<'a>(
        &self,
        sender: &mut CommandSender<'a>,
        server: &Server,
        args: &ConsumedArgs<'a>,
    ) -> Result<(), InvalidTreeError> {
        let world = server
            .worlds
            .first()
            .expect("There should always be atleast one world");
        let mut border = world.worldborder.lock().await;

        let distance = args
            .get("distance")
            .ok_or(InvalidTreeError::InvalidConsumptionError(None))?
            .parse::<i32>()
            .map_err(|err| InvalidTreeError::InvalidConsumptionError(Some(err.to_string())))?;

        if distance == border.warning_blocks {
            sender
                .send_message(
                    TextComponent::text(
                        "Nothing changed. The world border warning is already that distance",
                    )
                    .color(Color::Named(NamedColor::Red)),
                )
                .await;
            return Ok(());
        }

        sender
            .send_message(TextComponent::text(&format!(
                "Set the world border warning distance to {distance} block(s)"
            )))
            .await;
        border.set_warning_distance(world, distance).await;
        Ok(())
    }
}

struct WorldborderWarningTimeExecutor;

#[async_trait]
impl CommandExecutor for WorldborderWarningTimeExecutor {
    async fn execute<'a>(
        &self,
        sender: &mut CommandSender<'a>,
        server: &Server,
        args: &ConsumedArgs<'a>,
    ) -> Result<(), InvalidTreeError> {
        let world = server
            .worlds
            .first()
            .expect("There should always be atleast one world");
        let mut border = world.worldborder.lock().await;

        let time = args
            .get("time")
            .ok_or(InvalidTreeError::InvalidConsumptionError(None))?
            .parse::<i32>()
            .map_err(|err| InvalidTreeError::InvalidConsumptionError(Some(err.to_string())))?;

        if time == border.warning_time {
            sender
                .send_message(
                    TextComponent::text(
                        "Nothing changed. The world border warning is already that amount of time",
                    )
                    .color(Color::Named(NamedColor::Red)),
                )
                .await;
            return Ok(());
        }

        sender
            .send_message(TextComponent::text(&format!(
                "Set the world border warning time to {time} second(s)"
            )))
            .await;
        border.set_warning_delay(world, time).await;
        Ok(())
    }
}

pub fn init_command_tree<'a>() -> CommandTree<'a> {
    CommandTree::new(NAMES, DESCRIPTION)
        .with_child(
            literal("add").with_child(
                argument("distance", &SimpleArgConsumer)
                    .execute(&WorldborderAddExecutor)
                    .with_child(
                        argument("time", &SimpleArgConsumer).execute(&WorldborderAddTimeExecutor),
                    ),
            ),
        )
        .with_child(literal("center").with_child(
            argument("x", &PositionArgumentConsumer).with_child(
                argument("z", &PositionArgumentConsumer).execute(&WorldborderCenterExecutor),
            ),
        ))
        .with_child(
            literal("damage")
                .with_child(
                    literal("amount").with_child(
                        argument("damage_per_block", &SimpleArgConsumer)
                            .execute(&WorldborderDamageAmountExecutor),
                    ),
                )
                .with_child(
                    literal("buffer").with_child(
                        argument("distance", &SimpleArgConsumer)
                            .execute(&WorldborderDamageBufferExecutor),
                    ),
                ),
        )
        .with_child(literal("get").execute(&WorldborderGetExecutor))
        .with_child(
            literal("set").with_child(
                argument("distance", &SimpleArgConsumer)
                    .execute(&WorldborderSetExecutor)
                    .with_child(
                        argument("time", &SimpleArgConsumer).execute(&WorldborderSetTimeExecutor),
                    ),
            ),
        )
        .with_child(
            literal("warning")
                .with_child(
                    literal("distance").with_child(
                        argument("distance", &SimpleArgConsumer)
                            .execute(&WorldborderWarningDistanceExecutor),
                    ),
                )
                .with_child(literal("time").with_child(
                    argument("time", &SimpleArgConsumer).execute(&WorldborderWarningTimeExecutor),
                )),
        )
}
