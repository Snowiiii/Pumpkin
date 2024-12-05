use crate::command::tree_builder::require;
use crate::command::{
    args::ConsumedArgs, tree::CommandTree, CommandError, CommandExecutor, CommandSender,
};
use crate::entity::player::PermissionLvl;
use async_trait::async_trait;
use pumpkin_core::text::click::ClickEvent;
use pumpkin_core::text::hover::HoverEvent;
use pumpkin_core::text::{color::NamedColor, TextComponent};
use std::borrow::Cow;

const NAMES: [&str; 1] = ["seed"];

const DESCRIPTION: &str = "Displays the world seed.";

struct PumpkinExecutor;

#[async_trait]
impl CommandExecutor for PumpkinExecutor {
    async fn execute<'a>(
        &self,
        sender: &mut CommandSender<'a>,
        server: &crate::server::Server,
        _args: &ConsumedArgs<'a>,
    ) -> Result<(), CommandError> {
        let seed = match sender {
            CommandSender::Player(player) => {
                player.living_entity.entity.world.level.seed.0.to_string()
            }
            _ => match server.worlds.first() {
                Some(world) => world.level.seed.0.to_string(),
                None => {
                    return Err(CommandError::GeneralCommandIssue(
                        "Unable to get Seed".to_string(),
                    ))
                }
            },
        };

        sender
            .send_message(
                TextComponent::text("Seed: [")
                    .add_child(
                        TextComponent::text(&seed.clone())
                            .hover_event(HoverEvent::ShowText(Cow::from(
                                "Click to Copy to Clipboard",
                            )))
                            .click_event(ClickEvent::CopyToClipboard(Cow::from(seed)))
                            .color_named(NamedColor::Green),
                    )
                    .add_child(TextComponent::text("]")),
            )
            .await;
        Ok(())
    }
}

pub fn init_command_tree<'a>() -> CommandTree<'a> {
    CommandTree::new(NAMES, DESCRIPTION)
        .with_child(require(&|sender| {
            sender.has_permission_lvl(PermissionLvl::Two)
        }))
        .execute(&PumpkinExecutor)
}
