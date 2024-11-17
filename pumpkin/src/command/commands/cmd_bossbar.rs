use async_trait::async_trait;
use crate::client::bossbar::{Bossbar, BossbarColor, BossbarDivisions, BossbarFlags};
use crate::command::{CommandExecutor, CommandSender};
use crate::command::args::ConsumedArgs;
use crate::command::dispatcher::CommandError;
use crate::command::tree::CommandTree;
use crate::server::Server;

const NAMES: [&str; 1] = ["bossbar"];
const DESCRIPTION: &str = "Display bossbar";

struct BossbarExecuter;

#[async_trait]
impl CommandExecutor for BossbarExecuter {
    async fn execute<'a>(&self, sender: &mut CommandSender<'a>, server: &Server, args: &ConsumedArgs<'a>) -> Result<(), CommandError> {
        if let Some(player) = sender.as_player() {

            //TODO: Handling arguments etc...
            let bossbar = Bossbar { uuid: uuid::Uuid::new_v4(), title: "Test".to_string(), health: 0.5, color: BossbarColor::Pink, division: BossbarDivisions::Notches6, flags: BossbarFlags::DragonBar};
            player.send_bossbar(bossbar).await;
        }
        Ok(())
    }
}

pub fn init_command_tree<'a>() -> CommandTree<'a> {
    CommandTree::new(NAMES, DESCRIPTION).execute(&crate::command::commands::cmd_bossbar::BossbarExecuter)
}