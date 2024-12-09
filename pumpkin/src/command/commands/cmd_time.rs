use async_trait::async_trait;
use pumpkin_core::text::color::{Color, NamedColor};
use pumpkin_core::text::TextComponent;

use crate::command::args::arg_bounded_num::BoundedNumArgumentConsumer;
use crate::command::args::FindArgDefaultName;
use crate::command::tree_builder::{argument_default_name, literal};
use crate::command::{
    tree::CommandTree, tree_builder::require, CommandError, CommandExecutor, CommandSender,
    ConsumedArgs,
};
use pumpkin_core::permission::PermissionLvl;

const NAMES: [&str; 1] = ["time"];

const DESCRIPTION: &str = "Query the world time.";

// TODO: This should be either higher or not bounded
static ARG_NUMBER: BoundedNumArgumentConsumer<i32> = BoundedNumArgumentConsumer::new()
    .name("time")
    .min(0)
    .max(24000);

#[derive(Clone, Copy)]
enum Mode {
    Add,
    Set,
}

#[derive(Clone, Copy)]
enum QueryMode {
    DayTime,
    GameTime,
    Day,
}

struct TimeQueryExecutor(QueryMode);

#[async_trait]
impl CommandExecutor for TimeQueryExecutor {
    async fn execute<'a>(
        &self,
        sender: &mut CommandSender<'a>,
        server: &crate::server::Server,
        _args: &ConsumedArgs<'a>,
    ) -> Result<(), CommandError> {
        let mode = self.0;
        let world = server
            .worlds
            .first()
            .expect("There should always be at least one world");
        let level_time = world.level_time.lock().await;

        let msg = match mode {
            QueryMode::DayTime => {
                let curr_time = level_time.query_daytime();
                format!("Daytime is: {curr_time}")
            }
            QueryMode::GameTime => {
                let curr_time = level_time.query_gametime();
                format!("Gametime is: {curr_time}")
            }
            QueryMode::Day => {
                let curr_time = level_time.query_day();
                format!("Day is: {curr_time}")
            }
        };

        sender.send_message(TextComponent::text(&msg)).await;
        Ok(())
    }
}

struct TimeChangeExecutor(Mode);

#[async_trait]
impl CommandExecutor for TimeChangeExecutor {
    async fn execute<'a>(
        &self,
        sender: &mut CommandSender<'a>,
        server: &crate::server::Server,
        args: &ConsumedArgs<'a>,
    ) -> Result<(), CommandError> {
        let time_count = match ARG_NUMBER.find_arg_default_name(args) {
            Err(_) => 1,
            Ok(Ok(count)) => count,
            Ok(Err(())) => {
                sender
                    .send_message(
                        TextComponent::text("Time is too large or too small.")
                            .color(Color::Named(NamedColor::Red)),
                    )
                    .await;
                return Ok(());
            }
        };
        let mode = self.0;
        let world = server
            .worlds
            .first()
            .expect("There should always be at least one world");
        let mut level_time = world.level_time.lock().await;

        let msg = match mode {
            Mode::Add => {
                // add
                level_time.add_time(time_count.into());
                level_time.send_time(world).await;
                let curr_time = level_time.query_daytime();
                format!("Added {time_count} time for result: {curr_time}")
            }
            Mode::Set => {
                // set
                level_time.set_time(time_count.into());
                level_time.send_time(world).await;
                format!("Changed time to: {time_count}")
            }
        };

        sender.send_message(TextComponent::text(&msg)).await;
        Ok(())
    }
}

pub fn init_command_tree<'a>() -> CommandTree<'a> {
    CommandTree::new(NAMES, DESCRIPTION).with_child(
        require(&|sender| sender.has_permission_lvl(PermissionLvl::Two))
            .with_child(literal("add").with_child(
                argument_default_name(&ARG_NUMBER).execute(&TimeChangeExecutor(Mode::Add)),
            ))
            .with_child(
                literal("query")
                    .with_child(literal("daytime").execute(&TimeQueryExecutor(QueryMode::DayTime)))
                    .with_child(
                        literal("gametime").execute(&TimeQueryExecutor(QueryMode::GameTime)),
                    )
                    .with_child(literal("day").execute(&TimeQueryExecutor(QueryMode::Day))),
            )
            .with_child(literal("set").with_child(
                argument_default_name(&ARG_NUMBER).execute(&TimeChangeExecutor(Mode::Set)),
            )),
    )
}
