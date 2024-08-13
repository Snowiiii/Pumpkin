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
                TextComponent::from("Usage: /gamemode <mode>")
                    .color_named(pumpkin_text::color::NamedColor::Red),
            );
            return;
        }

        let mode_str = args[1].to_lowercase();
        match mode_str.parse() {
            Ok(mode) => {
                player.set_gamemode(mode);
                player.send_system_message(format!("Set own game mode to {:?}", mode).into());
            }
            Err(_) => {
                // try to parse from number
                match mode_str.parse::<u8>() {
                    Ok(i) => match GameMode::from_u8(i) {
                        Some(mode) => {
                            player.set_gamemode(mode);
                            player.send_system_message(
                                format!("Set own game mode to {:?}", mode).into(),
                            );
                            return;
                        }
                        None => {}
                    },
                    Err(_) => {}
                }

                player.send_system_message(
                    TextComponent::from("Invalid gamemode")
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
