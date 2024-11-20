use crate::command::args::arg_bounded_num::BoundedNumArgumentConsumer;
use crate::command::args::arg_resource_location::ResourceLocationArgumentConsumer;
use crate::command::args::arg_simple::SimpleArgConsumer;
use crate::command::args::{Arg, ConsumedArgs, DefaultNameArgConsumer, FindArgDefaultName};
use crate::command::dispatcher::CommandError;
use crate::command::dispatcher::CommandError::InvalidConsumption;
use crate::command::tree::CommandTree;
use crate::command::tree_builder::{argument, argument_default_name, literal};
use crate::command::{CommandExecutor, CommandSender};
use crate::entity::player::Player;
use crate::server::Server;
use crate::world::bossbar::Bossbar;
use crate::world::custom_bossbar::{BossbarUpdateError, CustomBossbar};
use async_trait::async_trait;
use png::chunk::tEXt;
use pumpkin_core::text::color::{Color, NamedColor};
use pumpkin_core::text::TextComponent;
use std::sync::Arc;
use uuid::Uuid;

const NAMES: [&str; 1] = ["bossbar"];
const DESCRIPTION: &str = "Display bossbar";

const ARG_NAME: &str = "name";

const AUTOCOMPLETE_CONSUMER: ResourceLocationArgumentConsumer =
    ResourceLocationArgumentConsumer::new(true);
const NON_AUTOCOMPLETE_CONSUMER: ResourceLocationArgumentConsumer =
    ResourceLocationArgumentConsumer::new(false);

enum CommandValueGet {
    Max,
    Players,
    Value,
    Visible,
}

enum CommandValueSet {
    Color,
    Max,
    Name,
    Players,
    Style,
    Value,
    Visible,
}

struct BossbarAddExecuter;

#[async_trait]
impl CommandExecutor for BossbarAddExecuter {
    async fn execute<'a>(
        &self,
        sender: &mut CommandSender<'a>,
        server: &Server,
        args: &ConsumedArgs<'a>,
    ) -> Result<(), CommandError> {
        if let Some(player) = sender.as_player() {
            let namespace = NON_AUTOCOMPLETE_CONSUMER.find_arg_default_name(args)?;
            let Some(Arg::Simple(name)) = args.get(ARG_NAME) else {
                return Err(InvalidConsumption(Some(ARG_NAME.into())));
            };

            if server
                .bossbars
                .lock()
                .await
                .has_bossbar(namespace.to_string())
            {
                sender
                    .send_message(
                        TextComponent::text(
                            format!("A boss bar already exists with the ID '{namespace}'").as_str(),
                        )
                        .color(Color::Named(NamedColor::Red)),
                    )
                    .await;
                return Ok(());
            }

            let bossbar = Bossbar::new(name.clone());
            server
                .bossbars
                .lock()
                .await
                .create_bossbar(namespace.to_string(), bossbar.clone());

            let mut uuids: Vec<Uuid> = vec![];

            //TODO: Remove after debugging
            if sender.is_player() {
                let te = sender.as_player().unwrap().gameprofile.id;
                uuids.push(te);
            }

            server
                .bossbars
                .lock()
                .await
                .update_players(server, namespace.to_string(), uuids)
                .await;
        }

        Ok(())
    }
}

struct BossbarGetExecuter(CommandValueGet);

#[async_trait]
impl CommandExecutor for BossbarGetExecuter {
    async fn execute<'a>(
        &self,
        sender: &mut CommandSender<'a>,
        server: &Server,
        args: &ConsumedArgs<'a>,
    ) -> Result<(), CommandError> {
        let namespace = AUTOCOMPLETE_CONSUMER.find_arg_default_name(args)?;

        let Some(bossbar) = server
            .bossbars
            .lock()
            .await
            .get_bossbar(namespace.to_string())
        else {
            sender
                .send_message(
                    TextComponent::text(
                        format!("No bossbar exists with the ID '{namespace}'").as_str(),
                    )
                    .color(Color::Named(NamedColor::Red)),
                )
                .await;
            return Ok(());
        };

        match self.0 {
            CommandValueGet::Max => {
                sender
                    .send_message(TextComponent::text(
                        format!(
                            "Custom bossbar [{}] has a maximum of {}",
                            bossbar.bossbar_data.title, bossbar.max
                        )
                        .as_str(),
                    ))
                    .await;
                return Ok(());
            }
            CommandValueGet::Players => {}
            CommandValueGet::Value => {
                sender
                    .send_message(TextComponent::text(
                        format!(
                            "Custom bossbar [{}] has a value of {}",
                            bossbar.bossbar_data.title, bossbar.value
                        )
                        .as_str(),
                    ))
                    .await;
                return Ok(());
            }
            CommandValueGet::Visible => {
                let state = if bossbar.visible { "shown" } else { "hidden" };
                sender
                    .send_message(TextComponent::text(
                        format!(
                            "Custom bossbar [{}] is currently {state}",
                            bossbar.bossbar_data.title
                        )
                        .as_str(),
                    ))
                    .await;
                return Ok(());
            }
        }

        Ok(())
    }
}

