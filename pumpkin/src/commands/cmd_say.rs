use crate::commands::tree::CommandTree;
use crate::commands::tree::RawArgs;
use crate::commands::tree_builder::argument;
use crate::commands::CommandSender;
use crate::server::Server;
use pumpkin_core::text::{color::NamedColor, TextComponent};

const NAMES: [&str; 1] = ["say"];
const DESCRIPTION: &str = "Sends a message to all players.";

const ARG_CONTENT: &str = "content";

pub fn consume_arg_content(_src: &CommandSender, args: &mut RawArgs) -> Option<String> {
    let mut all_args: Vec<String> = args.drain(..).map(|v| v.to_string()).collect();

    if all_args.is_empty() {
        None
    } else {
        all_args.reverse();
        Some(all_args.join(" "))
    }
}

pub fn init_command_tree<'a>() -> CommandTree<'a> {
    CommandTree::new(NAMES, DESCRIPTION).with_child(
        argument(ARG_CONTENT, consume_arg_content).execute(&|sender, server, args| {
            if let Some(content) = args.get("content") {
                let content = parse_selectors(content, sender, server);
                let message = &format!("[Console]: {content}");
                let message = TextComponent::text(message).color_named(NamedColor::Blue);

                server.broadcast_message(&message);
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

    let tokens: Vec<&str> = content.split_whitespace().collect();

    for token in tokens {
        let parsed_token = parse_token(token, sender, server);
        final_message.push_str(&parsed_token);
        final_message.push(' ');
    }

    final_message.trim_end().to_string()
}

fn parse_token<'a>(token: &'a str, sender: &'a CommandSender, server: &'a Server) -> String {
    let result = match token {
        "@p" => {
            if let CommandSender::Player(player) = sender {
                server
                    .get_nearest_player_name(player)
                    .map_or_else(Vec::new, |player_name| vec![player_name])
            } else {
                return token.to_string();
            }
        }
        "@r" => {
            let online_player_names: Vec<String> = server.get_online_player_names();

            if online_player_names.is_empty() {
                vec![String::from("nobody")]
            } else {
                vec![
                    online_player_names[rand::random::<usize>() % online_player_names.len()]
                        .clone(),
                ]
            }
        }
        "@s" => match sender {
            CommandSender::Player(p) => vec![p.gameprofile.name.clone()],
            _ => vec![String::from("console")],
        },
        "@a" => server.get_online_player_names(),
        "@here" => server.get_online_player_names(),
        _ => {
            return token.to_string();
        }
    };

    format_player_names(&result)
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
