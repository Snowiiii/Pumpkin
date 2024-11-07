use std::time;

use async_trait::async_trait;
use pumpkin_core::text::{color::NamedColor, TextComponent};
use pumpkin_protocol::CURRENT_MC_PROTOCOL;
use tokio::{fs::File, io::AsyncWriteExt};

use crate::{
    command::{
        args::ConsumedArgs, tree::CommandTree, tree_builder::literal, CommandExecutor, CommandSender, InvalidTreeError
    },
    server::CURRENT_MC_VERSION,
};

const NAMES: [&str; 1] = ["pumpkin"];

const DESCRIPTION: &str = "Display information about Pumpkin.";

struct PumpkinVersion;

#[async_trait]
impl CommandExecutor for PumpkinVersion {
    async fn execute<'a>(
        &self,
        sender: &mut CommandSender<'a>,
        _server: &crate::server::Server,
        _args: &ConsumedArgs<'a>,
    ) -> Result<(), InvalidTreeError> {
        let version = env!("CARGO_PKG_VERSION");
        let description = env!("CARGO_PKG_DESCRIPTION");

        sender.send_message(TextComponent::text(
             &format!("Pumpkin {version}, {description} (Minecraft {CURRENT_MC_VERSION}, Protocol {CURRENT_MC_PROTOCOL})")
         ).color_named(NamedColor::Green)).await;

        Ok(())
    }
}

struct PumpkinDump;

#[async_trait]
impl CommandExecutor for PumpkinDump {
    async fn execute<'a>(
        &self,
        sender: &mut CommandSender<'a>,
        server: &crate::server::Server,
        _args: &ConsumedArgs<'a>,
    ) -> Result<(), InvalidTreeError> {
        log::info!("Writing dump...");
        let mut f = File::create("Dump.txt").await.unwrap();
        for (idx, world) in server.worlds.iter().enumerate() {
            f.write(format!("World {idx}\n").as_bytes()).await.unwrap();
            f.write(format!("{:?}", world.level).as_bytes()).await.unwrap();
        }

        sender.send_message(TextComponent::text("Dump writted to file!")).await;
        Ok(())
    }
}

pub fn init_command_tree<'a>() -> CommandTree<'a> {
    CommandTree::new(NAMES, DESCRIPTION)
        .with_child(literal("version").execute(&PumpkinVersion))
        .with_child(literal("dump").execute(&PumpkinDump))
}
