use std::sync::Arc;

use async_trait::async_trait;
use pumpkin_config::BASIC_CONFIG;
use pumpkin_core::text::TextComponent;

use crate::{
    command::{
        args::ConsumedArgs, tree::CommandTree, CommandError, CommandExecutor, CommandSender,
    },
    entity::player::Player,
};

const NAMES: [&str; 1] = ["list"];

const DESCRIPTION: &str = "Print the list of online players.";

struct ListExecutor;

#[async_trait]
impl CommandExecutor for ListExecutor {
    async fn execute<'a>(
        &self,
        sender: &mut CommandSender<'a>,
        server: &crate::server::Server,
        _args: &ConsumedArgs<'a>,
    ) -> Result<(), CommandError> {
        let players: Vec<Arc<Player>> = server.get_all_players().await;

        let message = if players.is_empty() {
            "There are no players online."
        } else {
            &format!(
                "There are {} of a max of {} players online: {}",
                players.len(),
                BASIC_CONFIG.max_players,
                get_player_names(players)
            )
        };

        sender.send_message(TextComponent::text(message)).await;

        Ok(())
    }
}

fn get_player_names(players: Vec<Arc<Player>>) -> String {
    let mut names = String::new();
    for player in players {
        if !names.is_empty() {
            names.push_str(", ");
        }
        names.push_str(&player.gameprofile.name);
    }
    names
}

pub fn init_command_tree<'a>() -> CommandTree<'a> {
    CommandTree::new(NAMES, DESCRIPTION).execute(&ListExecutor)
}
