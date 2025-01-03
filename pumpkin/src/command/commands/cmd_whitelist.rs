use crate::command::tree_builder::literal;
use crate::data::whitelist_data::WHITELIST_CONFIG;
use crate::server::Server;
use crate::{
    command::{
        args::{arg_players::PlayersArgumentConsumer, Arg, ConsumedArgs},
        tree::CommandTree,
        tree_builder::argument,
        CommandError, CommandExecutor, CommandSender,
    },
    data::{ReloadJSONConfiguration, SaveJSONConfiguration},
};
use async_trait::async_trait;
use pumpkin_config::player_profile::PlayerProfile;
use pumpkin_config::BASIC_CONFIG;
use pumpkin_core::text::TextComponent;
use pumpkin_core::PermissionLvl;
use CommandError::InvalidConsumption;

const NAMES: [&str; 1] = ["whitelist"];
const DESCRIPTION: &str = "Manages the server's whitelist.";
const ARG_TARGET: &str = "player";

struct WhitelistAddExecutor;

#[async_trait]
impl CommandExecutor for WhitelistAddExecutor {
    async fn execute<'a>(
        &self,
        sender: &mut CommandSender<'a>,
        _server: &Server,
        args: &ConsumedArgs<'a>,
    ) -> Result<(), CommandError> {
        let mut whitelist_config = WHITELIST_CONFIG.write().await;

        let Some(Arg::Players(targets)) = args.get(&ARG_TARGET) else {
            return Err(InvalidConsumption(Some(ARG_TARGET.into())));
        };

        // from the command tree, the command can only be executed with one player
        let player = &targets[0];

        if whitelist_config
            .whitelist
            .iter()
            .any(|p| p.uuid == player.gameprofile.id)
        {
            let message = format!("Player {} is already whitelisted.", player.gameprofile.name);
            let msg = TextComponent::text(message);
            sender.send_message(msg).await;
        } else {
            let whitelist_entry =
                PlayerProfile::new(player.gameprofile.id, player.gameprofile.name.clone());

            whitelist_config.whitelist.push(whitelist_entry);
            whitelist_config.save();

            let player_name = &player.gameprofile.name;
            let message = format!("Added {player_name} to the whitelist.");
            let msg = TextComponent::text(message);
            sender.send_message(msg).await;
        }

        Ok(())
    }
}

struct WhitelistRemoveExecutor;

#[async_trait]
impl CommandExecutor for WhitelistRemoveExecutor {
    async fn execute<'a>(
        &self,
        sender: &mut CommandSender<'a>,
        server: &crate::server::Server,
        args: &ConsumedArgs<'a>,
    ) -> Result<(), CommandError> {
        let mut whitelist_config = WHITELIST_CONFIG.write().await;

        let Some(Arg::Players(targets)) = args.get(&ARG_TARGET) else {
            return Err(InvalidConsumption(Some(ARG_TARGET.into())));
        };

        // from the command tree, the command can only be executed with one player
        let player = &targets[0];

        if let Some(pos) = whitelist_config
            .whitelist
            .iter()
            .position(|p| p.uuid == player.gameprofile.id)
        {
            whitelist_config.whitelist.remove(pos);
            whitelist_config.save();
            let is_op = player.permission_lvl.load() >= PermissionLvl::Three;
            if *server.white_list.read().await && BASIC_CONFIG.enforce_whitelist && !is_op {
                let msg = TextComponent::text("You are not whitelisted anymore.");
                player.kick(msg).await;
            }
            let message = format!("Removed {} from the whitelist.", player.gameprofile.name);
            let msg = TextComponent::text(message);
            sender.send_message(msg).await;
        } else {
            let message = format!("Player {} is not whitelisted.", player.gameprofile.name);
            let msg = TextComponent::text(message);
            sender.send_message(msg).await;
        }

        Ok(())
    }
}

struct WhitelistListExecutor;

#[async_trait]
impl CommandExecutor for WhitelistListExecutor {
    async fn execute<'a>(
        &self,
        sender: &mut CommandSender<'a>,
        _server: &crate::server::Server,
        _args: &ConsumedArgs<'a>,
    ) -> Result<(), CommandError> {
        let whitelist_names: Vec<String> = WHITELIST_CONFIG
            .read()
            .await
            .whitelist
            .iter()
            .map(|p| p.name.clone())
            .collect();
        let message = format!("Whitelisted players: {whitelist_names:?}");
        let msg = TextComponent::text(message);
        sender.send_message(msg).await;
        Ok(())
    }
}

struct WhitelistOffExecutor;

#[async_trait]
impl CommandExecutor for WhitelistOffExecutor {
    async fn execute<'a>(
        &self,
        sender: &mut CommandSender<'a>,
        server: &Server,
        _args: &ConsumedArgs<'a>,
    ) -> Result<(), CommandError> {
        let mut whitelist = server.white_list.write().await;
        *whitelist = false;
        let msg = TextComponent::text(
            "Whitelist is now off. To persist this change, modify the configuration.toml file.",
        );
        sender.send_message(msg).await;
        Ok(())
    }
}

struct WhitelistOnExecutor;

#[async_trait]
impl CommandExecutor for WhitelistOnExecutor {
    async fn execute<'a>(
        &self,
        sender: &mut CommandSender<'a>,
        server: &Server,
        _args: &ConsumedArgs<'a>,
    ) -> Result<(), CommandError> {
        let mut whitelist = server.white_list.write().await;
        *whitelist = true;
        let msg = TextComponent::text(
            "Whitelist is now on. To persist this change, modify the configuration.toml file.",
        );
        sender.send_message(msg).await;
        Ok(())
    }
}

struct WhitelistReloadExecutor;

#[async_trait]
impl CommandExecutor for WhitelistReloadExecutor {
    async fn execute<'a>(
        &self,
        sender: &mut CommandSender<'a>,
        server: &Server,
        _args: &ConsumedArgs<'a>,
    ) -> Result<(), CommandError> {
        let mut whitelist_config = WHITELIST_CONFIG.write().await;
        whitelist_config.reload();
        // kick all players that are not whitelisted or operator if the whitelist is enforced
        if *server.white_list.read().await && BASIC_CONFIG.enforce_whitelist {
            for player in server.get_all_players().await {
                if !whitelist_config
                    .whitelist
                    .iter()
                    .any(|p| p.uuid == player.gameprofile.id)
                    && player.permission_lvl.load() < PermissionLvl::Three
                {
                    let msg = TextComponent::text("You are not whitelisted anymore.");
                    player.kick(msg).await;
                }
            }
        }

        let msg = TextComponent::text("Whitelist configuration reloaded.");
        sender.send_message(msg).await;
        Ok(())
    }
}

pub fn init_command_tree() -> CommandTree {
    CommandTree::new(NAMES, DESCRIPTION)
        .with_child(literal("add").with_child(
            argument(ARG_TARGET, PlayersArgumentConsumer).execute(WhitelistAddExecutor),
        ))
        .with_child(literal("remove").with_child(
            argument(ARG_TARGET, PlayersArgumentConsumer).execute(WhitelistRemoveExecutor),
        ))
        .with_child(literal("list").execute(WhitelistListExecutor))
        .with_child(literal("off").execute(WhitelistOffExecutor))
        .with_child(literal("on").execute(WhitelistOnExecutor))
        .with_child(literal("reload").execute(WhitelistReloadExecutor))
}
