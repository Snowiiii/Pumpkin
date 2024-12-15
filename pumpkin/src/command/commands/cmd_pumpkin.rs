use async_trait::async_trait;
use pumpkin_core::text::click::ClickEvent;
use pumpkin_core::text::hover::HoverEvent;
use pumpkin_core::text::{color::NamedColor, TextComponent};
use pumpkin_protocol::CURRENT_MC_PROTOCOL;
use std::borrow::Cow;

use crate::{
    command::{
        args::ConsumedArgs, tree::CommandTree, CommandError, CommandExecutor, CommandSender,
    },
    server::CURRENT_MC_VERSION,
    GIT_VERSION,
};

const NAMES: [&str; 2] = ["pumpkin", "version"];

const DESCRIPTION: &str = "Display information about Pumpkin.";

struct PumpkinExecutor;

const CARGO_PKG_VERSION: &str = env!("CARGO_PKG_VERSION");
const CARGO_PKG_DESCRIPTION: &str = env!("CARGO_PKG_DESCRIPTION");

#[async_trait]
impl CommandExecutor for PumpkinExecutor {
    async fn execute<'a>(
        &self,
        sender: &mut CommandSender<'a>,
        _server: &crate::server::Server,
        _args: &ConsumedArgs<'a>,
    ) -> Result<(), CommandError> {
        sender
            .send_message(
                TextComponent::text(&format!("Pumpkin {CARGO_PKG_VERSION} ({GIT_VERSION})"))
                    .hover_event(HoverEvent::ShowText(Cow::from("Click to Copy Version")))
                    .click_event(ClickEvent::CopyToClipboard(Cow::from(format!(
                        "Pumpkin {CARGO_PKG_VERSION} ({GIT_VERSION})"
                    ))))
                    .color_named(NamedColor::Green)
                    .add_child(
                        TextComponent::text(&format!(", {CARGO_PKG_DESCRIPTION}"))
                            .click_event(ClickEvent::CopyToClipboard(Cow::from(
                                CARGO_PKG_DESCRIPTION,
                            )))
                            .hover_event(HoverEvent::ShowText(Cow::from(
                                "Click to Copy Description",
                            )))
                            .color_named(NamedColor::White),
                    )
                    .add_child(
                        TextComponent::text(&format!(
                            " (Minecraft {CURRENT_MC_VERSION}, Protocol {CURRENT_MC_PROTOCOL})"
                        ))
                        .click_event(ClickEvent::CopyToClipboard(Cow::from(format!(
                            "(Minecraft {CURRENT_MC_VERSION}, Protocol {CURRENT_MC_PROTOCOL})"
                        ))))
                        .hover_event(HoverEvent::ShowText(Cow::from(
                            "Click to Copy Minecraft Version",
                        )))
                        .color_named(NamedColor::Gold),
                    )
                    .add_child(TextComponent::text(" "))
                    // https://snowiiii.github.io/Pumpkin/
                    .add_child(
                        TextComponent::text("Github Repository")
                            .click_event(ClickEvent::OpenUrl(Cow::from(
                                "https://github.com/Snowiiii/Pumpkin",
                            )))
                            .hover_event(HoverEvent::ShowText(Cow::from(
                                "Click to open repository.",
                            )))
                            .color_named(NamedColor::Blue)
                            .bold()
                            .underlined(),
                    )
                    // Added docs. and a space for spacing
                    .add_child(TextComponent::text(" "))
                    .add_child(
                        TextComponent::text("Docs")
                            .click_event(ClickEvent::OpenUrl(Cow::from(
                                "https://snowiiii.github.io/Pumpkin/",
                            )))
                            .hover_event(HoverEvent::ShowText(Cow::from("Click to open docs.")))
                            .color_named(NamedColor::Blue)
                            .bold()
                            .underlined(),
                    ),
            )
            .await;
        Ok(())
    }
}

pub fn init_command_tree<'a>() -> CommandTree<'a> {
    CommandTree::new(NAMES, DESCRIPTION).execute(&PumpkinExecutor)
}
