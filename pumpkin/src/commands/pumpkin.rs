use pumpkin_protocol::{text::TextComponent, CURRENT_MC_PROTOCOL};

use crate::server::CURRENT_MC_VERSION;

use super::Command;

pub struct PumpkinCommand {}

impl PumpkinCommand {
    pub fn new() -> Self {
        Self {}
    }
}

impl<'a> Command<'a> for PumpkinCommand {
    const NAME: &'a str = "pumpkin";

    const DESCRIPTION: &'a str = "Displays information about Pumpkin";

    fn on_execute(sender: &mut super::CommandSender<'a>, _command: String) {
        let version = env!("CARGO_PKG_VERSION");
        let description = env!("CARGO_PKG_DESCRIPTION");
        //  sender.send_message(TextComponent::from(format!("Pumpkin {version}, {description} (Minecraft {CURRENT_MC_VERSION}, Protocol {CURRENT_MC_PROTOCOL})")).color_named(pumpkin_protocol::text::NamedColor::Green))

        // test
        sender.send_message(
            TextComponent::from("Pumpkin")
                .color_named(pumpkin_protocol::text::NamedColor::Black)
                .bold(),
        );
        sender.send_message(
            TextComponent::from("is")
                .color_named(pumpkin_protocol::text::NamedColor::DarkBlue)
                .italic(),
        );
        sender.send_message(
            TextComponent::from("such")
                .color_named(pumpkin_protocol::text::NamedColor::DarkGreen)
                .underlined(),
        );
        sender.send_message(
            TextComponent::from("a")
                .color_named(pumpkin_protocol::text::NamedColor::DarkAqua)
                .strikethrough(),
        );
        sender.send_message(
            TextComponent::from("super")
                .color_named(pumpkin_protocol::text::NamedColor::DarkRed)
                .obfuscated(),
        );
        sender.send_message(
            TextComponent::from("mega")
                .color_named(pumpkin_protocol::text::NamedColor::DarkPurple)
                .bold(),
        );
        sender.send_message(
            TextComponent::from("great")
                .color_named(pumpkin_protocol::text::NamedColor::Gold)
                .italic(),
        );
        sender.send_message(
            TextComponent::from("project")
                .color_named(pumpkin_protocol::text::NamedColor::Gray)
                .underlined(),
        );
        sender.send_message(TextComponent::from(""));
        sender.send_message(
            TextComponent::from("no")
                .color_named(pumpkin_protocol::text::NamedColor::DarkGray)
                .bold(),
        );
        sender.send_message(
            TextComponent::from("worries")
                .color_named(pumpkin_protocol::text::NamedColor::Blue)
                .bold(),
        );
        sender.send_message(
            TextComponent::from("there")
                .color_named(pumpkin_protocol::text::NamedColor::Green)
                .underlined(),
        );
        sender.send_message(
            TextComponent::from("will")
                .color_named(pumpkin_protocol::text::NamedColor::Aqua)
                .italic(),
        );
        sender.send_message(
            TextComponent::from("be")
                .color_named(pumpkin_protocol::text::NamedColor::Red)
                .obfuscated(),
        );
        sender.send_message(
            TextComponent::from("chunk")
                .color_named(pumpkin_protocol::text::NamedColor::LightPurple)
                .bold(),
        );
        sender.send_message(
            TextComponent::from("loading")
                .color_named(pumpkin_protocol::text::NamedColor::Yellow)
                .strikethrough(),
        );
        sender.send_message(
            TextComponent::from("soon")
                .color_named(pumpkin_protocol::text::NamedColor::White)
                .bold(),
        );
        sender.send_message(
            TextComponent::from("soon").color_named(pumpkin_protocol::text::NamedColor::White),
        );
    }
}