struct BossbarListExecuter;

#[async_trait]
impl CommandExecutor for BossbarListExecuter {
    async fn execute<'a>(
        &self,
        sender: &mut CommandSender<'a>,
        server: &Server,
        _args: &ConsumedArgs<'a>,
    ) -> Result<(), CommandError> {
        let bossbars = server.bossbars.lock().await.get_all_bossbars();
        let Some(bossbars) = bossbars else {
            sender
                .send_message(TextComponent::text("There are no custom bossbars active"))
                .await;
            return Ok(());
        };

        let mut bossbars_string = String::new();
        for (i, bossbar) in bossbars.iter().enumerate() {
            if i == 0 {
                bossbars_string += format!("[{}]", bossbar.bossbar_data.title).as_str();
            } else {
                bossbars_string += format!(", [{}]", bossbar.bossbar_data.title).as_str();
            }
        }

        sender
            .send_message(TextComponent::text(
                format!(
                    "There are {} custom bossbar(s) active: {}",
                    bossbars.len(),
                    bossbars_string
                )
                .as_str(),
            ))
            .await;
        Ok(())
    }
}

struct BossbarRemoveExecuter;

#[async_trait]
impl CommandExecutor for BossbarRemoveExecuter {
    async fn execute<'a>(
        &self,
        sender: &mut CommandSender<'a>,
        server: &Server,
        args: &ConsumedArgs<'a>,
    ) -> Result<(), CommandError> {
        let namespace = AUTOCOMPLETE_CONSUMER.find_arg_default_name(args)?;

        if !server
            .bossbars
            .lock()
            .await
            .has_bossbar(namespace.to_string())
        {
            sender
                .send_message(
                    TextComponent::text(
                        format!("A boss bar already exists with the ID '{namespace}'").as_str(),
                    )
                    .color(Color::Named(NamedColor::Red)),
                )
                .await;
            return Ok(());
        }

        server
            .bossbars
            .lock()
            .await
            .remove_bossbar(namespace.to_string());

        Ok(())
    }
}

struct BossbarSetExecuter(CommandValueSet);

#[async_trait]
impl CommandExecutor for BossbarSetExecuter {
    async fn execute<'a>(
        &self,
        sender: &mut CommandSender<'a>,
        server: &Server,
        args: &ConsumedArgs<'a>,
    ) -> Result<(), CommandError> {
        let namespace = AUTOCOMPLETE_CONSUMER.find_arg_default_name(args)?;

        let Some(bossbar) = server
            .bossbars
            .lock()
            .await
            .get_bossbar(namespace.to_string())
        else {
            handle_bossbar_error(sender, BossbarUpdateError::InvalidResourceLocation).await;
            return Ok(());
        };

        match self.0 {
            CommandValueSet::Color => {}
            CommandValueSet::Max => {
                let Ok(max_value) = MAX_VALUE_CONSUMER.find_arg_default_name(args)? else {
                    sender
                        .send_message(
                            TextComponent::text_string(format!(
                                "{} is out of bounds.",
                                MAX_VALUE_CONSUMER.default_name()
                            ))
                            .color(Color::Named(NamedColor::Red)),
                        )
                        .await;
                    return Ok(());
                };

                match server
                    .bossbars
                    .lock()
                    .await
                    .update_health(
                        server,
                        namespace.to_string(),
                        max_value as u32,
                        bossbar.value,
                    )
                    .await
                {
                    Ok(_) => {}
                    Err(err) => {
                        handle_bossbar_error(sender, err);
                        return Ok(());
                    }
                }

                sender
                    .send_message(TextComponent::text(
                        format!(
                            "Custom bossbar [{}] has changed maximum to {}",
                            bossbar.bossbar_data.title, max_value
                        )
                        .as_str(),
                    ))
                    .await;
                return Ok(());
            }
            CommandValueSet::Name => {}
            CommandValueSet::Players => {}
            CommandValueSet::Style => {}
            CommandValueSet::Value => {
                let Ok(value) = VALUE_CONSUMER.find_arg_default_name(args)? else {
                    sender
                        .send_message(
                            TextComponent::text_string(format!(
                                "{} is out of bounds.",
                                VALUE_CONSUMER.default_name()
                            ))
                            .color(Color::Named(NamedColor::Red)),
                        )
                        .await;
                    return Ok(());
                };

                match server
                    .bossbars
                    .lock()
                    .await
                    .update_health(server, namespace.to_string(), bossbar.max, value as u32)
                    .await
                {
                    Ok(_) => {}
                    Err(err) => {
                        handle_bossbar_error(sender, err);
                        return Ok(());
                    }
                }

                sender
                    .send_message(TextComponent::text(
                        format!(
                            "Custom bossbar [{}] has changed value to {}",
                            bossbar.bossbar_data.title, value
                        )
                        .as_str(),
                    ))
                    .await;
                return Ok(());
            }
            CommandValueSet::Visible => {
                let state = if bossbar.visible { "shown" } else { "hidden" };
                sender
                    .send_message(TextComponent::text(
                        format!(
                            "Custom bossbar [{}] is currently {state}",
                            bossbar.bossbar_data.title
                        )
                        .as_str(),
                    ))
                    .await;
                return Ok(());
            }
        }

        Ok(())
    }
}

