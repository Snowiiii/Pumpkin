use async_trait::async_trait;
use pumpkin_core::text::color::{Color, NamedColor};
use pumpkin_core::text::TextComponent;

use crate::command::args::arg_bounded_num::BoundedNumArgumentConsumer;
use crate::command::args::FindArgDefaultName;
use crate::command::tree_builder::argument_default_name;
use crate::{
    command::{
        tree::CommandTree, tree_builder::require, CommandError, CommandExecutor, CommandSender,
        ConsumedArgs,
    },
    entity::player::PermissionLvl,
};

const NAMES: [&str; 1] = ["time"];

const DESCRIPTION: &str = "Query the world time.";

// const ARG_ACTION: &str = "action";

// const ARG_NUMBER: &str = "number";

static ARG_NUMBER: BoundedNumArgumentConsumer<i32> = BoundedNumArgumentConsumer::new()
    .name("count")
    .min(0)
    .max(24000);

struct TimeExecutor;

#[async_trait]
impl CommandExecutor for TimeExecutor {
    async fn execute<'a>(
        &self,
        sender: &mut CommandSender<'a>,
        server: &crate::server::Server,
        args: &ConsumedArgs<'a>,
    ) -> Result<(), CommandError> {
        // let Some(Arg::Msg(msg)) = args.get(ARG_MESSAGE) else {
        //     return Err(InvalidConsumption(Some(ARG_MESSAGE.into())));
        // };
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
        // logic
        let world = server
            .worlds
            .first()
            .expect("There should always be atleast one world");
        let mut level_time = world.level_time.lock().await;
        level_time.set_time(time_count.into()).await;
        let mut owned_string: String = "Change time to: ".to_owned();
        owned_string.push_str(&time_count.to_string());
        sender
            .send_message(TextComponent::text(&owned_string))
            .await;
        Ok(())
    }
}

pub fn init_command_tree<'a>() -> CommandTree<'a> {
    CommandTree::new(NAMES, DESCRIPTION).with_child(
        require(&|sender| sender.has_permission_lvl(PermissionLvl::Two))
            .with_child(argument_default_name(&ARG_NUMBER).execute(&TimeExecutor)),
    )
}
