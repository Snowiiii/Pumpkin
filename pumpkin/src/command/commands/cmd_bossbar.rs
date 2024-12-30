use crate::command::args::arg_bool::BoolArgConsumer;
use crate::command::args::arg_bossbar_color::BossbarColorArgumentConsumer;
use crate::command::args::arg_bossbar_style::BossbarStyleArgumentConsumer;
use crate::command::args::arg_bounded_num::BoundedNumArgumentConsumer;
use crate::command::args::arg_players::PlayersArgumentConsumer;
use crate::command::args::arg_resource_location::ResourceLocationArgumentConsumer;

use crate::command::args::{
    ConsumedArgs, DefaultNameArgConsumer, FindArg, FindArgDefaultName,
};
use crate::command::dispatcher::CommandError;

use crate::command::tree::CommandTree;
use crate::command::tree_builder::{argument, argument_default_name, literal};
use crate::command::{CommandExecutor, CommandSender};
use crate::server::Server;
use crate::world::bossbar::Bossbar;
use crate::world::custom_bossbar::BossbarUpdateError;
use async_trait::async_trait;
use pumpkin_core::text::color::{Color, NamedColor};
use pumpkin_core::text::TextComponent;
use uuid::Uuid;
use crate::command::args::arg_textcomponent::TextComponentArgConsumer;

const NAMES: [&str; 1] = ["bossbar"];
const DESCRIPTION: &str = "Display bossbar";

const ARG_NAME: &str = "name";

const ARG_VISIBLE: &str = "visible";

const fn autocomplete_consumer() -> ResourceLocationArgumentConsumer {
    ResourceLocationArgumentConsumer::new(true)
}
const fn non_autocomplete_consumer() -> ResourceLocationArgumentConsumer {
    ResourceLocationArgumentConsumer::new(false)
}

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
    Players(bool),
    Style,
    Value,
    Visible,
}

struct BossbarAddExecuter;

#[async_trait]
#[expect(clippy::inefficient_to_string)]
impl CommandExecutor for BossbarAddExecuter {
    async fn execute<'a>(
        &self,
        sender: &mut CommandSender<'a>,
        server: &Server,
        args: &ConsumedArgs<'a>,
    ) -> Result<(), CommandError> {
        let namespace = non_autocomplete_consumer().find_arg_default_name(args)?;
        let text_component = TextComponentArgConsumer::find_arg(args, ARG_NAME)?;

        if server.bossbars.lock().await.has_bossbar(namespace) {
            send_error_message(
                sender,
                format!("A boss bar already exists with the ID '{namespace}'"),
            )
            .await;
            return Ok(());
        }

        let bossbar = Bossbar::new(text_component);
        server
            .bossbars
            .lock()
            .await
            .create_bossbar(namespace.to_string(), bossbar.clone());

        sender
            .send_message(
                TextComponent::text("Created custom bossbar [")
                    .add_child(bossbar.title)
                    .add_child(TextComponent::text("]")),
            )
            .await;

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
        let namespace = autocomplete_consumer().find_arg_default_name(args)?;

        let Some(bossbar) = server.bossbars.lock().await.get_bossbar(namespace) else {
            send_error_message(
                sender,
                format!("No bossbar exists with the ID '{namespace}'"),
            )
            .await;
            return Ok(());
        };

        match self.0 {
            CommandValueGet::Max => {
                send_prefix_success_message(
                    sender,
                    bossbar.bossbar_data.title,
                    format!("has a maximum of {}", bossbar.max),
                )
                .await;
                return Ok(());
            }
            CommandValueGet::Players => {}
            CommandValueGet::Value => {
                send_prefix_success_message(
                    sender,
                    bossbar.bossbar_data.title,
                    format!("has a value of {}", bossbar.value),
                )
                .await;
                return Ok(());
            }
            CommandValueGet::Visible => {
                let state = if bossbar.visible { "shown" } else { "hidden" };
                send_prefix_success_message(
                    sender,
                    bossbar.bossbar_data.title,
                    format!("is currently {state}"),
                )
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
            send_success_message(sender, "There are no custom bossbars active".to_string()).await;
            return Ok(());
        };

        let mut bossbars_text = TextComponent::text(format!(
            "There are {} custom bossbar(s) active: ",
            bossbars.len()
        ));
        for (i, bossbar) in bossbars.iter().enumerate() {
            if i == 0 {
                bossbars_text = bossbars_text
                    .add_child(TextComponent::text("["))
                    .add_child(bossbar.bossbar_data.title.clone())
                    .add_child(TextComponent::text("]"));
            } else {
                bossbars_text = bossbars_text
                    .add_child(TextComponent::text(", ["))
                    .add_child(bossbar.bossbar_data.title.clone())
                    .add_child(TextComponent::text("]"));
            }
        }

        sender.send_message(bossbars_text).await;
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
        let namespace = autocomplete_consumer().find_arg_default_name(args)?;

        if !server.bossbars.lock().await.has_bossbar(namespace) {
            send_error_message(
                sender,
                format!("A boss bar already exists with the ID '{namespace}'"),
            )
            .await;
            return Ok(());
        }

        match server
            .bossbars
            .lock()
            .await
            .remove_bossbar(server, namespace.to_string())
            .await
        {
            Ok(()) => {}
            Err(err) => {
                handle_bossbar_error(sender, err).await;
                return Ok(());
            }
        };

        Ok(())
    }
}