async fn handle_bossbar_error(sender: &CommandSender, error: BossbarUpdateError) {
    match error {
        BossbarUpdateError::InvalidResourceLocation => {
            sender
                .send_message(
                    TextComponent::text(
                        format!("No bossbar exists with the ID '{namespace}'").as_str(),
                    )
                    .color(Color::Named(NamedColor::Red)),
                )
                .await;
        }
        BossbarUpdateError::NoChanges(message) => {
            sender
                .send_message(
                    TextComponent::text(format!("Nothing changed. {message}").as_str())
                        .color(Color::Named(NamedColor::Red)),
                )
                .await;
        }
    }
}

static MAX_VALUE_CONSUMER: BoundedNumArgumentConsumer<i32> = BoundedNumArgumentConsumer::new()
    .min(0)
    .max(2147483647)
    .name("max");

static VALUE_CONSUMER: BoundedNumArgumentConsumer<i32> = BoundedNumArgumentConsumer::new()
    .min(0)
    .max(2147483647)
    .name("value");

pub fn init_command_tree<'a>() -> CommandTree<'a> {
    CommandTree::new(NAMES, DESCRIPTION)
        .with_child(
            literal("add").with_child(
                argument_default_name(&NON_AUTOCOMPLETE_CONSUMER).with_child(
                    argument(ARG_NAME, &SimpleArgConsumer).execute(&BossbarAddExecuter),
                ),
            ),
        )
        .with_child(
            literal("get").with_child(
                argument_default_name(&AUTOCOMPLETE_CONSUMER)
                    .with_child(literal("max").execute(&BossbarGetExecuter(CommandValueGet::Max)))
                    .with_child(
                        literal("players").execute(&BossbarGetExecuter(CommandValueGet::Players)),
                    )
                    .with_child(
                        literal("value").execute(&BossbarGetExecuter(CommandValueGet::Value)),
                    )
                    .with_child(
                        literal("visible").execute(&BossbarGetExecuter(CommandValueGet::Visible)),
                    ),
            ),
        )
        .with_child(literal("list").execute(&BossbarListExecuter))
        .with_child(literal("remove").with_child(
            argument_default_name(&AUTOCOMPLETE_CONSUMER).execute(&BossbarRemoveExecuter),
        ))
        .with_child(
            literal("set").with_child(
                argument_default_name(&AUTOCOMPLETE_CONSUMER)
                    .with_child(
                        literal("max").with_child(
                            argument_default_name(&MAX_VALUE_CONSUMER)
                                .execute(&BossbarSetExecuter(CommandValueGet::Max)),
                        ),
                    )
                    .with_child(
                        literal("players").execute(&BossbarSetExecuter(CommandValueGet::Players)),
                    )
                    .with_child(
                        literal("value").with_child(
                            argument_default_name(&VALUE_CONSUMER)
                                .execute(&BossbarSetExecuter(CommandValueGet::Value)),
                        ),
                    )
                    .with_child(
                        literal("visible").execute(&BossbarSetExecuter(CommandValueGet::Visible)),
                    ),
            ),
        )
}
