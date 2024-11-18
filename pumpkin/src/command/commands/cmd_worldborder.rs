use async_trait::async_trait;
use pumpkin_core::{
    math::vector2::Vector2,
    text::{
        color::{Color, NamedColor},
        TextComponent,
    },
};
use std::sync::Arc;

use crate::{
    command::{
        args::{
            arg_bounded_num::BoundedNumArgumentConsumer,
            arg_position_2d::Position2DArgumentConsumer, ConsumedArgs, DefaultNameArgConsumer,
            FindArgDefaultName,
        },
        tree::CommandTree,
        tree_builder::{argument_default_name, literal},
        CommandError, CommandExecutor, CommandSender,
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
        server: &Arc<Server>,
        _args: &ConsumedArgs<'a>,
    ) -> Result<(), CommandError> {
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
        server: &Arc<Server>,
        args: &ConsumedArgs<'a>,
    ) -> Result<(), CommandError> {
        let world = server
            .worlds
            .first()
            .expect("There should always be atleast one world");
        let mut border = world.worldborder.lock().await;

        let Ok(distance) = DISTANCE_CONSUMER.find_arg_default_name(args)? else {
            sender
                .send_message(
                    TextComponent::text_string(format!(
                        "{} is out of bounds.",
                        DISTANCE_CONSUMER.default_name()
                    ))
                    .color(Color::Named(NamedColor::Red)),
                )
                .await;
            return Ok(());
        };

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
        server: &Arc<Server>,
        args: &ConsumedArgs<'a>,
    ) -> Result<(), CommandError> {
        let world = server
            .worlds
            .first()
            .expect("There should always be atleast one world");
        let mut border = world.worldborder.lock().await;

        let Ok(distance) = DISTANCE_CONSUMER.find_arg_default_name(args)? else {
            sender
                .send_message(
                    TextComponent::text_string(format!(
                        "{} is out of bounds.",
                        DISTANCE_CONSUMER.default_name()
                    ))
                    .color(Color::Named(NamedColor::Red)),
                )
                .await;
            return Ok(());
        };
        let Ok(time) = TIME_CONSUMER.find_arg_default_name(args)? else {
            sender
                .send_message(
                    TextComponent::text_string(format!(
                        "{} is out of bounds.",
                        TIME_CONSUMER.default_name()
                    ))
                    .color(Color::Named(NamedColor::Red)),
                )
                .await;
            return Ok(());
        };

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
        server: &Arc<Server>,
        args: &ConsumedArgs<'a>,
    ) -> Result<(), CommandError> {
        let world = server
            .worlds
            .first()
            .expect("There should always be atleast one world");
        let mut border = world.worldborder.lock().await;

        let Ok(distance) = DISTANCE_CONSUMER.find_arg_default_name(args)? else {
            sender
                .send_message(
                    TextComponent::text_string(format!(
                        "{} is out of bounds.",
                        DISTANCE_CONSUMER.default_name()
                    ))
                    .color(Color::Named(NamedColor::Red)),
                )
                .await;
            return Ok(());
        };

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
        server: &Arc<Server>,
        args: &ConsumedArgs<'a>,
    ) -> Result<(), CommandError> {
        let world = server
            .worlds
            .first()
            .expect("There should always be atleast one world");
        let mut border = world.worldborder.lock().await;

        let Ok(distance) = DISTANCE_CONSUMER.find_arg_default_name(args)? else {
            sender
                .send_message(
                    TextComponent::text_string(format!(
                        "{} is out of bounds.",
                        DISTANCE_CONSUMER.default_name()
                    ))
                    .color(Color::Named(NamedColor::Red)),
                )
                .await;
            return Ok(());
        };
        let Ok(time) = TIME_CONSUMER.find_arg_default_name(args)? else {
            sender
                .send_message(
                    TextComponent::text_string(format!(
                        "{} is out of bounds.",
                        TIME_CONSUMER.default_name()
                    ))
                    .color(Color::Named(NamedColor::Red)),
                )
                .await;
            return Ok(());
        };

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
        server: &Arc<Server>,
        args: &ConsumedArgs<'a>,
    ) -> Result<(), CommandError> {
        let world = server
            .worlds
            .first()
            .expect("There should always be atleast one world");
        let mut border = world.worldborder.lock().await;

        let Vector2 { x, z } = Position2DArgumentConsumer.find_arg_default_name(args)?;

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
        server: &Arc<Server>,
        args: &ConsumedArgs<'a>,
    ) -> Result<(), CommandError> {
        let world = server
            .worlds
            .first()
            .expect("There should always be atleast one world");
        let mut border = world.worldborder.lock().await;

        let Ok(damage_per_block) = DAMAGE_PER_BLOCK_CONSUMER.find_arg_default_name(args)? else {
            sender
                .send_message(
                    TextComponent::text_string(format!(
                        "{} is out of bounds.",
                        DAMAGE_PER_BLOCK_CONSUMER.default_name()
                    ))
                    .color(Color::Named(NamedColor::Red)),
                )
                .await;
            return Ok(());
        };

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
        server: &Arc<Server>,
        args: &ConsumedArgs<'a>,
    ) -> Result<(), CommandError> {
        let world = server
            .worlds
            .first()
            .expect("There should always be atleast one world");
        let mut border = world.worldborder.lock().await;

        let Ok(buffer) = DAMAGE_BUFFER_CONSUMER.find_arg_default_name(args)? else {
            sender
                .send_message(
                    TextComponent::text_string(format!(
                        "{} is out of bounds.",
                        DAMAGE_BUFFER_CONSUMER.default_name()
                    ))
                    .color(Color::Named(NamedColor::Red)),
                )
                .await;
            return Ok(());
        };

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
        server: &Arc<Server>,
        args: &ConsumedArgs<'a>,
    ) -> Result<(), CommandError> {
        let world = server
            .worlds
            .first()
            .expect("There should always be atleast one world");
        let mut border = world.worldborder.lock().await;

        let Ok(distance) = WARNING_DISTANCE_CONSUMER.find_arg_default_name(args)? else {
            sender
                .send_message(
                    TextComponent::text_string(format!(
                        "{} is out of bounds.",
                        WARNING_DISTANCE_CONSUMER.default_name()
                    ))
                    .color(Color::Named(NamedColor::Red)),
                )
                .await;
            return Ok(());
        };

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
        server: &Arc<Server>,
        args: &ConsumedArgs<'a>,
    ) -> Result<(), CommandError> {
        let world = server
            .worlds
            .first()
            .expect("There should always be atleast one world");
        let mut border = world.worldborder.lock().await;

        let Ok(time) = TIME_CONSUMER.find_arg_default_name(args)? else {
            sender
                .send_message(
                    TextComponent::text_string(format!(
                        "{} is out of bounds.",
                        TIME_CONSUMER.default_name()
                    ))
                    .color(Color::Named(NamedColor::Red)),
                )
                .await;
            return Ok(());
        };

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

static DISTANCE_CONSUMER: BoundedNumArgumentConsumer<f64> =
    BoundedNumArgumentConsumer::new().min(0.0).name("distance");

static TIME_CONSUMER: BoundedNumArgumentConsumer<i32> =
    BoundedNumArgumentConsumer::new().min(0).name("time");

static DAMAGE_PER_BLOCK_CONSUMER: BoundedNumArgumentConsumer<f32> =
    BoundedNumArgumentConsumer::new()
        .min(0.0)
        .name("damage_per_block");

static DAMAGE_BUFFER_CONSUMER: BoundedNumArgumentConsumer<f32> =
    BoundedNumArgumentConsumer::new().min(0.0).name("buffer");

static WARNING_DISTANCE_CONSUMER: BoundedNumArgumentConsumer<i32> =
    BoundedNumArgumentConsumer::new().min(0).name("distance");

pub fn init_command_tree<'a>() -> CommandTree<'a> {
    CommandTree::new(NAMES, DESCRIPTION)
        .with_child(
            literal("add").with_child(
                argument_default_name(&DISTANCE_CONSUMER)
                    .execute(&WorldborderAddExecutor)
                    .with_child(
                        argument_default_name(&TIME_CONSUMER).execute(&WorldborderAddTimeExecutor),
                    ),
            ),
        )
        .with_child(literal("center").with_child(
            argument_default_name(&Position2DArgumentConsumer).execute(&WorldborderCenterExecutor),
        ))
        .with_child(
            literal("damage")
                .with_child(
                    literal("amount").with_child(
                        argument_default_name(&DAMAGE_PER_BLOCK_CONSUMER)
                            .execute(&WorldborderDamageAmountExecutor),
                    ),
                )
                .with_child(
                    literal("buffer").with_child(
                        argument_default_name(&DAMAGE_BUFFER_CONSUMER)
                            .execute(&WorldborderDamageBufferExecutor),
                    ),
                ),
        )
        .with_child(literal("get").execute(&WorldborderGetExecutor))
        .with_child(
            literal("set").with_child(
                argument_default_name(&DISTANCE_CONSUMER)
                    .execute(&WorldborderSetExecutor)
                    .with_child(
                        argument_default_name(&TIME_CONSUMER).execute(&WorldborderSetTimeExecutor),
                    ),
            ),
        )
        .with_child(
            literal("warning")
                .with_child(
                    literal("distance").with_child(
                        argument_default_name(&WARNING_DISTANCE_CONSUMER)
                            .execute(&WorldborderWarningDistanceExecutor),
                    ),
                )
                .with_child(literal("time").with_child(
                    argument_default_name(&TIME_CONSUMER).execute(&WorldborderWarningTimeExecutor),
                )),
        )
}
