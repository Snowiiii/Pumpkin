use async_trait::async_trait;
use num_traits::ToPrimitive;
use pumpkin_core::text::click::ClickEvent;
use pumpkin_core::text::color::{Color, NamedColor};
use pumpkin_core::text::TextComponent;

use crate::command::args::arg_bounded_num::BoundedNumArgumentConsumer;
use crate::command::args::arg_command::CommandTreeArgumentConsumer;
use crate::command::args::{Arg, ConsumedArgs, FindArgDefaultName};
use crate::command::dispatcher::CommandError;
use crate::command::dispatcher::CommandError::InvalidConsumption;
use crate::command::tree::{Command, CommandTree};
use crate::command::tree_builder::{argument, argument_default_name};
use crate::command::{CommandExecutor, CommandSender};
use crate::server::Server;

const NAMES: [&str; 3] = ["help", "h", "?"];

const DESCRIPTION: &str = "Print a help message.";

const ARG_COMMAND: &str = "command";

const COMMANDS_PER_PAGE: i32 = 7;

static PAGE_NUMBER_CONSUMER: BoundedNumArgumentConsumer<i32> =
    BoundedNumArgumentConsumer::new().name("page").min(1);

struct CommandHelpExecutor;

#[async_trait]
impl CommandExecutor for CommandHelpExecutor {
    async fn execute<'a>(
        &self,
        sender: &mut CommandSender<'a>,
        _server: &Server,
        args: &ConsumedArgs<'a>,
    ) -> Result<(), CommandError> {
        let Some(Arg::CommandTree(tree)) = args.get(&ARG_COMMAND) else {
            return Err(InvalidConsumption(Some(ARG_COMMAND.into())));
        };

        let command_names = tree.names.join(", /");
        let usage = format!("{tree}");
        let description = tree.description;

        let header_text = format!(" Help - /{} ", tree.names[0]);

        let mut message = TextComponent::text("")
            .add_child(
                TextComponent::text_string("-".repeat((52 - header_text.len()) / 2) + " ")
                    .color_named(NamedColor::Yellow),
            )
            .add_child(TextComponent::text(&header_text))
            .add_child(
                TextComponent::text_string(
                    " ".to_owned() + &"-".repeat((52 - header_text.len()) / 2) + "\n",
                )
                .color_named(NamedColor::Yellow),
            )
            .add_child(
                TextComponent::text("Command: ")
                    .color_named(NamedColor::Aqua)
                    .add_child(
                        TextComponent::text_string(format!("/{command_names}"))
                            .color_named(NamedColor::Gold)
                            .bold(),
                    )
                    .add_child(TextComponent::text("\n").color_named(NamedColor::White))
                    .click_event(ClickEvent::SuggestCommand(
                        format!("/{}", tree.names[0]).into(),
                    )),
            )
            .add_child(
                TextComponent::text("Description: ")
                    .color_named(NamedColor::Aqua)
                    .add_child(
                        TextComponent::text_string(format!("{description}\n"))
                            .color_named(NamedColor::White),
                    ),
            )
            .add_child(
                TextComponent::text("Usage: ")
                    .color_named(NamedColor::Aqua)
                    .add_child(
                        TextComponent::text_string(format!("{usage}\n"))
                            .color_named(NamedColor::White),
                    )
                    .click_event(ClickEvent::SuggestCommand(format!("{tree}").into())),
            );

        message = message
            .add_child(TextComponent::text_string("-".repeat(52)).color_named(NamedColor::Yellow));

        sender.send_message(message).await;

        Ok(())
    }
}

struct BaseHelpExecutor;

