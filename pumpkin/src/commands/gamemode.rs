use num_traits::FromPrimitive;
use pumpkin_text::TextComponent;

use crate::entity::player::GameMode;

use super::Command;

pub struct GamemodeCommand {}

impl<'a> Command<'a> for GamemodeCommand {
    const NAME: &'a str = "gamemode";

    const DESCRIPTION: &'a str = "Changes the gamemode for a Player";

    fn on_execute(sender: &mut super::CommandSender<'a>, command: String) {
        let player = sender.as_mut_player().unwrap();
        let args: Vec<&str> = command.split_whitespace().collect();

        if args.len() != 2 {
            player.send_system_message(
                TextComponent::text("Usage: /gamemode <mode>")
                    .color_named(pumpkin_text::color::NamedColor::Red),
            );
            return;
        }

        let mode_str = args[1].to_lowercase();
        match mode_str.parse() {
            Ok(mode) => {
                player.set_gamemode(mode);
                player.send_system_message(TextComponent::text(&format!(
                    "Set own game mode to {:?}",
                    mode
                )));
            }
            Err(_) => {
                // try to parse from number
                if let Ok(i) = mode_str.parse::<u8>() {
                    if let Some(mode) = GameMode::from_u8(i) {
                        player.set_gamemode(mode);
                        player.send_system_message(TextComponent::text(&format!(
                            "Set own game mode to {:?}",
                            mode
                        )));
                        return;
                    }
                }

                player.send_system_message(
                    TextComponent::text("Invalid gamemode")
                        .color_named(pumpkin_text::color::NamedColor::Red),
                );
            }
        }
    }

    // TODO: support console, (name required)
    fn player_required() -> bool {
        true
    }
}
