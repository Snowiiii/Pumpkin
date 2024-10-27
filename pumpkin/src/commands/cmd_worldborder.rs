use async_trait::async_trait;
use pumpkin_core::text::TextComponent;

use crate::server::Server;

use super::{
    dispatcher::InvalidTreeError,
    tree::{ArgumentConsumer, CommandTree, ConsumedArgs, RawArgs},
    tree_builder::{argument, literal},
    CommandExecutor, CommandSender,
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
        let world = server.worlds.first().expect("goofy");
        let border = world.worldborder.lock().await;

        let diameter = border.new_diameter.round() as i32;
        sender
            .send_message(TextComponent::text(
                &format!("Border diameter: {diameter}",),
            ))
            .await;
        Ok(())
    }
}

struct WorldborderSetExecutor;

#[async_trait]
impl CommandExecutor for WorldborderSetExecutor {
    async fn execute<'a>(
        &self,
        _sender: &mut CommandSender<'a>,
        server: &Server,
        args: &ConsumedArgs<'a>,
    ) -> Result<(), InvalidTreeError> {
        let world = server.worlds.first().expect("goofy");
        let mut border = world.worldborder.lock().await;

        let distance = args.get("distance").unwrap().parse::<f64>().unwrap();

        border.set_diameter(world, distance, None).await;
        Ok(())
    }
}

struct WorldborderSetTimeExecutor;

#[async_trait]
impl CommandExecutor for WorldborderSetTimeExecutor {
    async fn execute<'a>(
        &self,
        _sender: &mut CommandSender<'a>,
        server: &Server,
        args: &ConsumedArgs<'a>,
    ) -> Result<(), InvalidTreeError> {
        let world = server.worlds.first().expect("goofy");
        let mut border = world.worldborder.lock().await;

        let distance = args.get("distance").unwrap().parse::<f64>().unwrap();
        let time = args.get("time").unwrap().parse::<i32>().unwrap();

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
        _sender: &mut CommandSender<'a>,
        server: &Server,
        args: &ConsumedArgs<'a>,
    ) -> Result<(), InvalidTreeError> {
        let world = server.worlds.first().expect("goofy");
        let mut border = world.worldborder.lock().await;

        let distance = args.get("distance").unwrap().parse::<f64>().unwrap();

        border.add_diameter(world, distance, None).await;
        Ok(())
    }
}

struct WorldborderAddTimeExecutor;

#[async_trait]
impl CommandExecutor for WorldborderAddTimeExecutor {
    async fn execute<'a>(
        &self,
        _sender: &mut CommandSender<'a>,
        server: &Server,
        args: &ConsumedArgs<'a>,
    ) -> Result<(), InvalidTreeError> {
        let world = server.worlds.first().expect("goofy");
        let mut border = world.worldborder.lock().await;

        let distance = args.get("distance").unwrap().parse::<f64>().unwrap();
        let time = args.get("time").unwrap().parse::<i32>().unwrap();

        border
            .add_diameter(world, distance, Some(i64::from(time) * 1000))
            .await;
        Ok(())
    }
}

struct WorldborderCenterExecutor;

#[async_trait]
impl CommandExecutor for WorldborderCenterExecutor {
    async fn execute<'a>(
        &self,
        _sender: &mut CommandSender<'a>,
        server: &Server,
        args: &ConsumedArgs<'a>,
    ) -> Result<(), InvalidTreeError> {
        let world = server.worlds.first().expect("goofy");
        let mut border = world.worldborder.lock().await;

        let x = args.get("x").unwrap().parse::<f64>().unwrap();
        let z = args.get("z").unwrap().parse::<f64>().unwrap();

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
        let _world = server.worlds.first().expect("goofy");
        // let mut border = world.worldborder.lock().await;

        let _damage_per_block = args
            .get("damage_per_block")
            .unwrap()
            .parse::<f32>()
            .unwrap();

        sender.send_message(TextComponent::text("todo")).await;
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
        let _world = server.worlds.first().expect("goofy");
        // let mut border = world.worldborder.lock().await;

        let _buffer = args.get("distance").unwrap().parse::<f32>().unwrap();

        sender.send_message(TextComponent::text("todo")).await;
        Ok(())
    }
}

struct WorldborderWarningDistanceExecutor;

#[async_trait]
impl CommandExecutor for WorldborderWarningDistanceExecutor {
    async fn execute<'a>(
        &self,
        _sender: &mut CommandSender<'a>,
        server: &Server,
        args: &ConsumedArgs<'a>,
    ) -> Result<(), InvalidTreeError> {
        let world = server.worlds.first().expect("goofy");
        let mut border = world.worldborder.lock().await;

        let distance = args.get("distance").unwrap().parse::<i32>().unwrap();

        border.set_warning_distance(world, distance).await;
        Ok(())
    }
}

struct WorldborderWarningTimeExecutor;

#[async_trait]
impl CommandExecutor for WorldborderWarningTimeExecutor {
    async fn execute<'a>(
        &self,
        _sender: &mut CommandSender<'a>,
        server: &Server,
        args: &ConsumedArgs<'a>,
    ) -> Result<(), InvalidTreeError> {
        let world = server.worlds.first().expect("goofy");
        let mut border = world.worldborder.lock().await;

        let time = args.get("time").unwrap().parse::<i32>().unwrap();

        border.set_warning_delay(world, time).await;
        Ok(())
    }
}

pub struct SimpleArgConsumer;

#[async_trait]
impl ArgumentConsumer for SimpleArgConsumer {
    async fn consume<'a>(
        &self,
        _sender: &CommandSender<'a>,
        _server: &Server,
        args: &mut RawArgs<'a>,
    ) -> Result<String, Option<String>> {
        args.pop().ok_or(None).map(ToString::to_string)
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
        .with_child(
            literal("center").with_child(
                argument("x", &SimpleArgConsumer).with_child(
                    argument("z", &SimpleArgConsumer).execute(&WorldborderCenterExecutor),
                ),
            ),
        )
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