struct BossbarSetExecuter(CommandValueSet);

#[async_trait]
impl CommandExecutor for BossbarSetExecuter {
    #[expect(clippy::too_many_lines)]
    async fn execute<'a>(
        &self,
        sender: &mut CommandSender<'a>,
        server: &Server,
        args: &ConsumedArgs<'a>,
    ) -> Result<(), CommandError> {
        let namespace = autocomplete_consumer().find_arg_default_name(args)?;

        let Some(bossbar) = server.bossbars.lock().await.get_bossbar(namespace) else {
            handle_bossbar_error(
                sender,
                BossbarUpdateError::InvalidResourceLocation(namespace.to_string()),
            )
            .await;
            return Ok(());
        };

        match self.0 {
            CommandValueSet::Color => {
                let color = BossbarColorArgumentConsumer.find_arg_default_name(args)?;
                match server
                    .bossbars
                    .lock()
                    .await
                    .update_color(server, namespace.to_string(), color.clone())
                    .await
                {
                    Ok(()) => {}
                    Err(err) => {
                        handle_bossbar_error(sender, err).await;
                        return Ok(());
                    }
                }
                send_prefix_success_message(
                    sender,
                    bossbar.bossbar_data.title,
                    String::from("has changed color"),
                )
                .await;
                Ok(())
            }
            CommandValueSet::Max => {
                let Ok(max_value) = max_value_consumer().find_arg_default_name(args)? else {
                    send_error_message(
                        sender,
                        format!("{} is out of bounds.", max_value_consumer().default_name()),
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
                    Ok(()) => {}
                    Err(err) => {
                        handle_bossbar_error(sender, err).await;
                        return Ok(());
                    }
                }

                send_prefix_success_message(
                    sender,
                    bossbar.bossbar_data.title,
                    format!("has changed maximum to {}", max_value),
                )
                .await;
                Ok(())
            }
            CommandValueSet::Name => {
                let text_component = TextComponentArgConsumer::find_arg(args, ARG_NAME)?;
                match server
                    .bossbars
                    .lock()
                    .await
                    .update_name(server, namespace, text_component.clone())
                    .await
                {
                    Ok(()) => {}
                    Err(err) => {
                        handle_bossbar_error(sender, err).await;
                        return Ok(());
                    }
                }

                send_prefix_success_message(sender, text_component, String::from("has been renamed"))
                    .await;
                Ok(())
            }
            CommandValueSet::Players(has_players) => {
                if !has_players {
                    match server
                        .bossbars
                        .lock()
                        .await
                        .update_players(server, namespace.to_string(), vec![])
                        .await
                    {
                        Ok(()) => {}
                        Err(err) => {
                            handle_bossbar_error(sender, err).await;
                            return Ok(());
                        }
                    }
                    send_prefix_success_message(
                        sender,
                        bossbar.bossbar_data.title,
                        String::from("no longer has any players"),
                    )
                    .await;
                    return Ok(());
                }

                //TODO: Confirm that this is the vanilla way
                let targets = PlayersArgumentConsumer.find_arg_default_name(args)?;
                let players: Vec<Uuid> =
                    targets.iter().map(|player| player.gameprofile.id).collect();
                let count = players.len();

                match server
                    .bossbars
                    .lock()
                    .await
                    .update_players(server, namespace.to_string(), players)
                    .await
                {
                    Ok(()) => {}
                    Err(err) => {
                        handle_bossbar_error(sender, err).await;
                        return Ok(());
                    }
                }

                let player_names: Vec<String> = targets
                    .iter()
                    .map(|player| player.gameprofile.name.clone())
                    .collect();

                send_prefix_success_message(
                    sender,
                    bossbar.bossbar_data.title,
                    format!("now has {count} player(s): {}", player_names.join(", ")),
                )
                .await;
                Ok(())
            }
            CommandValueSet::Style => {
                let style = BossbarStyleArgumentConsumer.find_arg_default_name(args)?;
                match server
                    .bossbars
                    .lock()
                    .await
                    .update_division(server, namespace.to_string(), style.clone())
                    .await
                {
                    Ok(()) => {}
                    Err(err) => {
                        handle_bossbar_error(sender, err).await;
                        return Ok(());
                    }
                }
                send_prefix_success_message(
                    sender,
                    bossbar.bossbar_data.title,
                    String::from("has changed style"),
                )
                .await;
                Ok(())
            }
            CommandValueSet::Value => {
                let Ok(value) = value_consumer().find_arg_default_name(args)? else {
                    send_error_message(
                        sender,
                        format!("{} is out of bounds.", value_consumer().default_name()),
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
                    Ok(()) => {}
                    Err(err) => {
                        handle_bossbar_error(sender, err).await;
                        return Ok(());
                    }
                }

                send_prefix_success_message(
                    sender,
                    bossbar.bossbar_data.title,
                    format!("changed value to {}", value),
                )
                .await;
                Ok(())
            }
            CommandValueSet::Visible => {
                let visibility = BoolArgConsumer::find_arg(args, ARG_VISIBLE)?;

                match server
                    .bossbars
                    .lock()
                    .await
                    .update_visibility(server, namespace.to_string(), visibility)
                    .await
                {
                    Ok(()) => {}
                    Err(err) => {
                        handle_bossbar_error(sender, err).await;
                        return Ok(());
                    }
                }

                let state = if visibility { "visible" } else { "hidden" };
                send_prefix_success_message(
                    sender,
                    bossbar.bossbar_data.title,
                    format!("is now {state}"),
                )
                .await;
                Ok(())
            }
        }
    }
}

fn max_value_consumer() -> BoundedNumArgumentConsumer<i32> {
    BoundedNumArgumentConsumer::new().min(0).name("max")
}

fn value_consumer() -> BoundedNumArgumentConsumer<i32> {
    BoundedNumArgumentConsumer::new().min(0).name("value")
}

pub fn init_command_tree() -> CommandTree {
    CommandTree::new(NAMES, DESCRIPTION)
        .with_child(
            literal("add").with_child(
                argument_default_name(non_autocomplete_consumer())
                    .with_child(argument(ARG_NAME, TextComponentArgConsumer).execute(BossbarAddExecuter)),
            ),
        )
        .with_child(
            literal("get").with_child(
                argument_default_name(autocomplete_consumer())
                    .with_child(literal("max").execute(BossbarGetExecuter(CommandValueGet::Max)))
                    .with_child(
                        literal("players").execute(BossbarGetExecuter(CommandValueGet::Players)),
                    )
                    .with_child(
                        literal("value").execute(BossbarGetExecuter(CommandValueGet::Value)),
                    )
                    .with_child(
                        literal("visible").execute(BossbarGetExecuter(CommandValueGet::Visible)),
                    ),
            ),
        )
        .with_child(literal("list").execute(BossbarListExecuter))
        .with_child(literal("remove").with_child(
            argument_default_name(autocomplete_consumer()).execute(BossbarRemoveExecuter),
        ))
        .with_child(
            literal("set").with_child(
                argument_default_name(autocomplete_consumer())
                    .with_child(
                        literal("color").with_child(
                            argument_default_name(BossbarColorArgumentConsumer)
                                .execute(BossbarSetExecuter(CommandValueSet::Color)),
                        ),
                    )
                    .with_child(
                        literal("max").with_child(
                            argument_default_name(max_value_consumer())
                                .execute(BossbarSetExecuter(CommandValueSet::Max)),
                        ),
                    )
                    .with_child(
                        literal("name").with_child(
                            argument(ARG_NAME, TextComponentArgConsumer)
                                .execute(BossbarSetExecuter(CommandValueSet::Name)),
                        ),
                    )
                    .with_child(
                        literal("players")
                            .with_child(
                                argument_default_name(PlayersArgumentConsumer)
                                    .execute(BossbarSetExecuter(CommandValueSet::Players(true))),
                            )
                            .execute(BossbarSetExecuter(CommandValueSet::Players(false))),
                    )
                    .with_child(
                        literal("style").with_child(
                            argument_default_name(BossbarStyleArgumentConsumer)
                                .execute(BossbarSetExecuter(CommandValueSet::Style)),
                        ),
                    )
                    .with_child(
                        literal("value").with_child(
                            argument_default_name(value_consumer())
                                .execute(BossbarSetExecuter(CommandValueSet::Value)),
                        ),
                    )
                    .with_child(
                        literal("visible").with_child(
                            argument(ARG_VISIBLE, BoolArgConsumer)
                                .execute(BossbarSetExecuter(CommandValueSet::Visible)),
                        ),
                    ),
            ),
        )
}

fn bossbar_prefix(title: TextComponent, trailing: String) -> TextComponent {
    TextComponent::text("Custom bossbar [")
        .add_child(title)
        .add_child(TextComponent::text(format!("] {trailing}")))
}

async fn send_prefix_success_message(
    sender: &CommandSender<'_>,
    title: TextComponent,
    message: String,
) {
    sender.send_message(bossbar_prefix(title, message)).await;
}
async fn send_success_message(sender: &CommandSender<'_>, message: String) {
    sender.send_message(TextComponent::text(message)).await;
}

async fn send_error_message(sender: &CommandSender<'_>, message: String) {
    sender
        .send_message(TextComponent::text(message).color(Color::Named(NamedColor::Red)))
        .await;
}

async fn handle_bossbar_error(sender: &CommandSender<'_>, error: BossbarUpdateError) {
    match error {
        BossbarUpdateError::InvalidResourceLocation(location) => {
            send_error_message(
                sender,
                format!("No bossbar exists with the ID '{location}'"),
            )
            .await;
        }
        BossbarUpdateError::NoChanges(message) => {
            send_error_message(sender, format!("Nothing changed. {message}")).await;
        }
    }
}