#[async_trait]
impl CommandExecutor for BaseHelpExecutor {
    async fn execute<'a>(
        &self,
        sender: &mut CommandSender<'a>,
        server: &Server,
        args: &ConsumedArgs<'a>,
    ) -> Result<(), CommandError> {
        let page_number = match PAGE_NUMBER_CONSUMER.find_arg_default_name(args) {
            Err(_) => 1,
            Ok(Ok(number)) => number,
            Ok(Err(())) => {
                sender
                    .send_message(
                        TextComponent::text("Invalid page number.")
                            .color(Color::Named(NamedColor::Red)),
                    )
                    .await;
                return Ok(());
            }
        };

        let mut commands: Vec<&CommandTree> = server
            .command_dispatcher
            .commands
            .values()
            .filter_map(|cmd| match cmd {
                Command::Tree(tree) => Some(tree),
                Command::Alias(_) => None,
            })
            .collect();

        commands.sort_by(|a, b| a.names[0].cmp(b.names[0]));

        let total_pages =
            (commands.len().to_i32().unwrap() + COMMANDS_PER_PAGE - 1) / COMMANDS_PER_PAGE;
        let page = page_number.min(total_pages);

        let start = (page - 1) * COMMANDS_PER_PAGE;
        let end = start + COMMANDS_PER_PAGE;
        let page_commands =
            &commands[start as usize..end.min(commands.len().to_i32().unwrap()) as usize];

        let arrow_left = if page > 1 {
            let cmd = format!("/help {}", page - 1);
            TextComponent::text("<<<")
                .color(Color::Named(NamedColor::Aqua))
                .click_event(ClickEvent::RunCommand(cmd.into()))
        } else {
            TextComponent::text("<<<").color(Color::Named(NamedColor::Gray))
        };

        let arrow_right = if page < total_pages {
            let cmd = format!("/help {}", page + 1);
            TextComponent::text(">>>")
                .color(Color::Named(NamedColor::Aqua))
                .click_event(ClickEvent::RunCommand(cmd.into()))
        } else {
            TextComponent::text(">>>").color(Color::Named(NamedColor::Gray))
        };

        let header_text = format!(" Help - Page {page}/{total_pages} ");

        let mut message = TextComponent::text("")
            .add_child(
                TextComponent::text_string("-".repeat((52 - header_text.len() - 3) / 2) + " ")
                    .color_named(NamedColor::Yellow),
            )
            .add_child(arrow_left.clone())
            .add_child(TextComponent::text(&header_text))
            .add_child(arrow_right.clone())
            .add_child(
                TextComponent::text_string(
                    " ".to_owned() + &"-".repeat((52 - header_text.len() - 3) / 2) + "\n",
                )
                .color_named(NamedColor::Yellow),
            );

        for tree in page_commands {
            message = message.add_child(
                TextComponent::text_string("/".to_owned() + &tree.names.join(", /"))
                    .color_named(NamedColor::Gold)
                    .add_child(TextComponent::text(" - ").color_named(NamedColor::Yellow))
                    .add_child(
                        TextComponent::text_string(tree.description.to_owned() + "\n")
                            .color_named(NamedColor::White),
                    )
                    .add_child(TextComponent::text("    Usage: ").color_named(NamedColor::Yellow))
                    .add_child(
                        TextComponent::text_string(format!("{tree}"))
                            .color_named(NamedColor::White),
                    )
                    .add_child(TextComponent::text("\n").color_named(NamedColor::White))
                    .click_event(ClickEvent::SuggestCommand(
                        format!("/{}", tree.names[0]).into(),
                    )),
            );
        }

        let footer_text = format!(" Page {page}/{total_pages} ");
        message = message
            .add_child(
                TextComponent::text_string("-".repeat((52 - footer_text.len() - 3) / 2) + " ")
                    .color_named(NamedColor::Yellow),
            )
            .add_child(arrow_left)
            .add_child(TextComponent::text(&footer_text))
            .add_child(arrow_right)
            .add_child(
                TextComponent::text_string(
                    " ".to_owned() + &"-".repeat((52 - footer_text.len() - 3) / 2),
                )
                .color_named(NamedColor::Yellow),
            );

        sender.send_message(message).await;

        Ok(())
    }
}

pub fn init_command_tree<'a>() -> CommandTree<'a> {
    CommandTree::new(NAMES, DESCRIPTION)
        .with_child(
            argument(ARG_COMMAND, &CommandTreeArgumentConsumer).execute(&CommandHelpExecutor),
        )
        .with_child(argument_default_name(&PAGE_NUMBER_CONSUMER).execute(&BaseHelpExecutor))
        .execute(&BaseHelpExecutor)
}
