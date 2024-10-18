use crate::commands::tree::CommandTree;
use crate::commands::tree::RawArgs;
use crate::commands::tree_builder::argument;
use crate::commands::CommandSender;
use crate::server::Server;
use pumpkin_core::math::vector3::Vector3;
use pumpkin_core::text::{color::NamedColor, TextComponent};
use regex::Regex;

const NAMES: [&str; 1] = ["say"];
const DESCRIPTION: &str = "Sends a message to all players.";

const ARG_CONTENT: &str = "content";

pub fn consume_arg_content(_src: &CommandSender, args: &mut RawArgs) -> Option<String> {
    args.pop().map(|v| v.to_string())
}

pub fn init_command_tree<'a>() -> CommandTree<'a> {
    CommandTree::new(NAMES, DESCRIPTION).with_child(
        argument(ARG_CONTENT, consume_arg_content).execute(&|sender, server, args| {
            if let Some(content) = args.get("content") {
                let content = parse_selectors(content, sender, server);
                let message = &format!("[Console]: {content}");
                let message = TextComponent::text(message).color_named(NamedColor::Blue);

                server.broadcast_message(message.clone());
                sender.send_message(message);
            } else {
                sender.send_message(
                    TextComponent::text("Please provide a message: say [content]")
                        .color_named(NamedColor::Red),
                );
            }

            Ok(())
        }),
    )
}

fn parse_selectors(content: &str, sender: &CommandSender, server: &Server) -> String {
    let mut final_message = String::new();
    let mut current_pos = 0;

    // regex to match
    // (@p, @a, @r, @e, @s, @here)
    let selector_pattern = Regex::new(r"@[parehs]\b").unwrap();

    for selector_match in selector_pattern.find_iter(content) {
        let selector = selector_match.as_str();
        let start = selector_match.start();

        // before selector
        final_message.push_str(&content[current_pos..start]);

        let result = match selector {
            "@p" => {
                let position = match sender {
                    CommandSender::Player(p) => p.last_position.load(),
                    _ => Vector3::new(0., 0., 0.),
                };
                vec![server
                    .get_nearest_player(&position)
                    .map_or_else(|| String::from("nobody"), |p| p.gameprofile.name.clone())]
            }
            "@r" => {
                let online_players: Vec<String> = server
                    .get_online_players()
                    .map(|p| p.gameprofile.name.clone())
                    .collect();

                if online_players.is_empty() {
                    vec![String::from("nobody")]
                } else {
                    vec![online_players[rand::random::<usize>() % online_players.len()].clone()]
                }
            }
            "@s" => match sender {
                CommandSender::Player(p) => vec![p.gameprofile.name.clone()],
                _ => vec![String::from("console")],
            },
            "@a" => server
                .get_online_players()
                .map(|p| p.gameprofile.name.clone())
                .collect::<Vec<_>>(),
            "@e" => vec![], // TODO
            "@here" => server
                .get_online_players()
                .map(|p| p.gameprofile.name.clone())
                .collect::<Vec<_>>(),
            _ => continue,
        };

        // formatted player names
        final_message.push_str(&format_player_names(&result));

        current_pos = selector_match.end();
    }

    final_message.push_str(&content[current_pos..]);

    final_message
}

// Helper function to format player names according to spec
// see https://minecraft.fandom.com/wiki/Commands/say
fn format_player_names(names: &[String]) -> String {
    match names.len() {
        0 => String::new(),
        1 => names[0].clone(),
        2 => format!("{} and {}", names[0], names[1]),
        _ => {
            let (last, rest) = names.split_last().unwrap();
            format!("{}, and {}", rest.join(", "), last)
        }
    }
}
